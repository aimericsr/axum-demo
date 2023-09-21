# AwesomeApp rust-web-app

## Starting the DB

```sh
# Start the needed services
docker compose up -d

## Dev (REPL)
```sh
# Terminal 1 - To run the server.
cargo watch -q -c -w src/ -w .cargo/ -x "run"

# Terminal 2 - To run the quick_dev.
cargo watch -q -c -w examples/ -x "run --example quick_dev"
```


## Dev

```sh
# Terminal 1 - To run the server.
cargo run

# Terminal 2 - To run the tests.
cargo run --example quick_dev
```

## Unit Test

```sh
cargo test -- --nocapture

cargo watch -q -c -x test model::task::tests::test_create -- --nocapture
```

## Manage Different Rust Versions

```sh
rustup help toolchain

rustup install nightly

rustup default nightly-aarch64-apple-darwin

rustup update

rustc --version

```

## Project Conventions

function test name : test*[function_name]*[ok/err]\_[case_tested]

## Features
- Timout
- CORS
- Serve static file 
- Helth check routes
- Rest Routes
- RPC Routes
- OpenAPI docs
- Auth with cookies and jwt
- Tracing / metrics export to jeager / - prometheus
- Graceful Shutdown for sending last traces

TO Do:
- add more filter to tracing
- add more unit test / do integration test
- load env variable only at the beginning
- version the api (path or url ?)

## License

This project is licensed under the [Apache License](LICENSE).