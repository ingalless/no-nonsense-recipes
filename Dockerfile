FROM rust:1.76 as builder
WORKDIR /usr/src/no-nonsense-recipes
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y build-essential && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/no-nonsense-recipes /usr/local/bin/no-nonsense-recipes
CMD ["no-nonsense-recipes"]

