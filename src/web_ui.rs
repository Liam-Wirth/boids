use wasm_bindgen::prelude::*;
use web_sys::{Window, Document, HtmlElement};

#[wasm_bindgen]
pub fn setup_ui() {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");

    setup_predator_toggle(&document);
    // Add more UI setup functions as needed
}

fn setup_predator_toggle(document: &Document) {
    let button = document
        .get_element_by_id("toggle-predator")
        .expect("should have toggle-predator button")
        .dyn_into::<HtmlElement>()
        .expect("should be an HtmlElement");

    let closure = Closure::wrap(Box::new(move || {
        // This function will be called when the button is clicked
        // You'll need to figure out how to communicate this to your Bevy app
        // One way is to use a global static variable:
        unsafe {
            PREDATOR_MODE = !PREDATOR_MODE;
        }
    }) as Box<dyn FnMut()>);

    button.set_onclick(Some(closure.as_ref().unchecked_ref()));
    closure.forget(); // Leaks memory, but ensures closure is valid for the lifetime of the program
}

// Global static to communicate between WASM and Bevy
static mut PREDATOR_MODE: bool = false;

#[wasm_bindgen]
pub fn is_predator_mode() -> bool {
    unsafe { PREDATOR_MODE }
}
