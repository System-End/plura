FROM rust:1.92 as builder
ARG APP_NAME=plura
WORKDIR /app

RUN apt-get update && \
    apt-get install -y clang llvm-dev libclang-dev pkg-config sqlite3 libsqlite3-dev git musl-tools

COPY . .

RUN rustup target add x86_64-unknown-linux-musl && \
    cargo build --release --target x86_64-unknown-linux-musl && \
    cp ./target/x86_64-unknown-linux-musl/release/$APP_NAME /bin/server

FROM alpine:latest AS final

ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
USER appuser

COPY --from=builder /bin/server /bin/

EXPOSE 8080

CMD ["/bin/server"]
