# AwesomeApp rust-web-app

## Starting the DB

```sh
# Start postgresql server docker image:
docker run -d --rm --name pg -p 5432:5432  -e POSTGRES_PASSWORD=welcome postgres:15

# (optional) To have a psql terminal on pg.
# In another terminal (tab) run psql:
docker exec -it -u postgres pg psql

# (optional) For pg to print all sql statements.
# In psql command line started above.
ALTER DATABASE postgres SET log_statement = 'all';
```

## Dev (REPL)

> NOTE: Install cargo watch with `cargo install cargo-watch`.

```sh
# Terminal 1 - To run the server.
cargo watch -q -c -w src/ -w .cargo/ -x "run"

# Terminal 2 - To run the quick_dev.
cargo watch -q -c -w examples/ -x "run --example quick_dev"
```

## Unit Test (REPL)

```sh
cargo watch -q -c -x "test -- --nocapture"

# Specific test with filter.
cargo watch -q -c -x "test model::task::tests::test_create -- --nocapture"
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

## Manage Rust Versions

```sh
rustup help toolchain

rustup install nightly

rustup default nightly-aarch64-apple-darwin

rustup update

rustc --version

```

## Project Conventions

function test name : test*[function_name]*[ok/err]\_[case_tested]

## Observabilty

Prometheuse : docker run -d \
 -p 9090:9090 \
 -v /$(pwd)/config/prometheus.yml:/etc/prometheus/prometheus.yml \
 prom/prometheus

Jaeger :
docker run -d -p6831:6831/udp -p6832:6832/udp -p16686:16686 -p14268:14268 jaegertracing/all-in-one:1.6

Features : 
Timout
CORS
Tracing / metrics export to jeager / prometheus
Helth check routes
OpenAPI docs
Rest Routes
RPC Routes
Auth with cookies and jwt
Serve static file 
Graceful Shutdown for sending last traces

TO Do:
add more filter to tracing
add more test
