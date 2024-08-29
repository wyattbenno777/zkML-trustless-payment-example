# Initialize

`cd server` then `cargo +nightly run --bin build_public_params`

This will create a json file containing the public parameters for the proof verifications

Still in `./server` directory, create a `.env` file with those 4 args:

```
SEPOLIA_RPC_ENDPOINT=<your RPC endpoint>
SEPOLIA_USDC_CONTRACT=<The USDC contract address>
SENDER_PRIVATE_KEY=<Sender account private key>
SENDER_ADDRESS=<Sender public address>
```

The sender account needs to have enough gas to send transactions, as well as a minimum of 1 USDC (by default the server sends 1 USDC as pamyent).

# Start server

When still in server directory: `cargo +nightly run`

# Compute proof and send to server

Go to client directory ( `cd ../client` from server directory) then use `cargo +nightly run`.

This will start computing the proof for executing fib.wat file with '10' as input
