use gtk4 as gtk;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, DrawingArea, Image};
use gdk4::Paintable;
use gstreamer as gst;
use std::env;
use gstreamer::{ElementFactory, Pipeline};
use gstreamer::prelude::GstBinExt;

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
    let pipeline = Pipeline::new(None);
    let video_sink = ElementFactory::make("gtk4paintablesink");
    pipeline.add(&video_sink).unwrap();

    // Link the DrawingArea to the video sink
    if let Some(widget) = video_sink.property("widget", ()) {
        drawing_area.set_child(Some(&widget));
    }

    // Set the pipeline to play
    pipeline.set_state(gstreamer::State::Playing);

    // Show the window
    window.show();
}

fn main() {
    // Initialize GStreamer first (so GStreamer plugins are ready when GTK starts)
    gst::init().expect("Failed to initialize GStreamer");

    // Build a GTK Application
    let app = Application::builder()
        .application_id("com.example.gstgtk4")
        .build();

    app.connect_activate(move |app| {
        // Create main window
        let window = ApplicationWindow::builder()
            .application(app)
            .title("GStreamer GTK4 Video")
            .default_width(800)
            .default_height(600)
            .build();

        // Create an Image widget which will render a Paintable (the video sink provides it)
        let image = Image::new();
        window.set_child(Some(&image));

        // Build a simple pipeline with a named sink:
        // we use gtk4paintablesink which exposes a `paintable` property (GdkPaintable)
        // Example pipeline: a test source -> convert -> gtk4paintablesink
        let pipeline_str = "videotestsrc pattern=ball ! videoconvert ! autovideosink name=mysink";
        let parsed = gst::parse_launch(pipeline_str).expect("Failed to parse pipeline");
        let pipeline = parsed
            .downcast::<gst::Pipeline>()
            .expect("Expected parsed pipeline to be a gst::Pipeline");

        // Get the sink element by name:
        let sink = pipeline
            .by_name("mysink")
            .expect("Could not find the gtk4paintablesink element (name=mysink)");

        // Get the `paintable` property from the sink.
        // This returns a gdk4::Paintable which GTK can render directly.
        let paintable: Paintable = sink
            .property("paintable")
            .expect("Failed to get 'paintable' property from gtk4paintablesink");

        // Attach paintable to the image widget
        image.set_paintable(Some(&paintable));

        // Show the window
        window.present();

        // Start playback
        pipeline
            .set_state(gst::State::Playing)
    });

    // Run the app
    // NOTE: pass command-line args so GTK can parse them
    let args: Vec<String> = env::args().collect();
    app.run_with_args(&args);
}

// // sender
// // gst-launch-1.0 -v libcamerasrc ! x264enc tune=zerolatency speed-preset=ultrafast ! rtph264pay pt=96 ! udpsink host=<PC_IP> port=5000
// // receiver
// // gst-launch-1.0 -v udpsrc port=5000 caps="application/x-rtp,media=video,encoding-name=H264,payload=96" ! rtph264depay ! decodebin ! autovideosink