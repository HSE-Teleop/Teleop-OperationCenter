# GStreamer integration

Trying to integrate the GStreamer pipeline into an application via Rust bindings and plugins.

## 1. Attempt

Initializing the app is no problem. A full guide on how to install the necessary dependencies can be found in the [README](README.md).

It looks like the application can build via cargo crate, but as soon as the pipeline tries to get integrated, it fails.
The appearing error is that the pipeline description cannot be parsed.

GStreamer usually waits till it receives data from a source, so it should not crash even if there is no current data exchange. That means that the sink (the part that is responsible for the data exchange) fails or is not working correctly.
In the examples of the GStreamer documentation, c-Code is used with video- and audio-sinks. For Rust there should be something similar named 'gtk4paintablesink' which uses c-bindings.
Apparently this sink is not working or is missing some dependencies.
It should get installed with the 'gstreamer1.0-gtk4' package but somehow does not.

It seems to be an extra package that has to be built.</br>
For that we use the official GitLab repository for plugins: https://gitlab.freedesktop.org/gstreamer/gst-plugins-rs.</br>
To build this project, we need cargo-c, which requires the c toolchain from OpenSSL:
```bash
  sudo apt update
  sudo apt install pkg-config libssl-dev build-essential openssl
  
  cargo install cargo-c     # Installing in the user space
```
---
Some issues can appear while downloading, updating or building when using different versions.
Make sure to have a stable and new version of Rust and rustc to install cargo-c.
When using rustup (`sudo apt install rustup`) be aware that it installs and manages packages for the current user not globally.
---
After that we can build the project with the command suggested in the GitLab repository:
```bash
  cargo cbuild -p gst-plugin-gtk4
  cargo cinstall -p gst-plugin-gtk4 --prefix=/usr       # Installs package system-wide under /usr/...
```
Now the last thing to do is to point to the library which holds the gtk4paintablesink.

---
If the build and installation process worked, then find out the correct path in your /usr directory where your `.pc`/`.so` files are located.

If the GStreamer inspect tool cannot find the sink (`gst-inspect-1.0 gtk4paintablesink`), add the target path to the global path (`export GST_PLUGIN_PATH=<PATH_TO_TARGET_DIR>:$GST_PLUGIN_PATH`).

## 2. Attempt

Somehow the build command suggested on the GitLab repository sometimes fails while copying the relevant files.

We can bypass this by building the project in the user directory and point to it in the global path variable (GST_PLUGIN_PATH):
```bash
  meson setup build
  ninja -C build
  sudo ninja -C build install     # Optionally: tries also to install system wide
```
After successfully building the project you should find a build directory within the project containing the necessary `.pc`/`.so` files.
Add this directory to the global path like this:
```bash
  export GST_PLUGIN_PATH=<PATH_TO_THE_CLONED_PROJECT>/build:$GST_PLUGIN_PATH
```
Verify that GStreamer finds that plugin with:
```bash
  gst-inspect-1.0 gtk4paintablesink
  # If you want to see if a directory might contain the right package, pass the directory as an option
  gst-inspect-1.0 --gst-plugin-path=<DIRECTORY> gtk4paintablesink
```

## 3. Attempt

As an absolute test, run a default pipeline with the sink inside your wsl:
```bash
  gst-launch-1.0 videotestsrc ! videoconvert ! gtk4paintablesink
```
If you are able to see a window, then everything works fine.

Now some last problems could occur when running the OperationCenter project outside the wsl.
Sometimes the environment paths differ from each other.
That is why we have to tell our process during runtime where to look (`std::env::set_var("GST_PLUGIN_PATH", "<PATH_TO_PLUGIN>"`).

# Dockerize

Integrating the GStreamer plugin should now work. 
To make it easier and to safe time and trouble, we will build an image to skip the building process.

## Dockerfile

Building an image is pretty much straightforward. We had to just follow the building instructions using Dockerfile syntax.
That is what the Dockerfile is doing. 

In short, we build the plugin used for the paintablesink and our app.
After that we copy the results into a new image so that it is cleaner.

[We had a little trouble there with the image versions and installing of packages]

## GUI forwarding

Our app provides a GUI to show our GStreamer pipeline, so we have to forward the display somehow to the client out of the docker container.

We are using WSL 2 with wslg the integrated way to display apps outside the wsl and Docker Desktop.
This allows use to run docker commands within the wsl. Which also means that the client system has to provide the newest functionality.

With that setup we can customize the 'docker run' command so that the app can connect to the wslg socket and forward the GUI.
The corresponding [script](run_wslg.sh) then needs to be executed inside the wsl with the necessary environment variables.

The script uses the wayland socket and mounts it through the docker container to the wsl.