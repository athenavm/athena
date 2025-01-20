# Athena Smart Contract Template

Welcome to the Athena smart contract template!
This guide will help you create, test, and deploy smart contracts on the Athena-enabled test network.
The template provides a minimal implementation with two essential methods: `spawn` and `verify`.
or detailed explanations of these methods, please refer to the [source code](src/contract.rs).

## Prerequisites

### 1. Install Rust

The template is written in Rust.
To get started, install the Rust toolchain by following the [official rustup guide](https://rustup.rs).

### 2. Set Up Athena Toolchain

You'll need our custom [RISC-V RV32EM toolchain](https://github.com/athenavm/rustc-rv32e-toolchain) to build contracts for Athena.
We provide a convenient `cargo athena` plugin and an `athup` helper script for installation.

To install `cargo athena`, run:

```sh
curl -L https://raw.githubusercontent.com/athenavm/athena/main/athup/athup | bash
```

Alternatively, if you've cloned this repository, run the script directly from `athup/athup`.

### 3. Connect to Athena Devnet

To deploy contracts, you'll need a go-spacemesh node connected to the Athena devnet.
Use our dedicated [go-spacemesh release](https://github.com/spacemeshos/go-spacemesh/releases/tag/athena-devnet-13-1.0.1)
along with this [configuration file](https://configs.spacemesh.network/config.devnet-athena-13.json).

> [!IMPORTANT]
> Standard go-spacemesh releases are not compatible with Athena.

## Development Guide

### Building Your Contract

Build the contract with:

```sh
cargo athena build
```

The compiled binary will be available at `elf/contract_template`.

### Testing

Run the included tests from [tests](tests/test.rs) with:

```sh
cargo test
```

> [!TIP]
> Tests run directly on your machine using the Athena VM,
> so the `athena` flag isn't needed.

### Deployment Process

#### What you will need

- A single-sig wallet with funds (1 SMH is sufficient)
- Go installation ([installation guide](https://go.dev/doc/install))
- Athena CLI tool from the [go-spacemesh repository](https://github.com/spacemeshos/go-spacemesh/tree/athena-poc/vm/cmd/client)

#### Step-by-Step Deployment

1. Clone the Athena branch:

```sh
git clone -b athena-poc https://github.com/spacemeshos/go-spacemesh.git
```

2. Navigate to `vm/cmd/client` and generate wallet keys:

```sh
go run . generateKey
```

> [!NOTE]
> The key is stored in `key.hex` file by default.

3. Get your wallet address:

```sh
go run . coinbase
```

4. Fund your wallet and verify the balance on the [explorer](https://explorer-devnet-athena.spacemesh.network/accounts)

5. Spawn your account (requires a running node with GRPC endpoint at localhost:9092):

```sh
go run . spawn --nonce 0 --address localhost:9092
```

6. Deploy your contract:

```sh
go run . deploy --nonce 1 --path <path to binary> --address localhost:9092
```

> [!NOTE]
> Each transaction requires an incremental nonce. Check the current nonce on the explorer.

## Further Development

While there isn't currently a comprehensive SDK for contract interaction,
you can reference our [wallet SDK](https://github.com/spacemeshos/go-spacemesh/blob/athena-poc/vm/sdk)
and the CLI tool for examples of creating and sending Athena transactions.

## Community Support

Join us on the #athena-vm channel in [Spacemesh Discord](https://discord.com/invite/yVhQ7rC)
for discussions, support, and sharing ideas about Athena VM development.
We're here to help you build amazing smart contracts!
