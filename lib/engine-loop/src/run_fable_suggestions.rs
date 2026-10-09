use crate::{reactor::Reactor, state::UpdateByRef};

pub trait Repository<Event, Reaction> {
    type Error;

    fn commit(&mut self, observed: &Event, decided: &[Reaction]) -> Result<(), Self::Error>;
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Recorded<Event, Reaction> {
    Observed(Event),
    Decided(Reaction),
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Replayed<State, Event, Reaction> {
    Snapshot(State),
    Recorded(Recorded<Event, Reaction>),
}

pub fn run<R, Event, Audit, Repo, Error>(
    state: &mut R::State,
    reactor: &R,
    repository: &mut Repo,
    events: impl IntoIterator<Item = Result<Event, Error>>,
    mut route: impl FnMut(R::Reaction),
    mut audit: impl FnMut(Audit),
) -> Result<(), Error>
where
    R: Reactor<Event>,
    R::State: UpdateByRef<Event, Audit = Audit> + UpdateByRef<R::Reaction, Audit = Audit>,
    Repo: Repository<Event, R::Reaction>,
    Error: From<Repo::Error>,
{
    let mut decided = Vec::new();
    let mut audits = Vec::new();

    events.into_iter().try_for_each(|event| {
        let event = event?;
        step(state, reactor, &event, &mut decided, &mut audits);
        repository.commit(&event, &decided)?;

        audits.drain(..).for_each(&mut audit);
        decided.drain(..).for_each(&mut route);

        Ok(())
    })
}

fn step<R, Event, Audit>(
    state: &mut R::State,
    reactor: &R,
    event: &Event,
    decided: &mut Vec<R::Reaction>,
    audits: &mut Vec<Audit>,
) where
    R: Reactor<Event>,
    R::State: UpdateByRef<Event, Audit = Audit> + UpdateByRef<R::Reaction, Audit = Audit>,
{
    audits.push(state.process_ref(event));
    reactor.react(state, event, |reaction| decided.push(reaction));
    audits.extend(decided.iter().map(|reaction| state.process_ref(reaction)));
}

pub fn replay<State, Event, Reaction, Error>(
    state: &mut State,
    history: impl IntoIterator<Item = Result<Replayed<State, Event, Reaction>, Error>>,
) -> Result<(), Error>
where
    State: UpdateByRef<Event> + UpdateByRef<Reaction>,
{
    history.into_iter().try_for_each(|replayed| {
        match replayed? {
            Replayed::Snapshot(snapshot) => *state = snapshot,
            Replayed::Recorded(Recorded::Observed(event)) => {
                state.process_ref(&event);
            }
            Replayed::Recorded(Recorded::Decided(reaction)) => {
                state.process_ref(&reaction);
            }
        }

        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::Update;
    use std::{cell::RefCell, rc::Rc};

    #[derive(Debug, Clone, Eq, PartialEq, Default)]
    struct Till {
        total: u64,
        refunds_in_flight: u64,
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    struct Deposit(u64);

    #[derive(Debug, Clone, Eq, PartialEq)]
    struct Refund(u64);

    impl Update<&Deposit> for Till {
        type Audit = String;

        fn process(&mut self, Deposit(amount): &Deposit) -> String {
            self.total += amount;
            format!("deposited {amount}")
        }
    }

    impl Update<&Refund> for Till {
        type Audit = String;

        fn process(&mut self, Refund(amount): &Refund) -> String {
            self.total -= amount;
            self.refunds_in_flight += 1;
            format!("refund of {amount} in flight")
        }
    }

    struct RefundOverTen;

    impl Reactor<Deposit> for RefundOverTen {
        type State = Till;
        type Reaction = Refund;

        fn react(&self, state: &Till, _: &Deposit, mut emit: impl FnMut(Refund)) {
            if state.total > 10 {
                emit(Refund(state.total - 10));
            }
        }
    }

    type Log = Rc<RefCell<Vec<Recorded<Deposit, Refund>>>>;

    #[derive(Debug, Default)]
    struct Journal {
        log: Log,
        refuse: bool,
    }

    impl Repository<Deposit, Refund> for Journal {
        type Error = &'static str;

        fn commit(&mut self, observed: &Deposit, decided: &[Refund]) -> Result<(), &'static str> {
            if self.refuse {
                return Err("refused");
            }
            let mut log = self.log.borrow_mut();
            log.push(Recorded::Observed(observed.clone()));
            log.extend(decided.iter().cloned().map(Recorded::Decided));

            Ok(())
        }
    }

    fn deposits(amounts: &[u64]) -> Vec<Result<Deposit, &'static str>> {
        amounts.iter().map(|amount| Ok(Deposit(*amount))).collect()
    }

    #[test]
    fn an_event_and_its_decisions_are_committed_before_any_decision_is_routed() {
        let mut till = Till::default();
        let mut journal = Journal::default();
        let log = Rc::clone(&journal.log);
        let mut routed = Vec::new();
        let mut audits = Vec::new();

        run(
            &mut till,
            &RefundOverTen,
            &mut journal,
            deposits(&[4, 9]),
            |refund| routed.push((refund, log.borrow().len())),
            |audit| audits.push(audit),
        )
        .unwrap();

        assert_eq!(routed, [(Refund(3), 3)]);
        assert_eq!(
            audits,
            ["deposited 4", "deposited 9", "refund of 3 in flight"]
        );
        assert_eq!(
            till,
            Till {
                total: 10,
                refunds_in_flight: 1
            }
        );
    }

    #[test]
    fn a_refused_commit_ends_the_run_before_routing_or_auditing() {
        let mut till = Till::default();
        let mut journal = Journal {
            refuse: true,
            ..Journal::default()
        };
        let mut routed = Vec::new();
        let mut audits = Vec::<String>::new();

        let ran = run(
            &mut till,
            &RefundOverTen,
            &mut journal,
            deposits(&[20]),
            |refund| routed.push(refund),
            |audit| audits.push(audit),
        );

        assert_eq!(ran, Err("refused"));
        assert!(routed.is_empty());
        assert!(audits.is_empty());
    }

    #[test]
    fn replaying_the_log_rebuilds_the_state_without_deciding_again() {
        let mut till = Till::default();
        let mut journal = Journal::default();
        run(
            &mut till,
            &RefundOverTen,
            &mut journal,
            deposits(&[8, 7, 30]),
            |_| {},
            |_| {},
        )
        .unwrap();
        let history: Vec<Result<_, &str>> = journal
            .log
            .borrow()
            .iter()
            .cloned()
            .map(|recorded| Ok(Replayed::Recorded(recorded)))
            .collect();

        let mut replayed = Till::default();
        replay(&mut replayed, history).unwrap();

        assert_eq!(replayed, till);
    }

    #[test]
    fn a_snapshot_replaces_the_state_and_later_records_fold_on_top() {
        let snapshot = Till {
            total: 6,
            refunds_in_flight: 2,
        };
        let history: [Result<Replayed<Till, Deposit, Refund>, &str>; 3] = [
            Ok(Replayed::Recorded(Recorded::Observed(Deposit(100)))),
            Ok(Replayed::Snapshot(snapshot)),
            Ok(Replayed::Recorded(Recorded::Decided(Refund(1)))),
        ];
        let mut till = Till::default();

        replay(&mut till, history).unwrap();

        assert_eq!(
            till,
            Till {
                total: 5,
                refunds_in_flight: 3
            }
        );
    }
}
