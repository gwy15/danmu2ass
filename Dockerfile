FROM rust:slim-bullseye as builder
WORKDIR /code

COPY . .
RUN cargo b --release --no-default-features --features quick_xml,rustls \
    && strip target/release/danmu2ass

# 
FROM debian:bullseye-slim
WORKDIR /code
COPY --from=builder /code/target/release/danmu2ass .
ENTRYPOINT [ "./danmu2ass" ]
CMD []
