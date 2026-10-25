use derive_more::{Display, Error};

pub trait Fanout<Output> {
    type Pending;
    type Error;

    fn send(&mut self, output: Output) -> Result<(), FanoutFailure<Self::Pending, Self::Error>>;
}

impl<FnSend, Output, Pending, Error> Fanout<Output> for FnSend
where
    FnSend: FnMut(Output) -> Result<(), FanoutFailure<Pending, Error>>,
{
    type Pending = Pending;
    type Error = Error;

    #[inline]
    fn send(&mut self, output: Output) -> Result<(), FanoutFailure<Pending, Error>> {
        self(output)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Display, Error)]
#[display("fanout send failed")]
pub struct FanoutFailure<Pending, Error> {
    pub pending: Pending,
    pub source: Error,
}

impl<Pending, Error> FanoutFailure<Pending, Error> {
    pub const fn new(pending: Pending, source: Error) -> Self {
        Self { pending, source }
    }
}
