use crate::{
    reactor::Reactor,
    sequence::{SequenceError, SequenceValidator, Sequenced},
    snapshot::{EngineSnapshot, EngineVersion},
    state::{Update, UpdateByRef},
    state_manager::StateManager,
};
use derive_more::{Display, Error};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct Engine<State, Strategy> {
    state: StateManager<State>,
    reactor: Strategy,
}

impl<State, Strategy, Input> Update<Sequenced<Input>> for Engine<State, Strategy>
where
    Input: From<ReactionSent<Strategy::Reaction>>,
    State: UpdateByRef<Input>,
    Strategy: Reactor<Input, State, <State as UpdateByRef<Input>>::Audit>,
{
    type Audit = Result<
        Sequenced<Output<Input, <State as UpdateByRef<Input>>::Audit, Strategy::Reaction>>,
        InputRejected<Input>,
    >;

    fn update(&mut self, input: Sequenced<Input>) -> Self::Audit {
        let audit = match self.state.update(input.as_ref()) {
            Ok(audit) => audit,
            Err(source) => return Err(InputRejected::new(input, source)),
        };

        let sequence = input.sequence();
        let item = input.into_item();
        let audit = audit.into_item();
        let reaction = self.reactor.react(&item, self.state.state(), &audit);

        Ok(Sequenced::new(sequence, Output::new(item, audit, reaction)))
    }
}

impl<State, Strategy> Engine<State, Strategy> {
    pub const fn new(state: StateManager<State>, reactor: Strategy) -> Self {
        Self { state, reactor }
    }

    pub const fn sequence_validator(&self) -> &SequenceValidator {
        self.state.sequence_validator()
    }

    pub const fn state(&self) -> &State {
        self.state.state()
    }

    pub const fn snapshot(&self, version: EngineVersion) -> EngineSnapshot<&State> {
        self.state.snapshot(version)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Output<Input, Audit, Reaction> {
    pub input: Input,
    pub audit: Audit,
    pub reaction: Reaction,
}

impl<Input, Audit, Reaction> Output<Input, Audit, Reaction> {
    pub const fn new(input: Input, audit: Audit, reaction: Reaction) -> Self {
        Self {
            input,
            audit,
            reaction,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ReactionSent<Reaction> {
    pub reaction: Reaction,
}

impl<Reaction> ReactionSent<Reaction> {
    pub const fn new(reaction: Reaction) -> Self {
        Self { reaction }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Display, Error)]
#[display("input rejected")]
pub struct InputRejected<Input> {
    pub input: Sequenced<Input>,
    pub source: SequenceError,
}

impl<Input> InputRejected<Input> {
    pub const fn new(input: Sequenced<Input>, source: SequenceError) -> Self {
        Self { input, source }
    }
}
