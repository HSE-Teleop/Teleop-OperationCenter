use std::cell::OnceCell;
use std::sync::Arc;
use gdk4::{Key, ModifierType};
use gdk4::prelude::*;
use glib::{MainContext, Propagation};
use gtk4::{Application, Button, EventControllerKey, Grid};
use gtk4::prelude::{GridExt, WidgetExt};
use crate::utils::zenoh_utils::{Pub, Sub};

thread_local! {
    static EVENT_PARAMETERS: OnceCell<EventParameters> = OnceCell::new();
}
#[derive(Debug)]
struct EventParameters {
    pub app: Application,
    pub controls: Grid,
    // zenoh parameters
    pub publishers: Arc<Vec<Pub<'static>>>, 
    pub subscribers: Arc<Vec<Sub>>
    // Add more parameters here if needed
}
impl EventParameters {
    pub fn new(app: Application, controls: Grid, publishers: Arc<Vec<Pub<'static>>>, subscribers: Arc<Vec<Sub>>) -> EventParameters {
        println!("Registered EventParameters!");
        EventParameters {
            app,
            controls, 
            publishers,
            subscribers
        }
    }
}

/// Implement functionality and pass extra arguments here to abstract the code.
pub fn register_key_controller<'a>(key_controller: &'a EventControllerKey, app: Application, controls: Grid, publishers: Arc<Vec<Pub<'static>>>, subscribers: Arc<Vec<Sub>>) -> &'a EventControllerKey {
    EVENT_PARAMETERS.with(|cell| {
        let _ = cell.set(EventParameters::new(
            app,
            controls,
            publishers,
            subscribers
        ));
    });

    key_controller.connect_key_pressed(event_controller);
    key_controller.connect_key_released(release_event_controller);

    println!("Registered event controller!");
    key_controller
}

fn event_controller(_: &EventControllerKey, key: Key, _id: u32, _modifier_type: ModifierType) -> Propagation {
    match key {
        Key::Escape => {
            escape_handler()
        }

        Key::space => {
            space_handler()
        }

        Key::w | Key::Up => {
            forward_handler(false)
        }

        Key::a | Key::Left => {
            left_handler(false)
        }

        Key::s | Key::Down => {
            backwards_handler(false)
        }

        Key::d | Key::Right => {
            right_handler(false)
        }

        _ => Propagation::Proceed,
    }
}
fn release_event_controller(_: &EventControllerKey, key: Key, _id: u32, _modifier_type: ModifierType) {
    match key {
         Key::w | Key::Up => {
            forward_handler(true);
        }

        Key::a | Key::Left => {
            left_handler(true);
        }

        Key::s | Key::Down => {
            backwards_handler(true);
        }

        Key::d | Key::Right => {
            right_handler(true);
        }

        _ => return,
    }
}

fn escape_handler() -> Propagation {
    let handled = EVENT_PARAMETERS.with(|cell| {
        if let Some(params) = cell.get() {
            // optional: quit app on Escape
            params.app.quit();
            true
        } else {
            false
        }
    });
    if handled {
        println!("Quit app");
        Propagation::Stop
    } else {
        eprintln!("EventParameters not initialized");
        Propagation::Proceed
    }
}
fn space_handler() -> Propagation {
    // rotate video (180deg?)
    println!("Rotate video");
    Propagation::Stop
}
fn forward_handler(released: bool) -> Propagation {
    let handled = EVENT_PARAMETERS.with(|cell| {
        if let Some(params) = cell.get() {
            println!("Move forward");
            let btn: Button = params.controls.child_at(1, 0).unwrap().downcast().expect("Downcast to button failed");
            if released {
                remove_active_class(&btn);
                // TODO: 0.1 steps while holding and -0.1 steps when released
                let pubs = Arc::clone(&params.publishers);
                MainContext::default().spawn_local(async move {
                    if let Some(p) = pubs.iter().find(|p| p.topic == "Vehicle/Teleop/EnginePower") {
                        if let Err(e) = p.put("0").await {
                            eprintln!("put failed: {e}");
                        }
                    }
                });
            }else{
                remove_active_class(&btn);
                add_active_class(&btn);
                // Configure zenoh publisher here
                let pubs = Arc::clone(&params.publishers);
                MainContext::default().spawn_local(async move {
                    if let Some(p) = pubs.iter().find(|p| p.topic == "Vehicle/Teleop/EnginePower") {
                        if let Err(e) = p.put("10").await {
                            eprintln!("put failed: {e}");
                        }
                    }
                });
            }
            true
        } else {
            false
        }
    });
    if handled {
        Propagation::Stop
    } else {
        eprintln!("EventParameters not initialized");
        Propagation::Proceed
    }
}
fn left_handler(released: bool) -> Propagation {
    let handled = EVENT_PARAMETERS.with(|cell| {
        if let Some(params) = cell.get() {
            println!("Move left");
            let btn: Button = params.controls.child_at(0, 1).unwrap().downcast().expect("Downcast to button failed");
            if released {
                remove_active_class(&btn);
            }else{
                remove_active_class(&btn);
                add_active_class(&btn);
            }
            true
        } else {
            false
        }
    });
    if handled {
        Propagation::Stop
    } else {
        eprintln!("EventParameters not initialized");
        Propagation::Proceed
    }
}
fn backwards_handler(released: bool) -> Propagation {
    let handled = EVENT_PARAMETERS.with(|cell| {
        if let Some(params) = cell.get() {
            println!("Move backwards");
            let btn: Button = params.controls.child_at(1, 1).unwrap().downcast().expect("Downcast to button failed");
            if released {
                remove_active_class(&btn);
            }else{
                remove_active_class(&btn);
                add_active_class(&btn);
            }
            true
        } else {
            false
        }
    });
    if handled {
        Propagation::Stop
    } else {
        eprintln!("EventParameters not initialized");
        Propagation::Proceed
    }
}
fn right_handler(released: bool) -> Propagation {
    let handled = EVENT_PARAMETERS.with(|cell| {
        if let Some(params) = cell.get() {
            println!("Move right");
            let btn: Button = params.controls.child_at(2, 1).unwrap().downcast().expect("Downcast to button failed");
            if released {
                remove_active_class(&btn);
            }else{
                remove_active_class(&btn);
                add_active_class(&btn);
            }
            true
        } else {
            false
        }
    });
    if handled {
        Propagation::Stop
    } else {
        eprintln!("EventParameters not initialized");
        Propagation::Proceed
    }
}

fn add_active_class(btn: &Button) {
    btn.add_css_class("active");
}
fn remove_active_class(btn: &Button) {
    btn.remove_css_class("active");
}