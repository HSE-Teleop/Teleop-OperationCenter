# Dockerfile.arm64
# ARM64 / aarch64 variant of the provided Ubuntu-based Dockerfile.
# Usage: build on an ARM64 host or use docker buildx for cross-building from x86_64.

# Builder stage: build plugin and rust app (ARM64)
FROM --platform=linux/arm64 ubuntu:25.10 AS builder

# Install system build deps + pip
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
    ca-certificates build-essential pkg-config git meson ninja-build \
    python3 python3-pip python3-setuptools python3-wheel \
    apt-utils curl gcc-aarch64-linux-gnu gcc \
    libssl-dev openssl \
    libgstreamer1.0-dev \
    libgstreamer-plugins-base1.0-dev \
    libgstreamer-plugins-bad1.0-dev \
    libgtk-4-dev \
    libglib2.0-dev \
    libcairo2-dev \
    libpango1.0-dev \
    libgdk-pixbuf-2.0-dev \
    libatk1.0-dev \
    libgraphene-1.0-dev \
    libgirepository1.0-dev \
    gir1.2-gtk-4.0 \
    libepoxy-dev \
    libx11-dev \
    libwayland-dev \
    libxkbcommon-dev \
 && rm -rf /var/lib/apt/lists/*

# Install Rust (rustup)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Update the local crate index (helpful to warm-up the registry)
RUN /root/.cargo/bin/cargo search

# Install modern meson via pip (bypass PEP 668)
RUN python3 -m pip install --no-cache-dir --break-system-packages 'meson>=1.1'

# Build gst-plugins-rs and install it into /usr/local
WORKDIR /usr/src/plugin

RUN git clone https://gitlab.freedesktop.org/gstreamer/gst-plugins-rs.git
WORKDIR /usr/src/plugin/gst-plugins-rs

# Adjust PKG_CONFIG_PATH for aarch64 multiarch
ARG PKG_CONFIG_PATH_DEFAULT="/usr/lib/aarch64-linux-gnu/pkgconfig"
ENV PKG_CONFIG_PATH="/usr/lib/aarch64-linux-gnu/pkgconfig:/usr/share/pkgconfig:${PKG_CONFIG_PATH:-${PKG_CONFIG_PATH_DEFAULT}}"

# Configure, build, install
RUN /root/.cargo/bin/cargo install cargo-c
# If you run this natively on an ARM host, cargo cbuild will build for native arch.
# If cross-building, consider setting TARGET (see note below).
RUN /root/.cargo/bin/cargo cbuild -p gst-plugin-gtk4 --prefix=/usr/local \
  && /root/.cargo/bin/cargo cinstall -p gst-plugin-gtk4 --prefix=/usr/local

# Build OperationCenter
WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

ENV PATH="/usr/local/bin:$PATH"
RUN /root/.cargo/bin/cargo build --release --bin OperationCenter

# Runtime image (ARM64)
FROM --platform=linux/arm64 ubuntu:25.10 AS runtime-base

RUN apt-get update \
 && apt-get install -y --no-install-recommends \
    ca-certificates gcc \
    # GStreamer runtime packages
    gstreamer1.0-tools \
    gstreamer1.0-plugins-base \
    gstreamer1.0-plugins-good \
    gstreamer1.0-plugins-bad \
    gstreamer1.0-plugins-ugly \
    gstreamer1.0-libav \
    libgstreamer1.0-0 \
    libgstreamer-plugins-base1.0-0 \
    # GTK4 runtime libs
    libgtk-4-1 \
    libglib2.0-0 \
    libcairo2 \
    libpango-1.0-0 \
    libgdk-pixbuf-2.0-0 \
    libepoxy0 \
    libx11-6 \
    libwayland-client0 \
    libwayland-cursor0 \
    libwayland-egl1 \
    libegl-mesa0 \
    libgbm1 \
    libdrm2 \
    libxkbcommon0 \
    dbus-x11 python3-gi \
    gir1.2-gtk-4.0 \
 && rm -rf /var/lib/apt/lists/*

# Copy Operation Center app
FROM runtime-base AS operation-center

# Copy the built binary
COPY --from=builder /usr/src/app/target/release/OperationCenter /usr/local/bin/OperationCenter

# Copy installed gst plugin files from builder's install location
COPY --from=builder /usr/local /usr/local

# Set GST_PLUGIN_PATH to common aarch64 and generic install locations
ARG GST_PLUGIN_PATH_DEFAULT="/usr/local/lib/aarch64-linux-gnu/gstreamer-1.0"
ENV GST_PLUGIN_PATH="/usr/local/lib/gstreamer-1.0:${GST_PLUGIN_PATH_DEFAULT}:${GST_PLUGIN_PATH:-}"
ENV GST_PLUGIN_PATH="/usr/local/lib:${GST_PLUGIN_PATH}"
ENV GST_PLUGIN_PATH="/usr/local:${GST_PLUGIN_PATH}"

# Expose port and set entrypoint
EXPOSE 5000
ENTRYPOINT ["/usr/local/bin/OperationCenter"]

# Notes:
# - This Dockerfile assumes you will build/run on an ARM64 (aarch64) environment.
# - If you need to cross-build from x86_64 to aarch64, use docker buildx and ensure QEMU support is enabled
#   (or install aarch64 cross-toolchains and adapt the build steps to use a cross linker and proper TARGET).
# - If you want to explicitly force cargo to target aarch64 when cross-compiling, set an ARG like TARGET=aarch64-unknown-linux-gnu
#   and pass --target=$TARGET to cargo/cargo-cbuild, and provide an appropriate cross-compiler toolchain.
