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

## Test

`cargo test`

## Docker

### Build Docker Image

`docker build --tag hello_actix_web:latest --file Dockerfile .`

### Run Docker Container

`docker run -p 80xx:8000 hello_actix_web:latest`
