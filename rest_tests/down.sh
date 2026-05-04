#!/usr/bin/env bash
set -euo pipefail

docker compose -f ./../blobfish/docker-compose.yml down --rmi all