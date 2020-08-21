FROM rust as builder

WORKDIR /app
ADD . .
RUN apt update && apt install -y musl-tools
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --release --target x86_64-unknown-linux-musl
RUN strip /app/target/release/macnickenson

# ---

FROM debian:buster-slim
COPY --from=builder /app/target/release/macnickenson /opt/macnickenson
CMD /opt/macnickenson
