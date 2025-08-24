# Teleop-OperationCenter

This repository provides a minimal GUI for the vehicle and its control.

---

**Table of contents**
1. **[Setup](#setup)**
2. **[Camera](#camera)**
3. **[Developing](#developing)**
4. **[Deploying](#runningdeploying)**
5. **[Troubleshooting](#troubleshooting)**

## Setup

In this section is everything described that needs to be done to work on this project.

### Rust toolchain

To run and build Rust applications, you need to install the Rust toolchain.
It is recommented to use the Linux toolchain for Rust. Therefore, install a wsl if not working on a Linux distribution.
In there execute `sudo apt install cargo`.</br>
Some IDEs find the path automatically others don't.
If not, the most common path to the binaries is the `/usr/bin` directory.
When using a wsl, it should look something like this `\\wsl.localhost\Ubuntu\usr\bin`.

### Technologies

We use Docker images to deploy the 'Operation Center'.
We recommend using Docker Desktop, and furthermore, an Ubuntu wsl if it's not installed yet.
If working with two distros make sure to use wsl2 in Docker and have Docker CLI (`sudo apt install docker-cli`) installed on the other wsl.

### WSL customization

!!!

UDP doesn't get forwarded to wsl (only TCP)

!!!

#### UDP Forwarding

Use open-source [UDP-Forwarder](https://github.com/matthid/UdpPortForwarder).
Has to run in the background to forward GStreamer pipeline via UDP to wsl and Docker.


## Camera

Camera access and resolution.

### Pi Cam

For pi cams there is an extra library that allows simple and direct access (`sudo apt install -y libcamera-apps`).


### USB Cams



## Developing

**For building and developing use the rust tool chain of Linux!**

This can be done by installing a Linux wsl when using Windows or other.
To make sure the build is running through, you need to install the following packages in your wsl or the Linux system your tool chain is running on:

```bash
  sudo apt install -y \
    libgtk-4-dev \
    libglib2.0-dev \
    libcairo2-dev \
    libpango1.0-dev \
    libgdk-pixbuf-2.0-dev \
    libatk1.0-dev \
    libgraphene-1.0-dev \
    libgstreamer-plugins-base1.0-dev \
    pkg-config \
    build-essential
```
If needed, also install the necessary packages for GStreamer:
```bash
  sudo apt install -y \
    gstreamer1.0-tools \
    gstreamer1.0-plugins-base \
    gstreamer1.0-plugins-good \
    gstreamer1.0-plugins-bad \
    gstreamer1.0-plugins-ugly \
    gstreamer1.0-libav \
    gstreamer1.0-gl \
    gstreamer1.0-alsa \
    gstreamer1.0-libcamera
```
Now you should be good to go.

Run the `cargo build` command and search for the built file `.../target/debug/OperationCenter` in your Linux environment.</br>
To execute the file run `./OperationCenter`. This should open a window.

More simple way, run the `run_wslg.sh` script to automatically build and run the application.

## Running/Deploying

Here are the instructions to deploy and run the Operation Center.
It is tested on a Windows machine using Docker and an Ubuntu wsl.

Execute the `install_OperationCenter.sh` file inside the wsl.</br>
This can be done by either cloning the repository or just the file from [GitHub](https://github.com/HSE-Teleop/Teleop-OperationCenter.git).
When downloaded on your host machine search for the file in the mount directory (`/mnt/`) inside your wsl.



## Troubleshooting

Fixes for common issues in each section.

### Running/Deploying

***Docker permission error:***

If the display socket in your wsl can be found but docker denies the connection to its Docker daemon socket, it is probably a root-protection error.
Fix it by adding your local user to the group of executables.</br>
```bash
    # create docker group if it doesn't exist (safe: -f won't error)
    sudo groupadd -f docker
    # add your user to the group
    sudo usermod -aG docker "$(id -un)"
```
