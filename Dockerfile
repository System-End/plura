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

WORKDIR /app
RUN chown appuser:appuser /app
USER appuser

COPY --from=builder /bin/server /app/plura

EXPOSE 8080

CMD ["./plura"]
