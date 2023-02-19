FROM rust

RUN cargo install diesel_cli --no-default-features --features postgres

WORKDIR /app
COPY . .

RUN diesel migration run
CMD ["cargo", "run", "--release"]
EXPOSE 8080