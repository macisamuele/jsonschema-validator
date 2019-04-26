#!/usr/bin/env bash
set -euo pipefail -o posix -o functrace

cargo metadata --format-version 1 | python3 -c "$(cat <<EOF
import json
import sys

metadata = json.load(sys.stdin)
package_metadata = next(filter(lambda item: item['id'] == metadata['resolve']['root'], metadata['packages']))
print(' '.join(filter(lambda item: item != 'default', package_metadata['features'])))
EOF
)"
