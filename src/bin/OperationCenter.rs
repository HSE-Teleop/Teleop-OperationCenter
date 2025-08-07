use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Picture};
use gdk4::Paintable;
use gstreamer as gst;
use gst::prelude::*;

fn main() {
    gtk4::init().unwrap();
    gst::init().unwrap();
    let app = Application::new(
        Some("com.example.minimal"),
        Default::default(),
    );

    app.connect_activate(build_ui);
    println!("Activated application");

    app.run();
    println!("Application stop");
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::new(app);
    window.set_title(Some("Operation Center"));
    window.set_default_size(720, 480);
    
    window.present();
}
// // sender
// // gst-launch-1.0 -v libcamerasrc ! x264enc tune=zerolatency speed-preset=ultrafast ! rtph264pay pt=96 ! udpsink host=<PC_IP> port=5000
// // receiver
// // gst-launch-1.0 -v udpsrc port=5000 caps="application/x-rtp,media=video,encoding-name=H264,payload=96" ! rtph264depay ! decodebin ! autovideosink