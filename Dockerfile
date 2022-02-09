# syntax=docker/dockerfile:1.3

FROM ekidd/rust-musl-builder:1.57.0 AS builder

ENV USER=visonic
ENV UID=10001

RUN sudo apt update && sudo apt -y install binutils-arm-linux-gnueabihf gcc-arm-linux-gnueabihf musl-tools
RUN sudo ln -s /usr/bin/arm-linux-gnueabihf-gcc /usr/bin/arm-linux-musleabihf-gcc

RUN sudo adduser \
    --disabled-password \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

WORKDIR /home/rust/src

COPY ./ .

RUN cargo install cargo-strip

RUN sudo -E bash -c '/opt/rust/cargo/bin/rustup target add armv7-unknown-linux-musleabihf'
RUN cargo build --target=armv7-unknown-linux-musleabihf --release
RUN cargo strip --target=armv7-unknown-linux-musleabihf

RUN cargo build --release
RUN cargo strip

####################################################################################################
## Final image
####################################################################################################
FROM scratch

COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /visonic

COPY --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/visonic ./
COPY --from=builder /home/rust/src/target/armv7-unknown-linux-musleabihf/release/visonic ./visonic-arm
COPY --from=builder /etc/ssl /etc/ssl

USER visonic:visonic
CMD ["/visonic/visonic"]