use std::panic;

use wasm_bindgen::prelude::*;
use wee_alloc::WeeAlloc;

// We choose `WeAlloc` for smaller code size in the resulting
// WASM module at the cost of slower performance we shouldn't
// matter too much for our use case.
#[global_allocator]
static ALLOC: WeeAlloc = WeeAlloc::INIT;

#[doc(hidden)]
#[wasm_bindgen(start)]
pub fn _setup_console_error() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello from Rust!");
}
