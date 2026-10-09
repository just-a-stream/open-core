mod replica;
mod machine;

pub trait Update<Input> {
    type Audit;

    fn process(&mut self, input: Input) -> Self::Audit;
}

pub trait UpdateByRef<Input>
where
    Input: ?Sized,
{
    type Audit;

    fn process_ref(&mut self, input: &Input) -> Self::Audit;
}

impl<Input, State, Audit> UpdateByRef<Input> for State
where
    Input: ?Sized,
    State: ?Sized + for<'a> Update<&'a Input, Audit = Audit>,
{
    type Audit = Audit;

    fn process_ref(&mut self, input: &Input) -> Self::Audit {
        self.process(input)
    }
}
