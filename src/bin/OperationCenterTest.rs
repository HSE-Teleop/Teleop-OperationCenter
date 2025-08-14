use eframe::{egui, App, Frame, NativeOptions};
use gstreamer as gst;
use gstreamer_app as gst_app;
use gstreamer_video as gst_video;

use std::sync::{Arc, Mutex};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // GStreamer initialisieren
    gst::init()?;

    // Gemeinsamer Speicher für das aktuelle Videobild
    let shared_frame: Arc<Mutex<Option<egui::ColorImage>>> = Arc::new(Mutex::new(None));

    // Pipeline parsen und zu gst::Pipeline downcasten
    let pipeline = gst::parse_launch(
        "videotestsrc pattern=smpte95 \
         ! videoconvert \
         ! videoscale \
         ! video/x-raw,format=RGBA \
         ! appsink name=sink",
    )?
        .downcast::<gst::Pipeline>()?;

    // appsink referenzieren und zu gst_app::AppSink casten
    let appsink = pipeline
        .by_name("sink")
        .expect("appsink not found")
        .downcast::<gst_app::AppSink>()
        .expect("Element ist keine AppSink");

    // Closure erhält Zugriff auf den gemeinsamen Bildpuffer
    let frame_clone = shared_frame.clone();
    appsink.set_callbacks(
        gst_app::AppSinkCallbacks::builder()
            .new_sample(move |sink| {
                // Rohdaten holen
                let sample = sink.pull_sample().map_err(|_| gst::FlowError::Eos)?;
                let buffer = sample.buffer().ok_or(gst::FlowError::Error)?;
                let map = buffer.map_readable().map_err(|_| gst::FlowError::Error)?;

                // Bildgröße ermitteln
                let info = gst_video::VideoInfo::from_caps(sample.caps().unwrap()).unwrap();
                let (w, h) = (info.width() as usize, info.height() as usize);

                // In egui-Bild umwandeln
                let img = egui::ColorImage::from_rgba_unmultiplied([w, h], map.as_slice());

                // Im Shared State ablegen
                *frame_clone.lock().unwrap() = Some(img);
                Ok(gst::FlowSuccess::Ok)
            })
            .build(),
    );

    // Pipeline starten
    pipeline.set_state(gst::State::Playing)?;

    // egui-Fenster starten
    let app = VideoApp {
        texture: None,
        frame_store: shared_frame,
    };
    eframe::run_native(
        "Teleop Control Center",
        NativeOptions::default(),
        Box::new(|_| Box::new(app)),
    )?;

    // Aufräumen
    pipeline.set_state(gst::State::Null)?;
    Ok(())
}

struct VideoApp {
    texture: Option<egui::TextureHandle>,
    frame_store: Arc<Mutex<Option<egui::ColorImage>>>,
}

impl App for VideoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // Neues Frame übernehmen
        if let Some(img) = self.frame_store.lock().unwrap().take() {
            match &self.texture {
                Some(t) => t.set(img, egui::TextureOptions::NEAREST),
                None => {
                    self.texture = Some(
                        ctx.load_texture("gst_frame", img, egui::TextureOptions::NEAREST),
                    );
                }
            }
        }

        // Zeichnen
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(tex) = &self.texture {
                ui.image(tex.id(), tex.size_vec2());
            } else {
                ui.label("Kein Videostream …");
            }
        });
    }
}