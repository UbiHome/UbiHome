FROM rust:1-trixie AS build
RUN apt-get update && apt-get install -y libdbus-1-dev pkg-config libasound2-dev libpipewire-0.3-dev libclang-dev
WORKDIR /src
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release && cp target/release/ubihome /usr/local/bin/ubihome

FROM ghcr.io/charmbracelet/vhs
RUN apt-get update && apt-get install -y nano libdbus-1-3 libasound2t64 libpipewire-0.3-0t64
COPY --from=build /usr/local/bin/ubihome /usr/local/bin/ubihome
