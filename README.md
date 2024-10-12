# Initialize

`cd server` then `RUST_LOG=debug cargo +nightly run --bin build_public_params`

This will create a json file containing the public parameters corresponding to the executable that the client wants to provably run, used for the proof verification

Still in `./server` directory, create a `.env` file with those 4 args:

```
SEPOLIA_RPC_ENDPOINT=<your RPC endpoint>
SEPOLIA_USDC_CONTRACT=<The USDC contract address>
SENDER_PRIVATE_KEY=<Sender account private key>
SENDER_ADDRESS=<Sender public address>
```

The sender account needs to have enough gas to send transactions, as well as a minimum of 1 USDC (by default the server sends 1 USDC as pamyent).

This value can be changed by modifying the `SEND_AMOUNT` const in `server/src/main.rs`.

# Start server

When still in server directory: `RUST_LOG=debug cargo +nightly run`

# Compute proof and send to server

Go to client directory ( `cd ../client` from server directory) then use `RUST_LOG=debug cargo +nightly run`.

This will start computing the proof for executing `gradient_boosting.wasm`, with a limit to 10 000 opcodes so that it runs faster.

The public params building, proving and verifying processes still takes a few minutes, be sure to set `RUST_LOG=debug` for a more verbose execution.
