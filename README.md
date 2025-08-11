# Teleop-OperationCenter

This repository provides a minimal GUI for the vehicle and its control.

---

**Table of contents**
1. **[Developing](#developing)**

##

---

## Developing

**For building and developing use the rust tool chain of Linux!**

This can be done by installing a Linux wsl when using Windows or other.
To make sure the build is running through, you need to install the following packages in your wsl or the Linux system your tool chain is running on:

```bash
  sudo apt install \
    libgtk-4-dev \
    libglib2.0-dev \
    libcairo2-dev \
    libpango1.0-dev \
    libgdk-pixbuf-2.0-dev \
    libatk1.0-dev \
    libgraphene-1.0-dev \
    libgstreamer1.0-dev \
    libgstreamer-plugins-base1.0-dev \
    pkg-config \
    build-essential
```
If needed, also install the necessary packages for GStreamer:
```bash
  sudo apt install \
    libgstreamer1.0-dev \
    libgstreamer-plugins-base1.0-dev \
    libgstreamer-plugins-good1.0-dev \
    libgstreamer-plugins-bad1.0-dev \
    libgstreamer-plugins-ugly1.0-dev
```
Now you should be good to go.

Run the `cargo build` command and search for the built file `.../target/debug/OperationCenter` in your Linux environment.</br>
To execute the file run `./OperationCenter`. This should open a window.