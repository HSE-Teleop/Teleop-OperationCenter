use gtk4 as gtk;
use gtk::prelude::*;
use gdk4;
use gtk4::Application;
use glib;
use gstreamer as gst;
use gst::prelude::*;
use gst::MessageView;

fn main() {
    // Init GStreamer first (must succeed)
    gst::init().expect("Failed to initialize GStreamer");
    gstgtk4::plugin_register_static().expect("Failed to register gstgtk4 plugin");

    // Check required gstreamer plugins
    ensure_plugins().expect("Missing GStreamer plugins");

    // Create GTK Application
    let app = Application::builder()
        .application_id("com.example.gstgtk4")
        .build();

    app.connect_activate(build_ui);

    // Run the app
    app.run();
}

fn build_ui(app: &Application) {
    // Build UI
    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("GStreamer + GTK4 Paintable Sink")
        .default_width(720)
        .default_height(480)
        .build();

    // A Picture widget to host the Gdk Paintable from the sink
    let picture = gtk::Picture::new();
    window.set_child(Some(&picture));
    window.present();

    // Pipeline description (your UDP H264 receiver)
    // Note: the caps string needs the inner quotes; we embed them inside Rust string.
    let pipeline_description = concat!(
        "udpsrc port=5000 caps=\"application/x-rtp,media=video,encoding-name=H264,payload=96\" ",
        "! rtph264depay ! decodebin ! videoconvert ! gtk4paintablesink name=sink"
    );

    // Build a pipeline from description
    let element = gstreamer::parse::launch(pipeline_description)
        .expect("Failed to parse pipeline description");
    let pipeline = element
        .downcast::<gst::Pipeline>()
        .expect("Parsed element is not a Pipeline");

    // Sanity check: is gtk4paintablesink available?
    if gst::ElementFactory::find("gtk4paintablesink").is_none() {
        eprintln!("ERROR: gtk4paintablesink element not found. Install the gst-plugin-gtk4 / gstreamer1.0-gtk4 package.");
        // we still continue so the user can see the message in UI, but don't attempt to play
        return;
    }

    // Find the sink element by name and get its 'paintable' property
    let sink = pipeline
        .by_name("sink")
        .expect("Couldn't find element named 'sink' in pipeline");

    // The sink exports a Gdk Paintable (GObject) in property "paintable"
    let paintable: gdk4::Paintable = sink.property("paintable");
    picture.set_paintable(Some(&paintable));

    // Watch the bus for errors / EOS and stop pipeline cleanly
    let bus = pipeline.bus().expect("Pipeline without bus?");
    let pipeline_weak = pipeline.downgrade();
    // keep the guard alive for lifetime (must not be dropped)
    let _bus_watch = bus
        .add_watch_local(move |_bus, msg| {
            match msg.view() {
                MessageView::Eos(_) => {
                    if let Some(p) = pipeline_weak.upgrade() {
                        p.set_state(gst::State::Null).ok();
                    }
                    gtk4::glib::ControlFlow::Break
                }
                MessageView::Error(err) => {
                    eprintln!(
                        "GStreamer error from {:?}: {} ({:?})",
                        err.src().map(|s| s.path_string()),
                        err.error(),
                        err.debug()
                    );
                    if let Some(p) = pipeline_weak.upgrade() {
                        p.set_state(gst::State::Null).ok();
                    }
                    gtk4::glib::ControlFlow::Break
                }
                _ => gtk4::glib::ControlFlow::Continue,
            }
        })
        .expect("Failed to add bus watch");

    // Start playing
    pipeline
        .set_state(gst::State::Playing)
        .expect("Unable to set pipeline to Playing state");

    // Make sure pipeline is set to NULL on application shutdown
    let pipeline_clone = pipeline.clone();
    app.connect_shutdown(move |_| {
        pipeline_clone.set_state(gst::State::Null).ok();
    });
}

fn ensure_plugins() -> Result<(), String> {
    if gst::ElementFactory::find("udpsrc").is_none() {
        return Err("udpsrc element not found: install gstreamer1.0-plugins-good (or equivalent)".into());
    }
    if gst::ElementFactory::find("gtk4paintablesink").is_none() {
        return Err("gtk4paintablesink element not found: install gstreamer1.0-gtk4 (or build gst-plugin-gtk4)".into());
    }
    Ok(())
}