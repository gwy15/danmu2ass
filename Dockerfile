FROM rust:slim-buster as builder
WORKDIR /code

COPY . .
RUN cargo b --release \
    && strip target/release/danmu2ass

# 
FROM debian:buster-slim
WORKDIR /code
COPY --from=builder /code/target/release/danmu2ass .
ENTRYPOINT [ "./danmu2ass" ]
CMD []
