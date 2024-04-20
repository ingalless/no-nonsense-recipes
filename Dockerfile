FROM rust:1.76 as builder
WORKDIR /usr/src/no-nonsense-recipes
COPY . .
RUN cargo install --path .

FROM debian:bookworm-slim

ENV APP_RECIPES_PATH=/home/recipes
ENV APP_COMPILED_PATH=/home/compiled

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
RUN mkdir /home/recipes /home/compiled
CMD ["no-nonsense-recipes"]

