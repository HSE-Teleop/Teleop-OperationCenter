# What has been modified

Car with CAN-Shield 2 has now a Pi4 instead of the previous Pi2.

Simulation script to bring up can0 interface and send test can data (sim_can.sh).

Provide static ip address (192.168.1.10)

Packages installed:
- update and upgrade
- can-utils, pkg-config, build-essential
- gstreamer packages

(`lsusb`)

Get camera with `v4l2-ctl --list-devices` -> See3CAM_CU135 </br>
Get camera capture `v4l2-ctl -d /dev/video0 --list-formats-ext` </br>
Then launch gstreamer (gst-launch1.0) in raw...

New approach to make app with ['eframe'](gstreamer_visualization_rust.md) and maybe system calls to python.

⇒ Not necessary