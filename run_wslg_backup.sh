#!/usr/bin/env bash
set -eo pipefail
# use `set -euo pipefail` to treat unset variables as error
IMAGE="operation-center:latest"
# build image if needed
docker build -t "$IMAGE" .
docker run --rm -it \
  -e DISPLAY="${DISPLAY:-}" \
  -e WAYLAND_DISPLAY="${WAYLAND_DISPLAY:-}" \
  -e XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-}" \
  -v /tmp/.X11-unix:/tmp/.X11-unix \
  -v "$XDG_RUNTIME_DIR":"$XDG_RUNTIME_DIR" \
  --device /dev/dri \
  --user "$(id -u):$(id -g)" \
  --name operation-center \
  "$IMAGE"