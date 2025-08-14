# Visualizing GStreamer Video Frames in Rust

If you want your Rust application to **receive video frames from GStreamer and display them on your laptop**, you essentially need two parts:  

1. **GStreamer** to handle decoding and frame delivery.  
2. **A Rust GUI or rendering library** to visualize the frames.  

---

## 1️⃣ Recommended Rust libraries for displaying video frames

| Library / Framework | Pros | Cons |
|---------------------|------|------|
| **[eframe / egui](https://github.com/emilk/egui)** | Easy to use, pure Rust, cross-platform, integrates well with GStreamer `appsink`. | Not GPU-optimized for heavy video playback, but fine for moderate FPS. |
| **[winit + pixels](https://github.com/parasyte/pixels)** | Very lightweight, direct pixel buffer rendering. Good for RGB frames from GStreamer. | No UI widgets, you must build your own controls. |
| **[wgpu](https://github.com/gfx-rs/wgpu)** | High-performance GPU rendering, cross-platform, modern API. | More complex for beginners. |
| **[SDL2 for Rust](https://github.com/Rust-SDL2/rust-sdl2)** | Mature, battle-tested, handles window & rendering well. | C library dependency, slightly less "Rust-native". |
| **[iced](https://github.com/iced-rs/iced)** | GUI framework with image display support. | Slightly heavier and less low-level control over frame rendering. |

> For quick visualization of GStreamer frames,  
> **`egui`** is good for UI + video display,  
> **`pixels`** is better for pure, fast framebuffer playback.

---

## 2️⃣ Minimal Example: GStreamer + egui

### Cargo.toml
```toml
[dependencies]
gstreamer = "0.22"
gstreamer-app = "0.22"
eframe = "0.27"
image = "0.25" # for RGB to egui image handling
```

### main.rs
```rust
use eframe::{egui, epi};
use gstreamer as gst;
use gstreamer_app as gst_app;
use gst::prelude::*;
use std::sync::{Arc, Mutex};

struct VideoApp {
    latest_frame: Arc<Mutex<Option<egui::ColorImage>>>,
}

impl epi::App for VideoApp {
    fn name(&self) -> &str {
        "GStreamer Video Viewer"
    }

    fn update(&mut self, ctx: &egui::Context, _: &mut epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(frame) = &*self.latest_frame.lock().unwrap() {
                let texture = ui.ctx().load_texture(
                    "video_frame",
                    frame.clone(),
                    egui::TextureOptions::default(),
                );
                ui.image(&texture, texture.size_vec2());
            }
        });
        ctx.request_repaint(); // keep redrawing
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    gst::init()?;

    let latest_frame = Arc::new(Mutex::new(None));
    let frame_clone = latest_frame.clone();

    // GStreamer pipeline: Replace with your actual source
    let pipeline_str = "\
        videotestsrc ! \        videoconvert ! \        video/x-raw,format=RGB ! \        appsink name=sink sync=false";

    let pipeline = gst::parse_launch(pipeline_str)?
        .downcast::<gst::Pipeline>()
        .unwrap();

    let appsink = pipeline.by_name("sink").unwrap()
        .downcast::<gst_app::AppSink>()
        .unwrap();

    appsink.set_callbacks(
        gst_app::AppSinkCallbacks::builder()
            .new_sample(move |sink| {
                let sample = sink.pull_sample().unwrap();
                let buffer = sample.buffer().unwrap();
                let map = buffer.map_readable().unwrap();

                let caps = sample.caps().unwrap();
                let s = caps.structure(0).unwrap();
                let width = s.get::<i32>("width").unwrap() as usize;
                let height = s.get::<i32>("height").unwrap() as usize;

                let pixels = map.as_slice().to_vec();
                let image = egui::ColorImage::from_rgb([width, height], &pixels);

                *frame_clone.lock().unwrap() = Some(image);

                Ok(gst::FlowSuccess::Ok)
            })
            .build(),
    );

    pipeline.set_state(gst::State::Playing)?;

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "GStreamer Video Viewer",
        native_options,
        Box::new(|_cc| Box::new(VideoApp { latest_frame })),
    );

    pipeline.set_state(gst::State::Null)?;
    Ok(())
}
```

---

## 3️⃣ Why this approach is nice
- You **stay in Rust** — no subprocess parsing.
- GStreamer handles all network/codec complexity.
- egui gives you a resizable, interactive window.
- You can easily add overlays, controls, or real-time statistics.

---

## 4️⃣ When to choose another rendering option
If you want:
- **High FPS fullscreen playback** → use **winit + pixels**.  
- **GPU-accelerated rendering** → use **wgpu**.  
- **Classic game-style windowing** → use **SDL2**.  
