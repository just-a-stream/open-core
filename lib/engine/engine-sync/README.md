# engine-sync

Contracts and types for a deterministic, synchronous event engine. The crate
knows no events, states, reactions or transport of its own, and it has no
async dependency.

- `Update` folds one input into a state and returns its audit. `Reactor`
  reads the input, the state and that audit, and returns one reaction. It
  never writes the state.
- `Sequencer` gives each input its `Sequence`. `StateManager` checks each
  `Sequenced` input with a `SequenceValidator` and refuses a gap, a repeat or
  an exhausted sequence before the state changes.
- `Engine` is a `StateManager` plus a `Reactor` strategy. It turns each
  sequenced input into a sequenced `Output { input, audit, reaction }`.
- `Fanout` sends each output on and picks no transport. A failed send
  returns the unsent work in `FanoutFailure`.
- `EngineSnapshot` stores the state with its `EngineVersion`, and restores it
  only into the same version.
