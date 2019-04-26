#!/usr/bin/env bash
set -euo pipefail -o posix -o functrace

not_executable_bash_scripts=$(for file in "$@"; do
    if [[ ! -x "${file}" ]]; then
	echo -n "${file} "
    fi
done)

if [[ "${not_executable_bash_scripts}" != "" ]]; then
    echo "The following scripts are not executable: ${not_executable_bash_scripts}" > /dev/stderr
    exit 1
else
    exit 0
fi
