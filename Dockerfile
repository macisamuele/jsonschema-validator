# The goal of this docker image is to provide a container to easily play with
# the project even if you don't have rust installed on your machine
#
# The docker file is not meant to be perfect or the smallest
# In the future the image could be re-customized a bit in order to simplify wheel build -> I'm not at that stage yet
#
# Example of usage
# $ docker run --rm -it -u $(id -u):$(id -g) -v $(pwd):/code --privileged <docker-image-name> bash

FROM ubuntu:bionic
MAINTAINER "Samuele Maci <macisamuele@gmail.com>"

RUN set -eux && \
    apt-get update && \
    apt-get install -y \
        binutils-dev \
        cmake \
        curl \
        entr \
        g++ \
        git \
        jq \
        libcurl4-openssl-dev \
        libdw-dev \
        libelf-dev \
        libiberty-dev \
        libssl-dev \
        pkg-config \
        python \
        tig \
        vim \
        zlib1g-dev && \
    rm -rf /var/lib/apt/lists/*

ENV RUST_TOOLCHAIN=nightly
ENV TRAVIS_RUST_VERSION=${RUST_TOOLCHAIN} \
    TRAVIS_OS_NAME=docker \
    TRAVIS_BUILD_DIR=/code

# Install cargo
RUN set -x && \
    curl --silent --fail --show-error --location --retry 3 https://sh.rustup.rs | \
        sh -s -- --default-toolchain ${RUST_TOOLCHAIN} --verbose -y && \
    curl https://bootstrap.pypa.io/get-pip.py  | python && \
    pip install virtualenv

ENV CARGO_HOME=/root/.cargo
ENV PATH=${CARGO_HOME}/bin:${PATH}

COPY . /code/

RUN set -x  && \
    MAKE_TARGET=coverage bash -x /code/scripts/travis/install.sh && \
    MAKE_TARGET=lint bash -x /code/scripts/travis/install.sh

WORKDIR /code

VOLUME ${CARGO_HOME}/registry
VOLUME /code/.coverage
VOLUME /code/src
VOLUME /code/build.rs
VOLUME /code/target

CMD ["/bin/bash"]
