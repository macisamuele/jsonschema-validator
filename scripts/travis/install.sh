#!/usr/bin/env bash
set -euo pipefail -o posix -o functrace
# The script is needed to install all the needed dependencies on the CI server
# Usage: bash <path to this file>

install_make() {
  # Install make on windows
  if [[ "${TRAVIS_OS_NAME}" == "windows" ]]; then
    choco install make
  fi
}

install_grcov() {
  if [[ ${MAKE_TARGET} == "coverage" ]]; then
    if ! command -v grcov @> /dev/null; then
      CARGO_HOME=${CARGO_HOME:-${HOME}/.cargo}
      GRCOV_DEFAULT_VERSION="v0.5.5"
      GITHUB_GRCOV="https://api.github.com/repos/mozilla/grcov/releases/latest"

      # Usage: download and install the latest kcov version by default.
      # Fall back to ${GRCOV_DEFAULT_VERSION} from the kcov archive if the latest is unavailable.
      GRCOV_VERSION=$(curl --silent --show-error --fail ${GITHUB_GRCOV} | jq -Mr .tag_name || echo)
      GRCOV_VERSION=${GRCOV_VERSION:-$GRCOV_DEFAULT_VERSION}

      GRCOV_TARBZ2="https://github.com/mozilla/grcov/releases/download/${GRCOV_VERSION}/grcov-linux-x86_64.tar.bz2"

      if curl -L --retry 3 "${GRCOV_TARBZ2}" | tar xjvf -; then
        mv ./grcov "${CARGO_HOME}/bin"
      else
        # Fallback to manually build grcov if we failed to find the pre-built release
        cargo install grcov
      fi
    fi
  fi
}

download_codecov_bash_script() {
  if [[ ${MAKE_TARGET} == "coverage" ]]; then
    make "${CODECOV_DIR}/codecov.bash"
  fi
}

install_lint_tools() {
  # Install pre-commit and lint tools
  if [[ ${MAKE_TARGET} == "lint" ]]; then
    if ! (pip freeze | grep -q pre-commit); then
      pip install --no-cache-dir --user pre-commit
    fi
    if ! (rustup component list --toolchain="${TRAVIS_RUST_VERSION}" | grep installed | grep -q rustfmt); then
      rustup component add rustfmt --toolchain="${TRAVIS_RUST_VERSION}"
    fi
    if ! (rustup component list --toolchain="${TRAVIS_RUST_VERSION}" | grep installed | grep -q clippy); then
      # Workaround in case clippy is not available in the current nightly release (https://github.com/rust-lang/rust-clippy#travis-ci)
      rustup component add clippy --toolchain="${TRAVIS_RUST_VERSION}" || cargo +"${TRAVIS_RUST_VERSION}" install --git https://github.com/rust-lang/rust-clippy/ --force clippy
    fi
  fi
}

install_make
install_grcov
download_codecov_bash_script
install_lint_tools
