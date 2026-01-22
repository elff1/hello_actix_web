FROM rust:trixie AS builder

WORKDIR /app

#RUN printf '[source.crates-io]\nreplace-with = "mirror"\n\n[source.mirror]\nregistry = "https://mirrors.tuna.tsinghua.edu.cn/git/crates.io-index.git"\n' > $CARGO_HOME/config.toml
RUN printf 'Types: deb\nURIs: https://mirrors.tuna.tsinghua.edu.cn/debian\nSuites: trixie trixie-updates trixie-backports\nComponents: main contrib non-free non-free-firmware\nSigned-By: /usr/share/keyrings/debian-archive-keyring.gpg\n\nTypes: deb\nURIs: https://security.debian.org/debian-security\nSuites: trixie-security\nComponents: main contrib non-free non-free-firmware\nSigned-By: /usr/share/keyrings/debian-archive-keyring.gpg\n' > /etc/apt/sources.list.d/debian.sources

RUN apt-get update && apt-get install -y \
    lld \
    clang

COPY Cargo.toml .
COPY Cargo.lock .
RUN sed -i 's#src/main.rs#dummy.rs#' Cargo.toml
RUN sed -i 's#src/lib.rs#dummy_lib.rs#' Cargo.toml
RUN echo "fn main() {}" > dummy.rs
RUN echo "pub struct DummyStruct;" > dummy_lib.rs
RUN cargo build --release
RUN sed -i 's#dummy_lib.rs#src/lib.rs#' Cargo.toml
RUN sed -i 's#dummy.rs#src/main.rs#' Cargo.toml

COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release

FROM debian:trixie-slim AS runtime

WORKDIR /app

COPY --from=builder /app/target/release/hello_actix_web .
COPY configuration configuration
ENV APP_ENVIRONMENT=production

ENTRYPOINT ["./hello_actix_web"]
