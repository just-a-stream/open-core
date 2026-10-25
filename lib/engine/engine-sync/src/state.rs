pub trait Update<Input> {
    type Audit;

    fn update(&mut self, input: Input) -> Self::Audit;
}

impl<Input, State> Update<Input> for &mut State
where
    State: Update<Input> + ?Sized,
{
    type Audit = State::Audit;

    fn update(&mut self, input: Input) -> Self::Audit {
        (**self).update(input)
    }
}

pub trait UpdateByRef<Input> {
    type Audit;

    fn update_by_ref(&mut self, input: &Input) -> Self::Audit;
}

impl<Input, State, Audit> UpdateByRef<Input> for State
where
    State: for<'a> Update<&'a Input, Audit = Audit>,
{
    type Audit = Audit;

    fn update_by_ref(&mut self, input: &Input) -> Audit {
        self.update(input)
    }
}
