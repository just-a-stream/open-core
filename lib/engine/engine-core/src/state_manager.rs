use crate::{
    sequence::{SequenceError, SequenceValidator, Sequenced},
    snapshot::{EngineSnapshot, EngineVersion},
    state::{Update, UpdateByRef},
};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct StateManager<State> {
    sequence_validator: SequenceValidator,
    state: State,
}

impl<State, Input> Update<Sequenced<&Input>> for StateManager<State>
where
    State: UpdateByRef<Input>,
{
    type Audit = Result<Sequenced<State::Audit>, SequenceError>;

    fn update(&mut self, input: Sequenced<&Input>) -> Self::Audit {
        let sequence = self
            .sequence_validator
            .validate_and_increment(input.sequence())?;

        Ok(Sequenced::new(
            sequence,
            self.state.update_by_ref(input.into_item()),
        ))
    }
}

impl<State, Input> Update<&Sequenced<Input>> for StateManager<State>
where
    State: UpdateByRef<Input>,
{
    type Audit = Result<Sequenced<State::Audit>, SequenceError>;

    fn update(&mut self, input: &Sequenced<Input>) -> Self::Audit {
        self.update(input.as_ref())
    }
}

impl<State> StateManager<State> {
    pub const fn new(sequence_validator: SequenceValidator, state: State) -> Self {
        Self {
            sequence_validator,
            state,
        }
    }

    pub const fn sequence_validator(&self) -> &SequenceValidator {
        &self.sequence_validator
    }

    pub const fn state(&self) -> &State {
        &self.state
    }

    pub const fn snapshot(&self, version: EngineVersion) -> EngineSnapshot<&State> {
        EngineSnapshot::new(version, self.sequence_validator, &self.state)
    }
}
