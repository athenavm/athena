# FFI bindings for the Athena VM

## `athcon`

The `athcon` directory contains the application-side bindings for calling Athena VM over FFI.

## `vmlib`

The `vmlib` directory contains the library-side bindings that allow the Athena VM to be called from the FFI.

## Tracing

The Athena VM is instrumented with [tracing](https://tracing.rs) for emitting structured events (logs).
The `vmlib` automatically configures a [subscriber](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/) that writes the events to stdout.
To enable the logs when using the Athena library, set the `RUST_LOG` environment variable to the desired log level, for example: `RUST_LOG=debug`.
See [example syntax](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#example-syntax) for more sophisticated examples.

## Tests

To see trace output for tests, make sure the test in question calls `setup_logger()`, then run the test as follows:

```
> RUST_LOG=<log level> cargo test <desired test or package> -- --nocapture
```
