#!/usr/bin/env bash
set -euo pipefail -o posix -o functrace

git submodule foreach --recursive 'git fetch origin && git reset --hard origin/master'

if [[ ! -f .gitmodules ]]; then
    echo "No registered submodules"
    exit 0
fi

submodules=$(awk '$1 ~ /path/ {print $3}' .gitmodules)
if [[ ${#submodules} -eq 0 ]]; then
    echo "No registered submodules"
    exit 0
fi

# shellcheck disable=SC2086
if git diff --exit-code ${submodules}; then
    # No differences have been identified
    exit 0
else
    echo "Submodules have been bumped, please make sure to commit them \`git add ${submodules}\`"
    exit 1
fi
