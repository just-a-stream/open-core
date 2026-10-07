
pub trait StateTransition<Event> {
    type Next;

    fn transition(self, event: Event) -> Self::Next;
}

pub trait TryStateTransition<Event> {
    type Next;
    type Error;

    fn try_transition(self, event: Event) -> Result<Self::Next, Self::Error>;
}
