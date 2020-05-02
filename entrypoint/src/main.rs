extern crate client;

use std::fs::File;
use std::io::Write;

use client::rx::shaderc::ShaderKind;

pub const VERTEX_SOURCE: &'static str = include_str!("../../shaders/one.vert");

pub const FRAGMENT_SOURCE: &'static str = include_str!("../../shaders/one.frag");

fn main() {
    let v = client::rx::graphics::pipelines::shader::compile(include_str!("../../shaders/one.vert"), client::rx::shaderc::ShaderKind::Vertex, "one.vert", "main").expect("");
    let f = client::rx::graphics::pipelines::shader::compile(include_str!("../../shaders/one.frag"), client::rx::shaderc::ShaderKind::Fragment, "one.frag", "main").expect("");

    {
        let mut v_file = File::create("assets/one.vert.spv").expect("");
        v_file.write_all(v.as_binary_u8()).expect("");
    };

    {
        let mut f_file = File::create("assets/one.frag.spv").expect("");
        f_file.write_all(f.as_binary_u8()).expect("");
    };

    client::start();
}
