mod replica;
mod machine;

pub trait Update<Input> {
    type Audit;

    fn process(&mut self, input: &Input) -> Self::Audit;
}