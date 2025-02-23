# Sponsored Transactions for Polkadot SDK

Small proof-of-concept for sponsored transactions in Polkadot SDK. Similar to how the `sudo` key is set in a Substrate chain, you can set a `payer` key using the Spontra pallet. You can then sponsor specific transactions using the `sponsor_call` extrinsic. Once a call is marked as sponsored, the payer pays for that call.

Note: This solution is **not analyzed for security best practices**. Don't use it in production without a thorough audit.

## Setup and Run

### Build

Use the following command to build the node without launching it:

```sh
cargo build --release
```

### Local Development Chain

The following command starts a single-node development chain that doesn't
persist state:

```sh
./target/release/spontra-node --dev
```

To purge the development chain's state, run the following command:

```sh
./target/release/spontra-node purge-chain --dev
```

To start the development chain with detailed logging, run the following command:

```sh
RUST_BACKTRACE=1 ./target/release/spontra-node -ldebug --dev
```

Development chains:

- Maintain state in a `tmp` folder while the node is running.
- Use the **Alice** and **Bob** accounts as default validator authorities.
- Use the **Alice** account as the default `sudo` account.
- Are preconfigured with a genesis state (`/node/src/chain_spec.rs`) that
  includes several pre-funded development accounts.

To persist chain state between runs, specify a base path by running a command
similar to the following:

```sh
// Create a folder to use as the db base path
$ mkdir my-chain-state

// Use of that folder to store the chain state
$ ./target/release/spontra-node --dev --base-path ./my-chain-state/

// Check the folder structure created inside the base path after running the chain
$ ls ./my-chain-state
chains
$ ls ./my-chain-state/chains/
dev
$ ls ./my-chain-state/chains/dev
db keystore network
```

### Test Data

The Sponsor Call extrinsic takes pallet name and call name as hex. Use the following test data:

Template: `0x54656d706c617465`

do_something: `0x646f5f736f6d657468696e67`

Balances: `0x42616c616e636573`

transfer_keep_alive: `0x7472616e736665725f6b6565705f616c697665`

## License

The code within this repository is licensed under Apache-2.0 license. See the [LICENSE](./LICENSE) file for more
details.