FROM rust:1.76 as builder
WORKDIR /usr/src/no-nonsense-recipes
COPY . .
RUN cargo install --path .


# Final package
FROM alpine:3.14

USER 0
COPY --from=builder /usr/local/cargo/bin/no-nonsense-recipes /usr/local/bin/no-nonsense-recipes
RUN chmod +x /usr/local/bin/no-nonsense-recipes
CMD ["/usr/local/bin/no-nonsense-recipes"]
