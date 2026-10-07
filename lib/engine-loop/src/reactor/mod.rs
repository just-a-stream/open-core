

pub trait Reactor<Event> {
    type State;
    type Reaction;

    fn react(&self, state: &Self::State, event: &Event, emit: impl FnMut(Self::Reaction));
}

pub trait ReactorMut<Event> {
    type State;
    type Reaction;

    fn react(&mut self, state: &Self::State, event: &Event, emit: impl FnMut(Self::Reaction));
}