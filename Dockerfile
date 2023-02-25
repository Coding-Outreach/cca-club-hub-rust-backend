FROM rust as build

RUN USER=root cargo new --bin cca_club_hub
WORKDIR /cca_club_hub

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src

RUN rm ./target/release/deps/cca_club_hub*
RUN cargo build --release

FROM debian:11.3-slim

RUN set -eux; \
    export DEBIAN_FRONTEND=noninteractive; \
	apt update; \
    apt install --yes --no-install-recommends libpq-dev

COPY --from=build /cca_club_hub/target/release/cca_club_hub .

EXPOSE 8080

CMD ["./cca_club_hub"]