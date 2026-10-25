use futures::{Stream, StreamExt as _};

pub trait Executor<Reaction> {
    type Event;
    type Events: Stream<Item = Self::Event>;

    fn execute(&self, reaction: Reaction) -> Self::Events;
}

pub fn execute_stream<Input, Reactions, Exec>(
    reactions: Reactions,
    executor: Exec,
) -> impl Stream<Item = Input>
where
    Reactions: Stream,
    Exec: Executor<Reactions::Item>,
    Exec::Events: Unpin,
    Exec::Event: Into<Input>,
{
    reactions
        .map(move |reaction| executor.execute(reaction))
        .flatten_unordered(None)
        .map(Into::into)
}
