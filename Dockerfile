FROM ubuntu:20.04 as build

RUN apt-get update
RUN apt-get install --no-install-recommends -y \
    ca-certificates curl build-essential pkg-config libssl-dev libpq-dev libudev-dev

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH=/root/.cargo/bin:$PATH

RUN mkdir -p /usr/src/app
WORKDIR /usr/src/app

COPY . /usr/src/app

RUN cargo build --release --bin auger-server

FROM ubuntu:20.04 AS run

RUN apt-get update
RUN apt-get install --no-install-recommends -y curl ca-certificates libssl-dev libpq-dev libudev-dev && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

COPY --from=build /usr/src/app/target/release/auger-server /usr/local/bin/auger-server

ENV RUST_LOG=info
ENV API_KEYS=your-api-key-here

EXPOSE 8180

ENTRYPOINT ["/usr/local/bin/auger-server"]