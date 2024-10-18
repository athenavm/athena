# VM SDK

## Debugging guest programs

The VM integrates a GDB stub using [gdbstub](https://docs.rs/crate/gdbstub/latest).
It allows to attach with GDB and debug a guest program running in the VM.

💡 The GDB support is experimental and some things might not work.

### Required instrumentation
The host program (which executes the guest program) must be instrumented to enable GDB debugging.
See the documentation of `ExecutionClient` for details.

Note: It's important that the guest program contains debugging information. This can be achieved by building it in `debug` profile or by adding to its `Cargo.toml`:

```toml
[profile.release]
debug=true
```

### Connecting with GDB

Note that you'll need a version of `gdb` with RISC-V support. The version of `gdb` included with most Linux distributions doesn't support RISC-V out of the box. You can obtain it as part of the [riscv-gnu-toolchain](https://github.com/riscv-collab/riscv-gnu-toolchain).

Once the instrumented program is executed, the GDBstub will halt it on the first instruction and await connection from a GDB client. Assuming, it listens on port 9001, execute the following:

```sh
> gdb
(gdb) file <path to guest program elf built with symbols>
(gdb) target remote localhost:9001
```

### Connecting with VsCode

First, install an extension with GDB support, e.g. [Native Debug](marketplace.visualstudio.com/items?itemName=webfreak.debug). Next, add the following to `launch.json` file:

```json
{
    "type": "gdb",
    "request": "attach",
    "name": "Attach to guest program",
    "executable": "<path to guest program elf>",
    "target": ":9001",
    "remote": true,
    "cwd": "${workspaceRoot}",
    "valuesFormatting": "parseText",
    "stopAtConnect": true
}
```

and start debugging normally.
