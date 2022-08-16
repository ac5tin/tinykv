FROM rustlang/rust:nightly as builder

# build
COPY . /app
WORKDIR /app
RUN cargo build --release


FROM debian:bookworm-slim
COPY --from=builder /app/target/release/tinykv /usr/local/bin/tinykv
RUN apt update -y
WORKDIR /app
CMD tinykv
