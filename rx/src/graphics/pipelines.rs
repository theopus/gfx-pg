use hal::pso::{
    AttributeDesc,
    Element,
};

struct D3Pipeline {}

trait Vertex {
    fn attributes() -> Vec<AttributeDesc>;
}

impl D3Pipeline {
    pub fn new<V: Vertex>() -> Result<Self, &'static str> {
        let attrs = V::attributes();
        Ok(D3Pipeline {})
    }
}


mod shader {
    use log::error;
    use shaderc::CompilationArtifact;
    use shaderc::Compiler;
    use shaderc::ShaderKind;

    pub fn compile(
        source: &str,
        kind: shaderc::ShaderKind,
        name: &str,
        entry_point: &str,
    ) -> Result<CompilationArtifact, &'static str> {
        Compiler::new()
            .ok_or("shaderc not found!")?
            .compile_into_spirv(
                source,
                kind,
                name,
                entry_point,
                None,
            )
            .map_err(|e| {
                error!("{}", e);
                "Couldn't compile vertex shader!"
            })
    }
}

