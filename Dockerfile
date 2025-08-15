# Builder stage: build plugin and rust app
#FROM rust:1-slim-bookworm AS builder
# Using custom rust image on Ubuntu because gtk4 somehow can not be found on debian
FROM ubuntu AS builder

# Install system build deps + pip
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
    ca-certificates build-essential pkg-config git meson ninja-build \
    python3 python3-pip python3-setuptools python3-wheel \
    apt-utils curl gcc \
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
    libgstreamer-plugins-base1.0-dev \
    libgirepository1.0-dev \
    gir1.2-gtk-4.0 \
    libepoxy-dev \
    libx11-dev \
    libwayland-dev \
    libxkbcommon-dev \
 && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="$PATH:~/.cargo/bin"
ENV PATH="/root/.cargo/bin:${PATH}"

# Update the local crate index
RUN ~/.cargo/bin/cargo search


# Install modern meson via pip (bypassing PEP 668)
# RUN python3 -m pip install --no-cache-dir --break-system-packages --upgrade pip \
#   && python3 -m pip install --no-cache-dir --break-system-packages 'meson>=1.1'
RUN python3 -m pip install --no-cache-dir --break-system-packages 'meson>=1.1'

# Build gst-plugins-rs and install it into /usr/local
WORKDIR /usr/src/plugin

RUN git clone https://gitlab.freedesktop.org/gstreamer/gst-plugins-rs.git
WORKDIR /usr/src/plugin/gst-plugins-rs
# configure, build, install
ENV PKG_CONFIG_PATH="/usr/lib/x86_64-linux-gnu/pkgconfig:/usr/share/pkgconfig:${PKG_CONFIG_PATH:-/usr/lib/x86_64-unknown-linux-gnu}"

RUN cargo install cargo-c
RUN cargo cbuild -p gst-plugin-gtk4 --prefix=/usr/local \
  && cargo cinstall -p gst-plugin-gtk4 --prefix=/usr/local

#RUN meson setup build --prefix=/usr/local \
# && ninja -C build \
# && ninja -C build install

# Build OperationCenter
WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

ENV PATH="/usr/local/bin:$PATH"
RUN cargo build --release --bin OperationCenter

# Runtime image
FROM ubuntu AS runtime-base
#FROM debian:bookworm-slim AS runtime-base

#ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
 && apt-get install -y --no-install-recommends \
    ca-certificates gcc \
    # GStreamer runtime packages
    dbus-x11 \
    gstreamer1.0-tools \
    gstreamer1.0-plugins-base \
    gstreamer1.0-plugins-good \
    gstreamer1.0-plugins-bad \
    gstreamer1.0-plugins-ugly \
    gstreamer1.0-libav \
    libgstreamer1.0-0 \
    libgstreamer-plugins-base1.0-0 \
    # GTK4 dependencies \
        libgtk-4-dev \
        libglib2.0-dev \
        libcairo2-dev \
        libpango1.0-dev \
        libgdk-pixbuf-2.0-dev \
        libepoxy-dev \
        libx11-dev \
        libwayland-dev \
        libxkbcommon-dev \
    gir1.2-gtk-4.0 \
    libgtk-4-1 \
    libglib2.0-0 \
    libgdk-pixbuf-2.0-0 \
    libcairo2 \
    libpango-1.0-0 \
    libatspi2.0-0 \
    libepoxy0 \
    libwayland-client0 \
 && rm -rf /var/lib/apt/lists/*

# Copy Operation Center app
FROM runtime-base AS operation-center

# Copy the built binary
COPY --from=builder /usr/src/app/target/release/OperationCenter /usr/local/bin/OperationCenter

# Copy installed gst plugin files from builder's install location
COPY --from=builder /usr/local /usr/local

# Make sure GStreamer will look there
ENV GST_PLUGIN_PATH="/usr/local/lib/gstreamer-1.0:${GST_PLUGIN_PATH:-/usr/local/lib/x86_64-unknown-linux-gnu/debug}"
ENV GST_PLUGIN_PATH="/usr/local/lib:${GST_PLUGIN_PATH:-}"
ENV GST_PLUGIN_PATH="/usr/local:${GST_PLUGIN_PATH:-}"

# Set entrypoint
ENTRYPOINT ["/usr/local/bin/OperationCenter"]