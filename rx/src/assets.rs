use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Error, Seek};
use std::path::PathBuf;

use image::RgbaImage;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

pub struct AssetsLoader {
    dir: PathBuf
}

impl AssetsLoader {
    const IMAGE_DIR: &'static str = "images";
    const MODEL_DIR: &'static str = "models";

    pub fn new(dir: &'static str) -> Result<Self, &str> {
        let dir = PathBuf::from(dir)
            .canonicalize()
            .map_err(|e| {
                error!("{:?}", e);
                "Error with assets loader"
            })?;
        info!("Assets location {:?}", dir);
        Ok(AssetsLoader { dir })
    }

    fn open_file(&self,
                 name: &'static str,
                 dir: &'static str,
                 ext: &'static str,
    ) -> Result<(BufReader<File>, PathBuf), &'static str> {
        let mut file_name = self.dir.
            as_path()
            .join(dir)
            .join(name);
        file_name.set_extension(ext);

        let file = File::open(file_name.clone())
            .map_err(|e| {
                error!("File not found: {:?}, err: {:?}", file_name, e);
                "Error opening file file"
            })?;
        Ok((BufReader::new(file), file_name))
    }

    pub fn load_img(&self, name: &'static str) -> Result<RgbaImage, &str> {
        let (buffer, file_name) = self.open_file(name, Self::IMAGE_DIR, "png")?;
        let image = image::load(buffer, image::PNG)
            .map_err(|e| {
                error!("{:?}", e);
                "Error with loading img"
            })?
            .to_rgba();
        info!("Loaded image: {:?}", file_name);
        Ok(image)
    }

    pub fn load_obj(&mut self, name: &'static str) -> Result<Mesh, &str> {
        let (mut buffer, file_name) = self.open_file(name, Self::MODEL_DIR, "obj")?;
        let (mut models, materials) = tobj::load_obj_buf(&mut buffer, |p| -> tobj::MTLLoadResult {
            Ok((Vec::new(), HashMap::new()))
        }).map_err(|e| {
            error!("{:?}", e);
            "Error with loading obj mesh"
        })?;
        let tobj::Mesh {
            positions,
            normals,
            texcoords,
            indices,
            ..
        } = models.pop().unwrap().mesh;
        info!("Loaded obj: {:?}", file_name);
        Ok(Mesh {
            positions,
            uvs: texcoords,
            normals,
            indices,
        })
    }
}

pub struct Mesh {
    pub positions: Vec<f32>,
    pub uvs: Vec<f32>,
    pub normals: Vec<f32>,
    pub indices: Vec<u32>,
}