# Initialize

`cd server` then `cargo +nightly run --bin build_public_params`

This will create a json file containing the public parameters for the proof verifications

# Start server

When still in server directory: `cargo +nightly run`

# Compute proof and send to server

Go to client directory ( `cd ../client` from server directory) then use `cargo +nightly run`.

This will start computing the proof for executing fib.wat file with '10' as input

# Error

Currently seems like I cannot deserialize the public_params json file into a BatchedPublicValues struct in server main.rs
