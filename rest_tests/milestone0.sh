#!/usr/bin/env bash
set -euo pipefail

bash run.sh

status_code=$(curl -s -o /dev/null -w "%{http_code}" localhost:31415/healthz)
if [ "$status_code" -ne 200 ]; then
  echo "Failed: $status_code"
  exit 1
fi

echo "done."

bash down.sh