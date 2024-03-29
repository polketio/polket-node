# This file is part of Polket.
# Copyright (C) 2021-2022 Polket.
# SPDX-License-Identifier: GPL-3.0-or-later

# This is a base image to build substrate nodes
FROM docker.io/paritytech/ci-linux:production as builder

WORKDIR /polket-node
COPY . .

RUN rustup toolchain install nightly-2022-08-30
RUN rustup target add wasm32-unknown-unknown --toolchain nightly-2022-08-30
RUN cargo build --locked --release

# This is the 2nd stage: a very small image where we copy the binary."
FROM docker.io/library/ubuntu:20.04
LABEL description="Multistage Docker image for polket-node" \
    image.type="builder" \
    image.authors="zhiquan911@gmail.com" \
    image.vendor="Polket" \
    image.description="Multistage Docker image for polket-node" \
    image.source="https://github.com/polketio/polket-node" \
    image.documentation="https://github.com/polketio/polket-node"

# Copy the node binary.
COPY --from=builder /polket-node/target/release/polket-node /usr/local/bin

RUN mkdir .local
RUN useradd -m -u 1000 -U -s /bin/sh -d /node-dev node-dev && \
    mkdir -p /chain-data /node-dev/.local/share && \
    chown -R node-dev:node-dev /chain-data && \
    ln -s /chain-data /node-dev/.local/share/polket-node && \
    mkdir /specs && \
    chown -R node-dev:node-dev /specs && \
    ln -s /specs /node-dev/.local/share/specs && \
    # unclutter and minimize the attack surface
    rm -rf /usr/bin /usr/sbin && \
    # check if executable works in this container
    /usr/local/bin/polket-node --version

USER node-dev

# Copy specs json file.
COPY ./specs/testnet.json /specs

EXPOSE 30333 9933 9944 9615
VOLUME ["/chain-data", "/specs"]

ENTRYPOINT ["/usr/local/bin/polket-node"]