FROM rustlang/rust:nightly as builder

# install capnproto
RUN curl -O https://capnproto.org/capnproto-c++-0.10.2.tar.gz && \
    tar zxf capnproto-c++-0.10.2.tar.gz && \
    cd capnproto-c++-0.10.2 && \
    ./configure && \
    make -j6 check && \
    make install


# build
COPY . /app
WORKDIR /app
RUN cargo build --release


FROM debian:bookworm-slim
COPY --from=builder /app/target/release/tinykv /usr/local/bin/tinykv



WORKDIR /app
CMD tinykv
