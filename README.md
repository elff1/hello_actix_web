# Hello Actix Web

A simple web application using the Actix Web framework in Rust.

## Prepare

### Install Tools

Make sure you have Rust and Cargo installed. You can install them from [rustup.rs](https://rustup.rs/).

PostgreSQL should be installed (only its client `psql` is needed). You can download it from [postgresql.org](https://www.postgresql.org/download/).

Also, install the SQLx CLI tool with PostgreSQL support:

`cargo install --version=0.8.6 sqlx-cli --no-default-features --features postgres`

### DB Setup

`./scripts/init_db.sh`

To skip Docker setup:

`SKIP_DOCKER=true ./scripts/init_db.sh`

## Run

`cargo run`

Set the `RUST_LOG` environment variable to see detailed logs, default is `info`:

`RUST_LOG=trace cargo run`

Structured logs can be viewed in a more readable format using `bunyan`:

`cargo run | bunyan`

Set the `APP_ENVIRONMENT` environment variable to `production` to run in production mode, default is `local`:

`APP_ENVIRONMENT=production cargo run`

## Test

`cargo test`

Set `TEST_LOG` environment variable to see logs during tests, and the default log level is `debug` which also can be overridden by `RUST_LOG`:

`TEST_LOG=true RUST_LOG="sqlx=error,info" cargo test | bunyan`

## Docker

### Build Docker Image

`docker build --tag hello_actix_web:latest --file Dockerfile .`

### Run Docker Container

`docker run -p 80xx:8000 hello_actix_web:latest`
