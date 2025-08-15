#!/usr/bin/env bash
set -euo pipefail

IMAGE="operation-center:latest"

# Build image
docker build -t "$IMAGE" .

# Prepare args
DOCKER_ARGS=()

# DISPLAY (X11)
if [ -n "${DISPLAY:-}" ]; then
  DOCKER_ARGS+=( -e DISPLAY="$DISPLAY" )
fi

# Wayland + XDG_RUNTIME_DIR + DBUS: only if present and socket exists
if [ -n "${WAYLAND_DISPLAY:-}" ] && [ -n "${XDG_RUNTIME_DIR:-}" ] && [ -S "${XDG_RUNTIME_DIR}/${WAYLAND_DISPLAY}" ]; then
  echo "INFO: Passing Wayland socket and XDG_RUNTIME_DIR into container."
  DOCKER_ARGS+=( -e WAYLAND_DISPLAY="$WAYLAND_DISPLAY" )
  DOCKER_ARGS+=( -e XDG_RUNTIME_DIR="$XDG_RUNTIME_DIR" -v "$XDG_RUNTIME_DIR":"$XDG_RUNTIME_DIR" )
fi

# If a DBUS session socket exists, pass DBUS_SESSION_BUS_ADDRESS and mount runtime dir
if [ -n "${XDG_RUNTIME_DIR:-}" ] && [ -S "${XDG_RUNTIME_DIR}/bus" ]; then
  DOCKER_ARGS+=( -e DBUS_SESSION_BUS_ADDRESS="unix:path=$XDG_RUNTIME_DIR/bus" )
  # runtime dir already mounted above if Wayland present; if not, mount just for dbus
  if ! grep -q "$XDG_RUNTIME_DIR" <<< "${DOCKER_ARGS[*]:-}"; then
    DOCKER_ARGS+=( -v "$XDG_RUNTIME_DIR":"$XDG_RUNTIME_DIR" -e XDG_RUNTIME_DIR="$XDG_RUNTIME_DIR" )
  fi
fi

# Mount X11 socket always (harmless)
DOCKER_ARGS+=( -v /tmp/.X11-unix:/tmp/.X11-unix )

# GPU device if available
if [ -d /dev/dri ]; then
  DOCKER_ARGS+=( --device /dev/dri )
fi

# Optionally forward GST_PLUGIN_PATH and PKG_CONFIG_PATH if set on host
[ -n "${GST_PLUGIN_PATH:-}" ] && DOCKER_ARGS+=( -e GST_PLUGIN_PATH="$GST_PLUGIN_PATH" )
[ -n "${PKG_CONFIG_PATH:-}" ] && DOCKER_ARGS+=( -e PKG_CONFIG_PATH="$PKG_CONFIG_PATH" )

# Run container as current user
docker run --rm -it \
  "${DOCKER_ARGS[@]}" \
  --user "$(id -u):$(id -g)" \
  --name operation-center \
  "$IMAGE"
