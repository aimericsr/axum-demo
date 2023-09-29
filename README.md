# AwesomeApp rust-web-app


## Web API to demonstrate axum-web capabilities


The observability architecture is based on the official [exemple](https://opentelemetry.io/docs/demo/architecture/) of the opentelemetry website.


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
- Visualize data with grafana
- Graceful Shutdown for sending last traces

TO Do:
- add more filter to tracing
- add more unit test / do integration test
- load env variable only at the beginning
- version the api (path or url ?)
- handle db connection retry system
- handle request body validation

## License

This project is licensed under the [Apache License](LICENSE).

## Rust : 

brew install openssl@1.1
cargo install cargo-edit
cargo install cargo-expand

cargo install --version=0.7.2 sqlx-cli --no-default-features --features postgres 

docker pull grafana/k6
brew install k6

curl -fsSL https://bun.sh/install | bash