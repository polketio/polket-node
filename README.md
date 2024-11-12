![w3f-grants](./docs/images/w3f-grants.svg)

# Polket Node

The goal of Polket is to create more commercial application scenarios for NFTs and connect to the Polkadot/Kusama network in a parachain manner. Based on the Polket chain, we will develop a smart fitness-type Web3 application. We will name it `ToEarnFun`.

## Getting Started

Follow the steps below to get started with the Node Template, or get it up and running right from
your browser in just a few clicks using [Playground](https://playground.substrate.dev/)
:hammer_and_wrench:

### Using Nix

Install [nix](https://nixos.org/) and optionally [direnv](https://github.com/direnv/direnv) and
[lorri](https://github.com/target/lorri) for a fully plug and play experience for setting up the
development environment. To get all the correct dependencies activate direnv `direnv allow` and
lorri `lorri shell`.

### Rust Setup

First, complete the [basic Rust setup instructions](./doc/rust-setup.md).

### Build

```sh
make build
```

> If this error message appears when compiling the source code:
>
>  Rust WASM toolchain not installed, please install it!

```sh

# Plesae install toolchain nightly
rustup target install wasm32-unknown-unknown --toolchain nightly-2022-08-30
rustup target add wasm32-unknown-unknown --toolchain nightly-2022-08-30

```

### Single-Node Development Chain

This command will start the single-node development chain with persistent state:

```sh
make run
```

Purge the development chain's state:

```bash
make purge-chain
```

Start the development chain with detailed logging:

```bash
RUST_BACKTRACE=1 ./target/release/polket-node -ldebug --dev
```

### Testing

```bash
make test
```

### Connect with Polkadot-JS Apps Front-end

Once the node template is running locally, you can connect it with **Polkadot-JS Apps** front-end
to interact with your chain. [Click
here](https://polkadot.js.org/apps/#/explorer?rpc=ws://localhost:9944) connecting the Apps to your
local node template.

### Multi-Node Local Testnet

If you want to see the multi-node consensus algorithm in action, refer to our
[Start a Private Network tutorial](https://substrate.dev/docs/en/tutorials/start-a-private-network/).

### Run in Docker

First, install [Docker](https://docs.docker.com/get-docker/) and
[Docker Compose](https://docs.docker.com/compose/install/).

Then run the following command to start a single node development chain.

```bash
./scripts/docker_run.sh
```

This command will firstly compile your code, and then start a local development network. You can
also replace the default command
(`cargo build --release && ./target/release/polket-node --dev --ws-external`)
by appending your own. A few useful ones are as follow.

```bash
# Run Substrate node without re-compiling
./scripts/docker_run.sh ./target/release/polket-node --dev --ws-external

# Purge the local dev chain
./scripts/docker_run.sh ./target/release/polket-node purge-chain --dev

# Check whether the code is compilable
./scripts/docker_run.sh cargo check
```
### Run Full Node on testnet

```bash

# pull latest docker image
docker pull polketio/polket-node:latest

mkdir -p ~/chain-data
chown 1000.1000 ~/chain-data -R

docker run -d \
-v ~/chain-data:/chain-data \
-p 9944:9944 \
-p 9933:9933 \
-p 30333:30333 \
polketio/polket-node:latest \
  --base-path "/chain-data" \
  --chain "/specs/testnet.json" \
  --pruning archive \
  --port 30333 \
  --ws-port 9944 \
  --rpc-port 9933 \
  --ws-external \
  --rpc-cors all \
  --rpc-external \
  --name your-fullnode-name

```

## License

[GPL-v3.0](./LICENSE)