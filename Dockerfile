FROM rust:1.76 as builder
WORKDIR /usr/src/no-nonsense-recipes
COPY . .
RUN cargo install --path .


# Final package
FROM alpine:3.14

COPY --from=builder /usr/local/cargo/bin/no-nonsense-recipes /usr/local/bin/no-nonsense-recipes
CMD ["no-nonsense-recipes"]
