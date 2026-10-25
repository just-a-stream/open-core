use derive_more::{Display, Error};
use engine_sync::UpdateByRef;
use futures::{Stream, ready};
use pin_project_lite::pin_project;
use std::{
    pin::Pin,
    task::{Context, Poll},
};

pin_project! {
    #[derive(Debug)]
    pub struct UpdateStream<Inputs, State> {
        #[pin]
        inputs: Inputs,
        state: State,
        ended: bool,
    }
}

impl<Inputs, State, Audit, Error> Stream for UpdateStream<Inputs, State>
where
    Inputs: Stream,
    State: UpdateByRef<Inputs::Item, Audit = Result<Audit, Error>>,
{
    type Item = Result<Audit, UpdateFailure<Inputs::Item, Error>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        if *this.ended {
            return Poll::Ready(None);
        }

        let Some(input) = ready!(this.inputs.poll_next(cx)) else {
            *this.ended = true;
            return Poll::Ready(None);
        };

        let audit = this.state.update_by_ref(&input).map_err(|source| {
            *this.ended = true;
            UpdateFailure::new(input, source)
        });

        Poll::Ready(Some(audit))
    }
}

impl<Inputs, State> UpdateStream<Inputs, State> {
    pub const fn new(inputs: Inputs, state: State) -> Self {
        Self {
            inputs,
            state,
            ended: false,
        }
    }

    pub const fn state(&self) -> &State {
        &self.state
    }

    pub fn into_state(self) -> State {
        self.state
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Display, Error)]
#[display("update failed")]
pub struct UpdateFailure<Input, Error> {
    pub input: Input,
    pub source: Error,
}

impl<Input, Error> UpdateFailure<Input, Error> {
    pub const fn new(input: Input, source: Error) -> Self {
        Self { input, source }
    }
}
