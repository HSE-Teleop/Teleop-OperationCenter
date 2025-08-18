#!/bin/bash
set -euo pipefail

# Docker registry
IMAGE="ghcr.io/hse-teleop/teleop-operationcenter/operation-center:latest"
NAME="${NAME:-operation-center}"
GL_MODE="${GL_MODE:-dri3_off}"

# Normalize XDG_RUNTIME_DIR (remove trailing slash)
XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-}"
XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR%/}"

echo "==> Normalized XDG_RUNTIME_DIR='${XDG_RUNTIME_DIR:-<unset>}'"

echo "==> Skip image build: $IMAGE"

# Host-side quick diagnostics (helps spot permission/owner issues)
if [ -n "$XDG_RUNTIME_DIR" ]; then
  echo "==> Host-side Wayland socket info (host):"
  if [ -S "${XDG_RUNTIME_DIR}/${WAYLAND_DISPLAY:-wayland-0}" ]; then
    ls -l "${XDG_RUNTIME_DIR}/${WAYLAND_DISPLAY}"
    stat -c 'mode=%a owner=%U:%G path=%n' "${XDG_RUNTIME_DIR}/${WAYLAND_DISPLAY}"
  else
    echo "   No socket file at ${XDG_RUNTIME_DIR}/${WAYLAND_DISPLAY:-wayland-0}"
    ls -ld "$XDG_RUNTIME_DIR" || true
  fi
fi

# Build docker args
DOCKER_ARGS=( --rm -it --name "$NAME" --user "$(id -u):$(id -g)" )

# share /dev/shm
[ -d /dev/shm ] && DOCKER_ARGS+=( -v /dev/shm:/dev/shm )

# Additional mount (mount socket to allow GUI display)
DOCKER_ARGS+=( -v /run/user/"$(id -u)"/"${WAYLAND_DISPLAY:-wayland-0}":/run/user/"$(id -u)"/"${WAYLAND_DISPLAY:-wayland-0}" )

# Mount XDG_RUNTIME_DIR once (needed for wayland socket & dbus)
if [ -n "$XDG_RUNTIME_DIR" ] && [ -d "$XDG_RUNTIME_DIR" ]; then
  DOCKER_ARGS+=( -e XDG_RUNTIME_DIR="$XDG_RUNTIME_DIR" -v "$XDG_RUNTIME_DIR":"$XDG_RUNTIME_DIR" )
fi

# If Wayland exists, prefer it and DO NOT pass DISPLAY/X11
if [ -n "${WAYLAND_DISPLAY:-}" ] && [ -n "$XDG_RUNTIME_DIR" ] && [ -S "${XDG_RUNTIME_DIR}/${WAYLAND_DISPLAY}" ]; then
  echo "==> Using Wayland: ${XDG_RUNTIME_DIR}/${WAYLAND_DISPLAY}"
  DOCKER_ARGS+=( -e WAYLAND_DISPLAY="$WAYLAND_DISPLAY" -e GDK_BACKEND=wayland -e XDG_SESSION_TYPE=wayland )
else
  echo "INFO: Wayland socket missing or not mounted. falling back to X11 (if available)."
  [ -n "${DISPLAY:-}" ] && DOCKER_ARGS+=( -e DISPLAY="$DISPLAY" -v /tmp/.X11-unix:/tmp/.X11-unix )
fi

# Forward DBUS if present
if [ -n "$XDG_RUNTIME_DIR" ] && [ -S "${XDG_RUNTIME_DIR}/bus" ]; then
  DOCKER_ARGS+=( -e DBUS_SESSION_BUS_ADDRESS="unix:path=${XDG_RUNTIME_DIR}/bus" )
fi

# GL modes
case "$GL_MODE" in
  auto) ;;
  dri3_off) DOCKER_ARGS+=( -e LIBGL_DRI3_DISABLE=1 ) ;;
  software) DOCKER_ARGS+=( -e LIBGL_ALWAYS_SOFTWARE=1 ) ;;
esac

# Pass GST/PKG envs if present
[ -n "${GST_PLUGIN_PATH:-}" ] && DOCKER_ARGS+=( -e GST_PLUGIN_PATH="$GST_PLUGIN_PATH" )
[ -n "${PKG_CONFIG_PATH:-}" ] && DOCKER_ARGS+=( -e PKG_CONFIG_PATH="$PKG_CONFIG_PATH" )

# Final debug command
#echo "==> Final docker args:"
#printf '  %s\n' "${DOCKER_ARGS[@]}"

# Run and drop into container (entrypoint or binary will run afterwards)
exec docker run "${DOCKER_ARGS[@]}" -p 5000:5000/udp --pull=always "$IMAGE"