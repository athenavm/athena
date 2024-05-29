# Athena

Athena is a prototype deterministic smart contract engine. It includes a virtual machine (VM) based on the [RV32EM][1] (RISC-V) ISA, an interpreter/compiler for running smart contract code, and related tooling. For more information on the Athena project and its goals see [Introducing Athena][2] and the [Athena project updates][3].

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

## License
This project is licensed under the MIT License. Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work shall be identically licensed, without any additional terms or conditions.

## Disclaimer
**Warning**: This code is not production quality and should not be used in production systems.

[1]: https://five-embeddev.com/riscv-user-isa-manual/Priv-v1.12/rv32e.html
[2]: https://spacemesh.io/blog/introducing-athena/
[3]: https://athenavm.github.io/
