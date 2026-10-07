# engine-core

The event-loop contracts: `Update`, a state that processes one input at a time and returns its
audit, `Reactor`, which answers each event from the state that event made, and `run`/`replay`,
the synchronous loop that drives them through a `Repository`.

## Provenance

Copied from cerebrum's `lib/engine/core` (package `engine-core`) at
`0e316f0f5438dfe2fe96cdad31546508ec13a914`, comments stripped.

- Kept: `Processor`, renamed `Update`, its output renamed `Audit`.
- Added: `Reactor`, from cerebrum's `lib/reactor/core` (package `reactor-core`) at
  `eb0d073`; `ReactorMut` and its `run`/`run_async` are not taken.
- Removed: `Engine`, `IndexUnexpected`, `Indexer` and `run`/`RunError`,
  which wonka does not use yet.
