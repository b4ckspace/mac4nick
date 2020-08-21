FROM rust as builder

WORKDIR /app
ADD . .
RUN apt update && apt install -y musl-tools
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --release --target x86_64-unknown-linux-musl
RUN find /app/target -name macnickenson
RUN strip /app/target/x86_64-unknown-linux-musl/release/macnickenson

# ---

FROM scratch
WORKDIR /app/static
WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/macnickenson /app/macnickenson
USER 1000
CMD ["/app/macnickenson"]
