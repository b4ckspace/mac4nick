FROM rust as builder

WORKDIR /app
ADD . .
RUN apt update && apt install -y musl-tools
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --release --target x86_64-unknown-linux-musl
RUN find /app/target -name mac4nick
RUN strip /app/target/x86_64-unknown-linux-musl/release/mac4nick

# ---

FROM scratch
WORKDIR /app/static
WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/mac4nick /app/mac4nick
USER 1000
CMD ["/app/mac4nick"]
