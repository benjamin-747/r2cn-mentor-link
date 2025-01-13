FROM rust:1.83.0-bookworm AS builder

WORKDIR /opt/r2cn


RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates 

# copy the source code, the context must be the root of the project
COPY . .

# build
RUN cargo build --release;


# final image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libssl-dev ca-certificates

COPY --from=builder /opt/r2cn/target/release/api /usr/local/bin/api

RUN chmod +x /usr/local/bin/api

VOLUME /opt/r2cn

CMD ["bash", "-c", "exec /usr/local/bin/api"]
