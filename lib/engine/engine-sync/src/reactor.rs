pub trait Reactor<Input, State, StateAudit> {
    type Reaction;

    fn react(&self, input: &Input, state: &State, audit: &StateAudit) -> Self::Reaction;
}

impl<FnReact, Input, State, StateAudit, Reaction> Reactor<Input, State, StateAudit> for FnReact
where
    FnReact: Fn(&Input, &State, &StateAudit) -> Reaction,
{
    type Reaction = Reaction;

    #[inline]
    fn react(&self, input: &Input, state: &State, audit: &StateAudit) -> Reaction {
        self(input, state, audit)
    }
}
