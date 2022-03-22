FROM rust:alpine as builder
WORKDIR /code

COPY . .
RUN cargo b --release

# 
FROM alpine:latest
WORKDIR /code
COPY --from=builder /code/target/release/danmu2ass .
ENTRYPOINT [ "./danmu2ass" ]
CMD []
