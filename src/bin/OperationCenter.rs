use gstreamer::prelude::{ElementExt, GstBinExt};
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Image};
use gdk4::Paintable;
use gstreamer as gst;
use std::env;
use flume::bounded;

use zenoh::{bytes::Encoding, key_expr::KeyExpr, Config, Session};
use zenoh::bytes::ZBytes;
use zenoh::pubsub::{Publisher, Subscriber};

pub struct Pub<'a> {
    pub topic: String,
    pub publisher: Publisher<'a>,
}

impl<'a> Pub<'a> {
    pub async fn put<V: Into<ZBytes>>(&self, v: V) -> zenoh::Result<()> {
        self.publisher.put(v).await
    }
}

pub struct Sub<H> {
    pub topic: String,
    pub subscriber: Subscriber<H>,
}

impl<H> Sub<H> {
    pub async fn recv_value(&self) -> zenoh::Result<String> {
        let sample = self.subscriber.recv_async().await?;
        let s = sample
            .payload()
            .slices()
            .map(|chunk| String::from_utf8_lossy(chunk).into_owned())
            .collect::<String>();
        Ok(s)
    }
}

const CONFIG: &str =
    r#"{
        "mode": "client",
        "connect": {
            "endpoints": ["tcp/zenoh:7447"],
            "timeout_ms": -1,
            "exit_on_failure": false
        }
    }"#;

async fn init_zenoh() -> zenoh::Result<Session> {
    zenoh::init_log_from_env_or("error");
    let config = Config::from_json5(CONFIG)?;
    
    println!("Opening Zenoh session...");
    zenoh::open(config).await
}

async fn declare_publishers<'a, S: AsRef<str>>(
    session: &'a Session, 
    topics: &[S]
) -> zenoh::Result<Vec<Pub<'a>>> {
    let mut pubs = Vec::with_capacity(topics.len());
    for topic in topics {
        println!("Declaring publisher: {}", topic);
        let key = topic.as_ref().to_owned();
        let p = session.declare_publisher(&key).await?;
        pubs.push(Pub { topic: key, publisher: p});
    }
    Ok(pubs)
}

async fn declare_subscribers<S: AsRef<str>, H>(
    session: &Session,
    topics: &[S],
) -> zenoh::Result<Vec<Sub<H>>> {
    let mut subs = Vec::with_capacity(topics.len());
    for topic in topics {
        println!("Declaring subscriber: {}", topic);
        let key = topic.as_ref().to_owned();
        let s = session.
            declare_subscriber(&key)
            .with(bounded(32))
            .await?;
        subs.push(Sub { topic: key, subscriber: s, });
    }
    Ok(subs)
}

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
#[tokio::main]
async fn main() {
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

    app.connect_activate(|app| {
        create_ui(app);
    });

    // Run the app
    // NOTE: pass command-line args so GTK can parse them
    let args: Vec<String> = env::args().collect();
    app.run_with_args(&args);

    let session = init_zenoh().await.unwrap();
    let topics = ["Vehicle/Teleop/EnginePower",
        "Vehicle/Teleop/SteeringAngle",
        "Vehicle/Teleop/ControlCounter",
        "Vehicle/Teleop/ControlTimestamp_ms",
    ];

    let publishers = declare_publishers(&session, &topics).await.unwrap();

    if let Some(topic) = publishers
        .iter()
        .find(|topic| topic.topic == "Vehicle/Teleop/EnginePower")
    {
        topic.put("42").await.unwrap();
    }

    let subscribers = declare_subscribers(&session, &["Vehicle/Speed"]).await.unwrap();

    if let Some(s) = subscribers
        .iter()
        .find(|s| s.topic == "Vehicle/Speed")
    {
        s.recv_value().await.unwrap();
    }
}

// // sender
// // gst-launch-1.0 -v libcamerasrc ! x264enc tune=zerolatency speed-preset=ultrafast ! rtph264pay pt=96 ! udpsink host=<PC_IP> port=5000
// // receiver
// // gst-launch-1.0 -v udpsrc port=5000 caps="application/x-rtp,media=video,encoding-name=H264,payload=96" ! rtph264depay ! decodebin ! autovideosink