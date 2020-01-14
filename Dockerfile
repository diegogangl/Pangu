# select image
FROM rustlang/rust:nightly-stretch

# copy your source tree
COPY ./ ./

RUN apt-get install -y python3

# build for release
RUN cargo build --release

