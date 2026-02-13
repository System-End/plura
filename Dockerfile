FROM rust:1.92-alpine as builder
ARG APP_NAME
WORKDIR /app

RUN apk add --no-cache clang lld musl-dev git llvm-dev libc-dev

COPY . .

RUN cargo build --release --target x86_64-unknown-linux-musl && \
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
