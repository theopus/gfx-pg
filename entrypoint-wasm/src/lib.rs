extern crate wasm_bindgen;
extern crate client;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn start() {
    client::start();
}
