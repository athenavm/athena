# Introduction

Athena is a general-purpose VM that is designed to run smart contracts for the Spacemesh blockchain protocol. While
Athena is the Spacemesh VM, it's modular and chain agnostic and is designed to work in many places, at L1 and L2, in
many chains.

At its core is an interpreter that targets the RISC-V instruction set. The ultimate aim behind Athena is to build a
modern, secure, and performant RISC-V blockchain VM that natively targets ZK execution and proving. In order to achieve
this, the future plan is to prove Athena program execution using RISC-V zkVMs such as
[risc0](https://github.com/risc0/risc0/) and [sp1](https://github.com/succinctlabs/sp1/).

Athena supports mainline Rust as its smart contract programming language. Rust is the perfect candidate for writing
blockchain programs due to its mature ecosystem, high performance, built-in safety features, and powerful and mature
LLVM toolchain. Athena supports both the RV32IM and RV32EM variants of RISC-V, but for now all Athena programs target
RV32EM. We chose the "embedded" variant, with 16 rather than 32 registers, in order to facilitate developing a "runtime
kernel" that will wrap Athena programs when they're run in the zkVMs mentioned above.

Since today Rust doesn't yet fully support RV32EM, we had to create and maintain
[our own Rust toolchain](https://github.com/athenavm/rustc-rv32e-toolchain/tree/main) with support for this target. For
now, Athena users need to download this toolchain to be able to compile Athena code, but this process is fully automated
by the CLI tools. It's our goal to remove this requirement in the not-too-distant future, realizing our vision that
"Athena is just Rust."

With this, let us start writing programs for Athena!
