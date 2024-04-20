FROM rust:1.76 as builder
WORKDIR /usr/src/no-nonsense-recipes
COPY . .
RUN cargo install --path .

FROM debian:bookworm-slim
RUN apt-get update \
	&& apt-get install -y --no-install-recommends \
		curl \
		ca-certificates \
		gcc \
		libc6-dev \
		pkg-config \
		libssl-dev \
	&& rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/no-nonsense-recipes /usr/local/bin/no-nonsense-recipes
CMD ["no-nonsense-recipes"]

