mod utils;
use utils::event_controller::register_key_controller;

use gtk4 as gtk;
use gstreamer as gst;

use gst::prelude::*;

use gtk::prelude::*;
use gtk::{gio};

use std::cell::RefCell;
use std::str::FromStr;
use gdk4::Display;
use gtk4::EventControllerKey;

fn create_ui(app: &gtk::Application) {
    let pipeline = gst::Pipeline::new();

    // let overlay = gst::ElementFactory::make("clockoverlay")
    //     .property_from_str("halignment", &"right")
    //     .property("font-desc", "Monospace 42")
    //     .build()
    //     .unwrap();

    let gtksink = gst::ElementFactory::make("gtk4paintablesink")
        .build()
        .unwrap();

    // Integrate GStreamer pipeline %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

    // Simplify: use gtk4paintablesink directly as the sink element
    let sink: gst::Element = gtksink.clone().upcast();

    // --- create UDP RTP receiving elements ---
    let src = gst::ElementFactory::make("udpsrc")
        .name("udp_src")
        .property("port", 5000i32)
        .build()
        .unwrap();

    // prefer a capsfilter rather than setting caps property on udpsrc directly
    let rtp_caps = gst::Caps::from_str(
        "application/x-rtp,media=video,encoding-name=H264,payload=96"
    ).unwrap();
    let capsfilter = gst::ElementFactory::make("capsfilter")
        .name("rtp_caps")
        .property("caps", &rtp_caps)
        .build()
        .unwrap();

    let rtph264depay = gst::ElementFactory::make("rtph264depay")
        .name("rtp_h264_depay")
        .build()
        .unwrap();

    // `h264parse` is recommended between depay and decodebin for robust parsing
    let h264parse = gst::ElementFactory::make("h264parse")
        .name("h264_parse")
        .build()
        .unwrap();

    // decodebin will create dynamic src pads we must link when they appear
    let decodebin = gst::ElementFactory::make("decodebin")
        .name("decoder")
        .build()
        .unwrap();

    // queue between decodebin and overlay to avoid blocking and for threading separation
    let queue = gst::ElementFactory::make("queue")
        .name("decode_queue")
        .build()
        .unwrap();
    // Füge videoconvert als statisches Element nach der queue hinzu
    let videoconvert = gst::ElementFactory::make("videoconvert")
        .name("videoconvert_post")
        .build()
        .unwrap();

    // %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

    // Ensure all elements, including queue, are added to the pipeline (pipeline, overlay, videoconvert, sink)
    pipeline.add_many(&[&src, &capsfilter, &rtph264depay, &h264parse, &decodebin, &queue, &videoconvert, &sink]).unwrap();

    // link the static part: udpsrc -> capsfilter -> rtph264depay -> h264parse -> decodebin
    gst::Element::link_many(&[&src, &capsfilter, &rtph264depay, &h264parse, &decodebin])
        .expect("Failed to link UDP -> depay -> parse -> decodebin");

    // Own approach: static link queue -> overlay -> videoconvert -> sink
    // queue.link(&overlay).expect("Failed to link queue -> overlay");
    queue.link(&videoconvert).expect("Failed to link queue -> overlay");
    // overlay.link(&videoconvert).expect("Failed to link overlay -> videoconvert");
    videoconvert.link(&sink).expect("Failed to link videoconvert -> sink");

    // --- dynamic pad linking: decodebin -> queue (when decodebin exposes a new src pad) ---
    let queue_weak = queue.downgrade();

    decodebin.connect_pad_added(move |_, src_pad| {
        // only handle video pads
        let caps = match src_pad.current_caps().or_else(|| Option::from(src_pad.query_caps(None))) {
            Some(c) => c,
            None => return,
        };
        let s = match caps.structure(0) {
            Some(s) => s,
            None => return,
        };
        let name = s.name();

        if !name.starts_with("video/") {
            // ignore audio or other pads
            return;
        }

        // upgrade queue
        let queue = match queue_weak.upgrade() {
            Some(q) => q,
            None => return,
        };

        // if the queue sink pad is already linked, don't try again
        let queue_sink = queue.static_pad("sink").expect("queue must have sink pad");
        if queue_sink.is_linked() {
            // already linked, probably another video stream - ignore
            return;
        }

        match src_pad.link(&queue_sink) {
            Ok(_) => {
                println!("Linked decodebin src pad to queue sink");
            }
            Err(err) => {
                eprintln!("Failed to link decodebin pad to queue: {:?}", err);
            }
        }
    });

    let window = gtk::ApplicationWindow::new(app);
    window.set_default_size(640, 480);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);

    let gst_widget = gstgtk4::RenderWidget::new(&gtksink);
    vbox.append(&gst_widget);

    // Label to show the current position (add if needed)
    let label = gtk::Label::new(Some("Position: 00:00:00"));
    // vbox.append(&label);
    
    // Add controls
    let controls = gtk::Grid::builder()
        .row_spacing(6)
        .column_spacing(6)
        .row_homogeneous(true)
        .column_homogeneous(true)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .build();

    let btn_up = gtk::Button::builder()
        .icon_name("go-up-symbolic")
        .tooltip_text("Up")
        .build();
    let btn_left = gtk::Button::builder()
        .icon_name("go-previous-symbolic")
        .tooltip_text("Left")
        .build();
    let btn_right = gtk::Button::builder()
        .icon_name("go-next-symbolic")
        .tooltip_text("Right")
        .build();
    let btn_down = gtk::Button::builder()
        .icon_name("go-down-symbolic")
        .tooltip_text("Down")
        .build();

    // Optional: Click handler
    btn_up.connect_clicked(|_| println!("Arrow: Up"));
    btn_left.connect_clicked(|_| println!("Arrow: Left"));
    btn_right.connect_clicked(|_| println!("Arrow: Right"));
    btn_down.connect_clicked(|_| println!("Arrow: Down"));

    controls.attach(&btn_up,    1, 0, 1, 1);
    controls.attach(&btn_left,  0, 1, 1, 1);
    controls.attach(&btn_right, 2, 1, 1, 1);
    controls.attach(&btn_down,  1, 1, 1, 1);
    let css = r#"
        .arrow {
            transition: 100ms ease-in-out;
        }
        .arrow.active {
            background-color: alpha(@theme_selected_bg_color, 0.4);
            box-shadow: inset 0 0 0 2px @theme_selected_bg_color;
        }
    "#;
    let provider = gtk::CssProvider::new();
    provider.load_from_data(css);
    if let Some(display) = Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
    btn_up.add_css_class("arrow");
    btn_left.add_css_class("arrow");
    btn_right.add_css_class("arrow");
    btn_down.add_css_class("arrow");
    vbox.append(&controls);

    window.set_child(Some(&vbox));
    window.present();
    // Register event handler
    let key_controller = EventControllerKey::new();
    register_key_controller(&key_controller, app.clone(), controls.clone());
    window.add_controller(key_controller);

    app.add_window(&window);

    let pipeline_weak = pipeline.downgrade();
    let timeout_id = glib::timeout_add_local(std::time::Duration::from_millis(500), move || {
        let Some(pipeline) = pipeline_weak.upgrade() else {
            return glib::ControlFlow::Break;
        };

        let position = pipeline.query_position::<gst::ClockTime>();
        label.set_text(&format!("Position: {:.0}", position.display()));
        glib::ControlFlow::Continue
    });

    let bus = pipeline.bus().unwrap();

    pipeline
        .set_state(gst::State::Playing)
        .expect("Unable to set the pipeline to the `Playing` state");

    let app_weak = app.downgrade();
    let bus_watch = bus
        .add_watch_local(move |_, msg| {
            use gst::MessageView;

            let Some(app) = app_weak.upgrade() else {
                return glib::ControlFlow::Break
            };

            match msg.view() {
                MessageView::Eos(..) => app.quit(),
                MessageView::Error(err) => {
                    println!(
                        "Error from {:?}: {} ({:?})",
                        err.src().map(|s| s.path_string()),
                        err.error(),
                        err.debug()
                    );
                    app.quit();
                }
                _ => (),
            };

            glib::ControlFlow::Continue
        })
        .expect("Failed to add bus watch");

    let timeout_id = RefCell::new(Some(timeout_id));
    let pipeline = RefCell::new(Some(pipeline));
    let bus_watch = RefCell::new(Some(bus_watch));
    app.connect_shutdown(move |_| {
        window.close();

        drop(bus_watch.borrow_mut().take());
        if let Some(pipeline) = pipeline.borrow_mut().take() {
            pipeline
                .set_state(gst::State::Null)
                .expect("Unable to set the pipeline to the `Null` state");
        }

        if let Some(timeout_id) = timeout_id.borrow_mut().take() {
            timeout_id.remove();
        }
    });
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