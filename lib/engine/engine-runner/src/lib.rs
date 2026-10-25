#![doc = include_str!("../README.md")]

pub use executor::{Executor, execute_stream};
pub use run::{RunError, run};
pub use update_stream::{UpdateFailure, UpdateStream};

mod executor;
mod run;
mod update_stream;
