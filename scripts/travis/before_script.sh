#!/usr/bin/env bash
set -euo pipefail -o posix -o functrace
# The script is needed to dump all the version of the tooling that is installed on the CI Server.
# This is needed to provide useful information in case of build failures that are related to weirdly behaving versions

if [[ ${MAKE_TARGET} == "lint" ]]; then pre-commit --version; fi
rustup --version
rustup show
rustc --version --verbose
cargo --version --verbose
if [[ ${MAKE_TARGET} == "coverage" ]]; then grcov --version; fi
