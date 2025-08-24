use std::cell::OnceCell;
use gdk4::{Key, ModifierType};
use gdk4::prelude::*;
use glib::Propagation;
use gtk4::{Application, Button, EventControllerKey, Grid};
use gtk4::prelude::{GridExt, WidgetExt};

thread_local! {
    static EVENT_PARAMETERS: OnceCell<EventParameters> = OnceCell::new();
}
#[derive(Debug)]
struct EventParameters {
    pub app: Application,
    pub controls: Grid,
    // Add more parameters here if needed
}
impl EventParameters {
    pub fn new(app: Application, controls: Grid) -> EventParameters {
        println!("Registered EventParameters!");
        EventParameters {
            app,
            controls,
        }
    }
}

/// Implement functionality and pass extra arguments here to abstract the code.
pub fn register_key_controller(key_controller: &EventControllerKey, app: Application, controls: Grid) -> &EventControllerKey {
    EVENT_PARAMETERS.with(|cell| {
        let _ = cell.set(EventParameters::new(
            app.clone(),
            controls.clone()
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
            // TODO: forward movement
            println!("Move forward");
            let btn: Button = params.controls.child_at(1, 0).unwrap().downcast().expect("Downcast to button failed");
            if released {remove_active_class(&btn);}else{add_active_class(&btn);}
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
            // TODO: left movement
            println!("Move left");
            let btn: Button = params.controls.child_at(0, 1).unwrap().downcast().expect("Downcast to button failed");
            if released {remove_active_class(&btn);}else{add_active_class(&btn);}
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
            // TODO: backwards movement
            println!("Move backwards");
            let btn: Button = params.controls.child_at(1, 1).unwrap().downcast().expect("Downcast to button failed");
            if released {remove_active_class(&btn);}else{add_active_class(&btn);}
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
            // TODO: right movement
            println!("Move right");
            let btn: Button = params.controls.child_at(2, 1).unwrap().downcast().expect("Downcast to button failed");
            if released {remove_active_class(&btn);}else{add_active_class(&btn);}
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