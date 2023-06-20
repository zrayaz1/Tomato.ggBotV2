FROM rust:latest

WORKDIR /usr/src/tomatobot

COPY . .

RUN cargo build

CMD cargo run



