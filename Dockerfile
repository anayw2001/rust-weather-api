FROM rustlang/rust:nightly as builder
WORKDIR /usr/src/rust-weather-api
COPY . .
RUN apt-get update \
&& apt-get install -y ca-certificates tzdata libclang-dev protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

RUN cargo install --path .

FROM debian:bullseye-slim
#RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/rust-weather-api /usr/local/bin/rust-weather-api
CMD ["rust-weather-api"]