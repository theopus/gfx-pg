extern crate client;

use std::fs::File;
use std::io::Write;

use client::rx::shaderc::ShaderKind;

pub const VERTEX_SOURCE: &'static str = include_str!("../../shaders/one.vert");

pub const FRAGMENT_SOURCE: &'static str = include_str!("../../shaders/one.frag");


fn main() {
//    let v = client::rx::graphics::pipelines::shader::compile(VERTEX_SOURCE, ShaderKind::Vertex, "one", "main").expect("");
//    let f = client::rx::graphics::pipelines::shader::compile(FRAGMENT_SOURCE, ShaderKind::Fragment, "one", "main").expect("");
//
//    let mut v_file = File::create("vertex.spr").expect("");
//    v_file.write_all(v.as_binary_u8());
//    let mut f_file = File::create("frag.spr").expect("");
//    f_file.write_all(f.as_binary_u8());

    client::start();
}
