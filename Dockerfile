# syntax=docker/dockerfile:1.3

FROM ekidd/rust-musl-builder:1.57.0 AS builder

ENV USER=visonic
ENV UID=10001

RUN sudo adduser \
    --disabled-password \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

WORKDIR /home/rust/src

COPY ./ .

RUN cargo build --release
RUN strip ./target/x86_64-unknown-linux-musl/release/visonic

####################################################################################################
## Final image
####################################################################################################
FROM scratch

COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /visonic

COPY --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/visonic ./

USER visonic:visonic
CMD ["/visonic/visonic"]