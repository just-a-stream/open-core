# engine-runner

Drivers and lazy stream stages for an `engine-sync` engine. The library
defines the stages and never drives them: the binary picks the runtime, the
channels and the tasks.

- `run` drives any `Update` with a `Result` audit over an `Iterator` and
  sends each output to an `engine-sync` `Fanout`, which picks no transport.
- `UpdateStream` folds a `Stream` and ends after the first error. The driver
  reads the state between polls.
- `execute_stream` runs every released reaction through an `Executor`
  concurrently, with no limit, and yields each event as an engine input.
