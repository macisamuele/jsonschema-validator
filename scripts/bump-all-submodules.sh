#!/bin/bash
set -euxo pipefail -o posix -o functrace

git submodule foreach --recursive 'git fetch origin && git reset --hard origin/master'

if [ ! -f .gitmodules ]; then
    echo "No registered submodules"
    exit 0
fi

submodules=$(awk '$1 ~ /path/ {print $3}' .gitmodules)
if [ ${#submodules} -eq 0 ]; then
    echo "No registered submodules"
    exit 0
fi

if git diff --exit-code ${submodules}; then
    # No differences have been identified
    exit 0
fi

git add ${submodules}
git commit -m "Bump submodules $(date)"
