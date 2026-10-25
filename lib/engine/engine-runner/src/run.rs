use derive_more::{Display, Error};
use engine_sync::{Fanout, FanoutFailure, Update};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Display, Error)]
pub enum RunError<UpdateError, FanoutError> {
    #[display("state update failed")]
    Update(#[error(source)] UpdateError),
    #[display("fanout failed")]
    Fanout(#[error(source)] FanoutError),
}

pub fn run<State, Inputs, Output, UpdateError, Fan>(
    state: &mut State,
    inputs: Inputs,
    fanout: &mut Fan,
) -> Result<(), RunError<UpdateError, FanoutFailure<Fan::Pending, Fan::Error>>>
where
    Inputs: IntoIterator,
    State: Update<Inputs::Item, Audit = Result<Output, UpdateError>>,
    Fan: Fanout<Output> + ?Sized,
{
    inputs.into_iter().try_for_each(|input| {
        let output = state.update(input).map_err(RunError::Update)?;

        fanout.send(output).map_err(RunError::Fanout)
    })
}
