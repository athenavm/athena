# Athena smart contract template

This template shows the bare minimum that a smart contract must currently implement
to be deployed and ran on the Athena-enabled test network.

The contract implements 2 necessary methods:

- spawn
- verify

Please check the comments in the [code](src/contract.rs) for explanations.

## Prerequisites

### Rust installed

The template is implememented in the Rust programming language and a working rust toolchain is needed.
Follow the [official rustup guide](https://rustup.rs) too install it.

### Athena toolchain

We maintain a [toolchain](https://github.com/athenavm/rustc-rv32e-toolchain) which
is needed to build the contract code for the Athena (RISC-V RV32EM).

This repository contains a `cargo athena` plugin used to build the contracts
at [cli](../../cli) as well as `athup` helper script to simplify installing it.

To install `cargo athena` download and execute the athup:

```sh
curl -L https://raw.githubusercontent.com/athenavm/athena/main/athup/athup | bash
```

or execute it directly from within this repository at `athup/athup`.

### A go-spacemesh node connected to the Athena devnet

In order to deploy the contract onto the Spacemesh Athena devnet,
one needs to have a go-spacemesh node connected to the network.
There is a special [go-spacemesh release](https://github.com/spacemeshos/go-spacemesh/releases/tag/athena-devnet-13-1.0.1)
crafted specifially for Athena.

> [!IMPORTANT]
> A regular go-spacemesh release **will not work**.

Use it together with [configuration file](https://configs.spacemesh.network/config.devnet-athena-13.json)
to connect to the network and later publish transactions to the devnet via it.

## Building the contract

To build, simply execute the below command. The built executable is placed at `elf/contract_template`.

```sh
cargo athena build
```

## Testing the contract

The template comes with some tests in [tests](tests/test.rs). They can be run with

```sh
cargo test
```

Keep in mind there is no `athena` in the command because the tests are built and executed directly on your machine. They spin up the Athena VM and execute the smart contract code inside.

## Deploying the contract onto the Athena test network

A contract can be deployed using the standard Spacemesh single-sig wallet.
It has a `deploy` method designed specifically for this purpose.
First, you will need a single-sig wallet account with some funds in it.
One SMH is more than enough.

Unfortunately, currently the Spacemesh wallet application doesn't support
deploying contracts (yet).
You will need to use the CLI tool developed specifically for the Athena test network.
It's hosted at [go-spacemesh repository](https://github.com/spacemeshos/go-spacemesh/tree/athena-poc/vm/cmd/client).

It's written in Go, so you will need a working Go installation to proceed.
Go can be installed by following the [Go official guide](https://go.dev/doc/install).
Once it is installed, checkout the code at branch `athena-poc`:

```sh
git clone -b athena-poc https://github.com/spacemeshos/go-spacemesh.git

```

Next, go to `vm/cmd/client` directory. Create the single-sig wallet keys with

```sh
go run . generateKey
```

It will create a `key.hex` containing a private key for the wallet.

Now, we need to check the address of the wallet to move some funds to it.

```sh
go run . coinbase
```

It will print the principal address of the wallet.
You need to move funds to it or earn rewards for smeshing on the network as usual.

The balance can be checked at the [explorer](https://explorer-devnet-athena.spacemesh.network/accounts).
Once the account has some funds, it can be spawned.

The tool needs to talk with some go-spacemesh node connected to
the network to publish a transaction to the network.
Spacemesh doesn't provide a publicly available API at the moment.

Follow the steps [to run athena node](#a-go-spacemesh-node-connected-to-the-athena-devnet).
The node by defualt exposes a `TransactionService` GRPC endpoint exposed at `localhost:9092`.

```sh
go run . spawn --nonce 0 --address localhost:9092
```

Finally, it can be used to **deploy** the contract.

> [!NOTE]
> The nonce cannot be reused and should be increasing with each TX.
> The current counter value can also be checked on the explorer on the account page.

```sh
go run . deploy --nonce 1 --path <path to binary> --address localhost:9092
```

From this point onward, there is no ready tool or SDK to interact with the deployed contract (e.g. call its methods).
You can check the SDK for our single- and multi- sig wallets in the [SDK for go-spacemesh wallets](https://github.com/spacemeshos/go-spacemesh/blob/athena-poc/vm/sdk)
to see how to create Athena transactions and the code in the CLI tool (vm/cmd/cli) for examples how to send a TX to a node.

## Contact

The official Athena VM channel #athena-vm on [Spacemesh Discord](https://discord.gg/spacemesh)
is the best place to ask for help and share ideas about the Athen VM, SDK, contracts and everything related.
