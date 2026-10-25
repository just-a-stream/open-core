#![doc = include_str!("../README.md")]

pub use engine::{Engine, InputRejected, Output, ReactionSent};
pub use fanout::{Fanout, FanoutFailure};
pub use reactor::Reactor;
pub use sequence::{
    Sequence, SequenceError, SequenceUnexpected, SequenceValidator, Sequenced, Sequencer,
};
pub use snapshot::{EngineSnapshot, EngineVersion, EngineVersionMismatch};
pub use state::{Update, UpdateByRef};
pub use state_manager::StateManager;

mod engine;
mod fanout;
mod reactor;
mod sequence;
mod snapshot;
mod state;
mod state_manager;
