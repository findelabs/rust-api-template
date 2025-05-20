#!/bin/sh

set -o errexit
set -o pipefail
set -o nounset

if [ $# -gt 0 ]; then
    echo "Running overridden command '$*'."
    exec "$*"
else
    echo "Running main"
    exec /app/main
fi
