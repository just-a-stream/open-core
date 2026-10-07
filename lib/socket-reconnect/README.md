# socket-reconnect

Turn an async connect function into a `Stream` of sockets that reconnects on
failure, with pluggable backoff and error handling.

- `init_reconnecting_socket(connect, timeout_connect, backoff)` yields one
  connection result per attempt, sleeping for the `ReconnectBackoff` between
  attempts and timing out slow connects.
- The `ReconnectingSocket` extension trait composes the policy:
  - `on_connect_err` decides whether a failed connect is retried or ends the stream;
  - `on_stream_err` and `on_stream_err_filter` decide whether an error from a live
    socket passes through or triggers a reconnect;
  - `with_socket_updates` flattens the sockets into `SocketUpdate::{Connected,
    Item, Reconnecting}` events, handing over each new sink as it connects.
- Backoff and error handlers are single-method traits with blanket impls for
  closures, so a plain closure or a named type works at every call site.

Library code stays lazy: every stage takes and returns a `Stream`, and the
caller chooses how to drive it.

The optional `serde` feature derives `Serialize` and `Deserialize` for
`SocketUpdate`.
