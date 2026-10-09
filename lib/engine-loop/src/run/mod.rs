use crate::{reactor::Reactor, state::UpdateByRef};

enum StateAudit<E, R> {
    Event(E),
    Reaction(R)
}

fn next<R, Event>(
    state: &mut R::State,
    reactor: &R,
    event: Event,
    mut audit: impl FnMut(StateAudit<<R::State as UpdateByRef<Event>>::Audit, <R::State as UpdateByRef<R::Reaction>>::Audit>),
    reaction_buff: &mut Vec<R::Reaction>,
)
where
    R: Reactor<Event>,
    R::State: UpdateByRef<Event> + UpdateByRef<R::Reaction>,
{
    audit(StateAudit::Event(state.process_ref(&event)));

    reactor.react(
        &state,
        &event,
        |reaction| reaction_buff.push(reaction)
    );

    for reaction in reaction_buff.iter() {
        audit(StateAudit::Reaction(state.process_ref(reaction)));
    }
}

// fn next_new<R, Event>(
//     state: &mut R::State,
//     reactor: &R,
//     event: Event,
//     mut audit: impl FnMut()
// )
// where
//     R: Reactor<Event>,
// {
//     process_with_audit(state, )
//
// }

fn process_reactions<'a, Reactions, Reaction, State>(
    reactions: Reactions,
    state: &mut State,
    mut audit: impl FnMut(State::Audit),
)
where
    Reactions: IntoIterator<Item = &'a Reaction>,
    Reaction: 'a,
    State: UpdateByRef<Reaction>,
{
    let mut reactions = reactions.into_iter();
    while let Some(reaction) = reactions.next() {
        audit(state.process_ref(reaction))
    }
}

fn process_and_react<R, Event>(
    state: &mut R::State,
    reactor: &R,
    event: &Event,
    audit: impl FnMut(<R::State as UpdateByRef<Event>>::Audit),
    emit: impl FnMut(R::Reaction)
)
where
    R: Reactor<Event>,
    R::State: UpdateByRef<Event>,
{
    process_with_audit(state, event, audit);
    react(reactor, state, event, emit);
}

fn process_with_audit<State, Event>(
    state: &mut State,
    event: &Event,
    mut audit: impl FnMut(State::Audit)
)
where
    State: UpdateByRef<Event>,
{
    audit(state.process_ref(event));
}

fn react<R, Event>(
    reactor: &R,
    state: &R::State,
    event: &Event,
    emit: impl FnMut(R::Reaction)
)
where
    R: Reactor<Event>,
    R::State: UpdateByRef<R::Reaction>,
{
    reactor.react(state, event, emit)
}



// struct Audit {
//     input: Event,
//     update_audit: Audit,
// }
//
// struct StateAudits<Event, Audit> {
//     event: Event,
//     event_update: Audit,
//     reaction_update: Audit,
// }


// #[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
// pub enum Recorded<Event, Reaction> {
//     Observed(Event),
//     Decided(Reaction),
// }
//
// impl<Event, Reaction> Recorded<Event, Reaction> {
//     pub const fn as_ref(&self) -> Recorded<&Event, &Reaction> {
//         match self {
//             Self::Observed(event) => Recorded::Observed(event),
//             Self::Decided(reaction) => Recorded::Decided(reaction),
//         }
//     }
// }
//
// pub trait Repository<State, Event, Reaction> {
//     type Error;
//
//     fn commit(
//         &mut self,
//         recorded: &[Recorded<Event, Reaction>],
//         state: &State,
//     ) -> Result<(), Self::Error>;
// }
//
// #[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
// pub enum SnapshotOrUpdate<State, Update> {
//     Snapshot(State),
//     Update(Update),
// }
//
// pub fn run<State, Decider, Event, Audit, Repo, Error>(
//     state: &mut State,
//     decider: &Decider,
//     repository: &mut Repo,
//     events: impl IntoIterator<Item = Result<Event, Error>>,
//     mut route: impl FnMut(Decider::Reaction),
//     mut audit: impl FnMut(Audit),
// ) -> Result<(), Error>
// where
//     State: for<'a> Update<Recorded<&'a Event, &'a Decider::Reaction>, Audit = Audit>,
//     Decider: Reactor<Event, State = State>,
//     Repo: Repository<State, Event, Decider::Reaction>,
//     Error: From<Repo::Error>,
// {
//     events.into_iter().try_for_each(|event| {
//         let event = event?;
//         let audits = state.process(Recorded::Observed(&event));
//         let mut reactions = Vec::new();
//         decider.react(state, &event, |reaction| reactions.push(reaction));
//
//         let audits_decided = reactions
//             .iter()
//             .map(|reaction| state.process(Recorded::Decided(reaction)))
//             .collect::<Vec<_>>();
//         let recorded = Some(Recorded::Observed(event))
//             .into_iter()
//             .chain(reactions.into_iter().map(Recorded::Decided))
//             .collect::<Vec<_>>();
//
//         repository.commit(&recorded, state)?;
//
//         audit(audits);
//         audits_decided.into_iter().for_each(&mut audit);
//         recorded.into_iter().for_each(|recorded| {
//             if let Recorded::Decided(reaction) = recorded {
//                 route(reaction);
//             }
//         });
//
//         Ok(())
//     })
// }
//
// pub fn replay<State, Event, Reaction, Error>(
//     state: &mut State,
//     history: impl IntoIterator<Item = Result<SnapshotOrUpdate<State, Recorded<Event, Reaction>>, Error>>,
// ) -> Result<(), Error>
// where
//     State: for<'a> Update<Recorded<&'a Event, &'a Reaction>>,
// {
//     history.into_iter().try_for_each(|entry| {
//         match entry? {
//             SnapshotOrUpdate::Snapshot(snapshot) => *state = snapshot,
//             SnapshotOrUpdate::Update(recorded) => {
//                 state.process(recorded.as_ref());
//             }
//         }
//
//         Ok(())
//     })
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::{cell::RefCell, rc::Rc};
//
//     #[derive(Debug, Clone, Eq, PartialEq, Default)]
//     struct Till {
//         total: u64,
//         refunds: u64,
//     }
//
//     #[derive(Debug, Clone, Eq, PartialEq)]
//     struct Deposit(u64);
//
//     #[derive(Debug, Clone, Eq, PartialEq)]
//     struct Refund(u64);
//
//     impl Update<Recorded<&Deposit, &Refund>> for Till {
//         type Audit = String;
//
//         fn process(&mut self, input: Recorded<&Deposit, &Refund>) -> String {
//             match input {
//                 Recorded::Observed(Deposit(amount)) => {
//                     self.total += amount;
//                     format!("deposited {amount}")
//                 }
//                 Recorded::Decided(Refund(amount)) => {
//                     self.total -= amount;
//                     self.refunds += 1;
//                     format!("refunded {amount}")
//                 }
//             }
//         }
//     }
//
//     #[derive(Debug)]
//     struct RefundOverTen;
//
//     impl Reactor<Deposit> for RefundOverTen {
//         type State = Till;
//         type Reaction = Refund;
//
//         fn react(&self, state: &Till, _event: &Deposit, mut emit: impl FnMut(Refund)) {
//             if state.total > 10 {
//                 emit(Refund(state.total - 10));
//             }
//         }
//     }
//
//     type Log = Rc<RefCell<Vec<Recorded<Deposit, Refund>>>>;
//
//     #[derive(Debug, Default)]
//     struct Journal {
//         log: Log,
//         refuse: bool,
//     }
//
//     impl Repository<Till, Deposit, Refund> for Journal {
//         type Error = &'static str;
//
//         fn commit(
//             &mut self,
//             recorded: &[Recorded<Deposit, Refund>],
//             _state: &Till,
//         ) -> Result<(), &'static str> {
//             if self.refuse {
//                 return Err("refused");
//             }
//             self.log.borrow_mut().extend_from_slice(recorded);
//
//             Ok(())
//         }
//     }
//
//     fn deposits(amounts: &[u64]) -> Vec<Result<Deposit, &'static str>> {
//         amounts.iter().map(|amount| Ok(Deposit(*amount))).collect()
//     }
//
//     #[test]
//     fn an_event_and_its_decisions_are_committed_before_any_decision_is_routed() {
//         let mut till = Till::default();
//         let mut journal = Journal::default();
//         let log = Rc::clone(&journal.log);
//         let mut routed = Vec::new();
//         let mut audits = Vec::new();
//
//         run(
//             &mut till,
//             &RefundOverTen,
//             &mut journal,
//             deposits(&[4, 9]),
//             |refund| routed.push((refund, log.borrow().len())),
//             |audit| audits.push(audit),
//         )
//         .unwrap();
//
//         assert_eq!(routed, [(Refund(3), 3)]);
//         assert_eq!(audits, ["deposited 4", "deposited 9", "refunded 3"]);
//         assert_eq!(
//             till,
//             Till {
//                 total: 10,
//                 refunds: 1
//             }
//         );
//     }
//
//     #[test]
//     fn a_refused_commit_stops_the_loop_before_any_decision_is_routed() {
//         let mut till = Till::default();
//         let mut journal = Journal {
//             refuse: true,
//             ..Journal::default()
//         };
//         let mut routed = Vec::new();
//
//         let ran = run(
//             &mut till,
//             &RefundOverTen,
//             &mut journal,
//             deposits(&[20, 5]),
//             |refund| routed.push(refund),
//             |_| {},
//         );
//
//         assert_eq!(ran, Err("refused"));
//         assert_eq!(routed, []);
//         assert!(journal.log.borrow().is_empty());
//     }
//
//     #[test]
//     fn replaying_the_committed_log_rebuilds_the_state_without_reacting() {
//         let mut till = Till::default();
//         let mut journal = Journal::default();
//         run(
//             &mut till,
//             &RefundOverTen,
//             &mut journal,
//             deposits(&[8, 7, 30]),
//             |_| {},
//             |_| {},
//         )
//         .unwrap();
//         let history = journal
//             .log
//             .borrow()
//             .iter()
//             .cloned()
//             .map(|recorded| Ok::<_, &'static str>(SnapshotOrUpdate::Update(recorded)))
//             .collect::<Vec<_>>();
//
//         let mut replayed = Till::default();
//         replay(&mut replayed, history).unwrap();
//
//         assert_eq!(replayed, till);
//     }
//
//     #[test]
//     fn a_snapshot_replaces_the_state_and_later_updates_fold_on_top() {
//         let snapshot = Till {
//             total: 6,
//             refunds: 2,
//         };
//         let history: [Result<SnapshotOrUpdate<Till, Recorded<Deposit, Refund>>, &'static str>; 3] = [
//             Ok(SnapshotOrUpdate::Update(Recorded::Observed(Deposit(100)))),
//             Ok(SnapshotOrUpdate::Snapshot(snapshot)),
//             Ok(SnapshotOrUpdate::Update(Recorded::Decided(Refund(1)))),
//         ];
//         let mut till = Till::default();
//
//         replay(&mut till, history).unwrap();
//
//         assert_eq!(
//             till,
//             Till {
//                 total: 5,
//                 refunds: 3
//             }
//         );
//     }
// }
