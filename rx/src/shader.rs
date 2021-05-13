use shaderc;
use anyhow::*;
use std::path::PathBuf;
use std::ffi::OsStr;
use std::fs::{read_to_string, write};

pub struct SpirvCompiler {
    compiler: shaderc::Compiler
}

impl SpirvCompiler {
    pub fn new() -> anyhow::Result<Self> {
        Ok(SpirvCompiler { compiler: shaderc::Compiler::new().context("Unable to create shader compiler")? })
    }


}

impl SpirvCompiler {
    pub fn compile_to_fs(&mut self, from: PathBuf) -> Result<PathBuf> {
        let extension = from
            .extension()
            .context("File has no extension")?
            .to_str()
            .context("Extension cannot be converted to &str")?;
        let kind = match extension {
            "vert" => shaderc::ShaderKind::Vertex,
            "frag" => shaderc::ShaderKind::Fragment,
            "comp" => shaderc::ShaderKind::Compute,
            _ => bail!("Unsupported shader: {}", from.display()),
        };

        let src = read_to_string(from.clone())?;
        let spv_path = from.with_extension(format!("{}.spv", extension));

        let compiled = self.compiler.compile_into_spirv(
            &src,
            kind,
            &spv_path.to_str().unwrap(),
            "main",
            None,
        )?;
        write(spv_path.clone(), compiled.as_binary_u8())?;
        Ok(spv_path)
    }
}