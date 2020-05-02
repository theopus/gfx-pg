extern crate client;
extern crate wasm_bindgen;

use std::fs::File;
use std::io::Write;

use wasm_bindgen::prelude::*;


#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn go_pidor() {
    client::start();
}
