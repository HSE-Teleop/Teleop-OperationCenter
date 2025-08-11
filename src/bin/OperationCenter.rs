use gstreamer as gst;
use gst::prelude::*;

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{gdk, gio, glib};

use gtk::prelude::*;
use gstreamer::prelude::*;
use gtk::{Application, ApplicationWindow, DrawingArea};
use gstreamer::{ElementFactory, Pipeline};

fn create_ui(app: &gtk::Application) {
    // Create a window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Operation Center")
        .default_width(800)
        .default_height(600)
        .build();

    // Create a DrawingArea for video output
    let drawing_area = DrawingArea::new();
    window.set_child(Some(&drawing_area));

    // Create a GStreamer pipeline
    let pipeline = Pipeline::new();
    let video_sink = ElementFactory::make("gtk4paintablesink");
    pipeline.add(&video_sink).unwrap();

    // Link the DrawingArea to the video sink
    if let Some(widget) = video_sink.property("widget", ()) {
        drawing_area.set_child(Some(&widget));
    }

    // Set the pipeline to play
    pipeline.set_state(gstreamer::State::Playing).expect("Unable to set pipeline to Playing");

    // Show the window
    window.show();
}

fn main() -> glib::ExitCode {
    gst::init().unwrap();
    gtk::init().unwrap();

    gstgtk4::plugin_register_static().expect("Failed to register gstgtk4 plugin");

    let app = gtk::Application::new(None::<&str>, gio::ApplicationFlags::FLAGS_NONE);

    app.connect_activate(create_ui);
    let res = app.run();

    unsafe {
        gst::deinit();
    }

    res
}
// // sender
// // gst-launch-1.0 -v libcamerasrc ! x264enc tune=zerolatency speed-preset=ultrafast ! rtph264pay pt=96 ! udpsink host=<PC_IP> port=5000
// // receiver
// // gst-launch-1.0 -v udpsrc port=5000 caps="application/x-rtp,media=video,encoding-name=H264,payload=96" ! rtph264depay ! decodebin ! autovideosink