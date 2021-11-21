FROM rust:1-bullseye

WORKDIR /usr/src/discord-c4
COPY . .

RUN apt-get update && apt-get install openssl

RUN RUST_BACKTRACE=1 cargo install --path .

CMD ["discord-c4"]
