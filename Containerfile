FROM rust:latest
RUN rustup target add x86_64-unknown-linux-musl
RUN apt-get update && apt-get install -y musl-tools musl musl-dev libssl-dev
WORKDIR /app
COPY . .

CMD ["cargo", "build", "--release",  "--target", "x86_64-unknown-linux-musl"]
