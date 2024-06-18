# Athena

Athena is a prototype deterministic smart contract engine that serves as the [Spacemesh network VM][2], and Athena is being designed and built by the  [Spacemesh][8] team. However, Athena is designed to be modular and largely protocol-agnostic so [it will run on other chains][9]. Contributions and integrations are welcome.

Athena includes a virtual machine (VM) based on the RISC-V ISA, including support for [RV32IM][10] and [RV32EM][1], an interpreter/compiler for running smart contract code, and related tooling. The VM is modular and features a mature FFI that can be integrated into any language that supports CFFI. For more information on the Athena project and its goals see [Introducing Athena][2] and the [Athena project updates][3].

## Project Goals
- **Developer Experience**: Provide a robust environment with extensive tooling support.
- **Performance**: Ensure fast execution of smart contracts. (Note that this is a long-term goal and is not a goal at this prototype phase.)
- **Small On-Chain Footprint**: Deployed Athena code should be as compact as possible.
- **Security**: Implement strong isolation and safety measures.
- **ZK Provability**: Ensure provability in zero-knowledge (ZK) environments; aim to natively target RISC-V-based ZK circuits.
- **Mainline Rust Support**: Allow writing Athena code using ordinary Rust and compiling with the standard Rust/LLVM pipeline. Aim to provide as much stdlib support as possible.

## Non-Goals
- **Floating Point Support**: No support for floating-point operations.
- **SIMD**: No support for Single Instruction, Multiple Data operations.
- **Additional RISC-V Extensions**: No RISC-V extensions beyond the M extension.
- **Interoperability**: At this time Athena is not intended to interoperate with, or target, other blockchain VMs.
- **Full System Emulation**: Athena is not intended to run an operating system or standard application binaries. Try https://github.com/d0iasm/rvemu.

## Acknowledgements
The overall project structure and a large portion of the core code was copied from [SP1][4] under the MIT license with gratitude. Other projects that have been influential on the Athena design include [RiscZero][5] and [PolkaVM][6].

## License
This project is dual-licensed under both the Apache and MIT Licenses, at your option. Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions. See [Rationale of Apache dual licensing][7].

## Disclaimer
**Warning**: This code is not production quality and should not be used in production systems.

[1]: https://five-embeddev.com/riscv-user-isa-manual/Priv-v1.12/rv32e.html
[2]: https://spacemesh.io/blog/introducing-athena/
[3]: https://athenavm.github.io/
[4]: https://github.com/succinctlabs/sp1/
[5]: https://github.com/risc0/risc0/
[6]: https://github.com/koute/polkavm
[7]: https://internals.rust-lang.org/t/rationale-of-apache-dual-licensing/
[8]: https://spacemesh.io/
[9]: https://www.athenavm.org/athena/update/2024/06/14/june-project-update.html#ecosystem
[10]: https://five-embeddev.com/riscv-user-isa-manual/Priv-v1.12/rv32.html#rv32
