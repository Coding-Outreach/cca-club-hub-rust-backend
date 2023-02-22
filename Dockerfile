FROM rust:buster

RUN cargo install diesel_cli --no-default-features --features postgres

WORKDIR /app
COPY . .

CMD ["cargo", "run", "--release"]
EXPOSE 8080