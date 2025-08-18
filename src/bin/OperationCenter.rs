use gstreamer::prelude::{ElementExt, GstBinExt};
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Image};
use gdk4::Paintable;
use gstreamer as gst;
use std::env;

fn dump_env() {
    eprintln!("--- ENV DUMP START ---");
    for k in ["HOME","PATH","GST_PLUGIN_PATH","LD_LIBRARY_PATH","XDG_DATA_DIRS","GIO_EXTRA_MODULES","DISPLAY"].iter() {
        eprintln!("{:20} = {:?}", k, env::var(k));
    }
    eprintln!("--- ENV DUMP END ---");
}

unsafe fn set_missing_env() {
    env::set_var("GST_PLUGIN_PATH", "/home/olbap/Teleop-OperationCenter/gst-plugins-rs/build");
}


fn create_ui(app: &Application) {
    // Create main window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Operation Center")
        .default_width(800)
        .default_height(600)
        .build();

    // Create an Image widget which will render a Paintable (the video sink provides it)
    let image = Image::new();
    window.set_child(Some(&image));

    // Build a simple pipeline with a named sink:
    // we use gtk4paintablesink which exposes a `paintable` property (GdkPaintable)
    // let pipeline_str = "videotestsrc pattern=ball ! videoconvert ! gtk4paintablesink name=mysink";
    // let pipeline_str = "udpsrc address=0.0.0.0 port=5000 caps=\"application/x-rtp,media=video,encoding-name=JPEG,payload=26\" ! rtpjpegdepay ! jpegdec ! gtk4paintablesink name=mysink";
    let pipeline_str = concat!(
        "udpsrc address=0.0.0.0 port=5000 caps=\"application/x-rtp,media=video,encoding-name=JPEG,payload=26\" ",
        "! rtpjitterbuffer ! rtpjpegdepay ! jpegdec ! videoconvert ! identity name=probe_id ! queue ! gtk4paintablesink name=mysink"
    );
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
        .property::<Paintable>("paintable");
    // .expect("Failed to get 'paintable' property from gtk4paintablesink");

    // Attach paintable to the image widget
    image.set_paintable(Some(&paintable));

    // Show the window
    window.present();

    // Start playback
    pipeline
        .set_state(gst::State::Playing)
        .expect("Unable to set the pipeline to Playing");
}

fn main() {
    #[cfg(debug_assertions)]
    {
        unsafe {
            set_missing_env();
        }
    }
    dump_env();

    // Initialize GStreamer first (so GStreamer plugins are ready when GTK starts)
    gst::init().expect("Failed to initialize GStreamer");

    // Build a GTK Application
    let app = Application::builder()
        .application_id("com.example.minmal")
        .build();

    app.connect_activate( |app| {
        create_ui(app);
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