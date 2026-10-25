use crate::{sequence::SequenceValidator, state_manager::StateManager};
use derive_more::{Constructor, Display, Error};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct EngineSnapshot<State> {
    version: EngineVersion,
    sequence_validator: SequenceValidator,
    state: State,
}

impl<State> EngineSnapshot<State> {
    pub(crate) const fn new(
        version: EngineVersion,
        sequence_validator: SequenceValidator,
        state: State,
    ) -> Self {
        Self {
            version,
            sequence_validator,
            state,
        }
    }

    pub const fn version(&self) -> EngineVersion {
        self.version
    }

    pub fn try_into_state_manager(
        self,
        version_expected: EngineVersion,
    ) -> Result<StateManager<State>, EngineVersionMismatch> {
        if self.version != version_expected {
            return Err(EngineVersionMismatch {
                expected: version_expected,
                found: self.version,
            });
        }

        Ok(StateManager::new(self.sequence_validator, self.state))
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[derive(Constructor, Display)]
pub struct EngineVersion(u32);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Display, Error)]
#[display("engine version {expected} cannot restore a snapshot from version {found}")]
pub struct EngineVersionMismatch {
    pub expected: EngineVersion,
    pub found: EngineVersion,
}
