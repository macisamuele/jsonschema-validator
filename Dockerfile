# The goal of this docker image is to provide a container to easily play with
# the project even if you don't have rust installed on your machine
#
# The docker file is not meant to be perfect or the smallest
# In the future the image could be re-customized a bit in order to simplify wheel build -> I'm not at that stage yet
#
# Example of usage
# $ docker run --rm -it -u $(id -u):$(id -g) -v $(pwd):/repository --privileged <docker-image-name> bash

FROM ubuntu:bionic
MAINTAINER "Samuele Maci <macisamuele@gmail.com>"

ENV RUST_TOOLCHAIN=stable

RUN set -eux && \
    apt-get update && \
    apt-get install -y \
        binutils-dev \
        cmake \
        cmake \
        curl \
        g++ \
        git \
        jq \
        libcurl4-openssl-dev \
        libdw-dev \
        libelf-dev \
        libiberty-dev \
        libssl-dev \
        pkg-config \
        tig \
        vim \
        zlib1g-dev && \
    rm -rf /var/lib/apt/lists/*

# Install cargo
RUN curl --silent --fail --show-error --location --retry 3 https://sh.rustup.rs | \
        sh -s -- --default-toolchain ${RUST_TOOLCHAIN} --verbose -y && \

# Install cargo-kcov
RUN set -exu && \
    ${HOME}/.cargo/bin/cargo install cargo-kcov && \
    ${HOME}/.cargo/bin/cargo kcov --print-install-kcov-sh | sh

# Install rust linters
RUN set -eux &&\
    ${HOME}/.cargo/bin/rustup component add rustfmt-preview --toolchain ${RUST_TOOLCHAIN} && \
    ${HOME}/.cargo/bin/rustup component add clippy-preview --toolchain ${RUST_TOOLCHAIN}

WORKDIR /code
ENV CARGO_HOME=/root/.cargo

CMD ['/bin/bash']
