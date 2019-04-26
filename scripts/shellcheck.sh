#!/usr/bin/env bash
set -euo pipefail -o posix -o functrace

REPO_ROOT="$( cd "$( dirname "${BASH_SOURCE[0]}" )/../" >/dev/null 2>&1 && pwd )"

if command -v shellcheck &> /dev/null; then
    SHELLCHECK=shellcheck
elif command -v docker &> /dev/null; then
    # Command extracted from https://github.com/pre-commit/pre-commit/blob/v1.15.2/pre_commit/languages/docker.py
    SHELLCHECK="docker run
        --rm
        -v ${REPO_ROOT}:/src:rw,Z
        --workdir /src
        koalaman/shellcheck:latest
    "
else
    echo "shellcheck is not installed on your machine and you don't have docker" > /dev/stderr
    echo "Ignoring it for now as this checks only scripts and not 'production' code" > /dev/stderr
    exit 0
fi

${SHELLCHECK} "${@}"
