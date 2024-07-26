# Athena

Athena is a prototype deterministic smart contract engine that serves as the [Spacemesh network VM][2], and Athena is being designed and built by the  [Spacemesh][8] team. However, Athena is designed to be modular and largely protocol-agnostic so [it will run on other chains][9]. Contributions and integrations are welcome.

Athena includes a virtual machine (VM) based on the RISC-V ISA, including support for [RV32IM][10] and [RV32EM][1], an interpreter/compiler for running smart contract code, and related tooling. The VM is modular and features a mature FFI that can be integrated into any language that supports CFFI. For more information on the Athena project and its goals see [Introducing Athena][2] and the [Athena project updates][3].

## Project Goals
- **Developer Experience**: Provide a robust environment with extensive tooling support.
- **Simplicity**: VMs tend to be extraordinarily complex. The Athena codebase is clean and contains an order of magnitude less code than most VMs.
- **Modularity**: Athena is totally self-contained and agnostic to the environment where it runs. It has a well-defined Host<>VM CFFI API and ABI that can be used to swap hosts or VMs, and Athena can talk to a host written in Go or another compatible language.
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

## Progress

Athena is currently in a prototype stage. The goal of this stage of the project is to create a working, end to end proof of concept VM that successfully and securely executes transactions on a testnet. We also intend to test proving these transactions in ZK. Here's a map of the immediate goals and progress. See the [Project boards][12] for more up to date progress.

| Phase | Description | Status | Report |
| ----- | ----------- | ------ | ------ |
| 0. Initial R&D | Study the status quo, finalize prototype design | ✅ | [read][13] |
| 1. Prototype VM | Build a VM that can compile and run RISC-V code | ✅ | [read][14] |
| 2. Blockchain integration | Add FFI and support for host functions, gas metering, etc. | ✅ | [read][15] |
| 3. go-spacemesh integration | Prototype integration into the go-spacemesh full node | 🚧 | |
| 4. Testnet launch | Launch a testnet where Athena smart contracts can be tested | ⛔ | |
| 5. Mechanism/rollup design | Turn Athena into an optimistic rollup with incentives, punishments, etc. | ⛔ | |
| 6. Succintness/ZK proving | Prototype ZK rollup | ⛔ | |

## Acknowledgements
The overall project structure and a large portion of the core code was copied from [SP1][4] under the MIT license with gratitude. Other projects that have been influential on the Athena design include [RiscZero][5] and [PolkaVM][6]. See [ATTRIBUTIONS.md][11] for others.

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
[11]: ATTRIBUTIONS.md
[12]: https://github.com/athenavm/athena/projects?query=is%3Aopen
[13]: https://www.athenavm.org/athena/update/2024/05/09/project-update.html
[14]: https://www.athenavm.org/athena/update/2024/06/14/june-project-update.html
[15]: https://www.athenavm.org/athena/update/2024/07/20/july-project-update.html
