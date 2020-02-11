use core::ptr;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Error, Seek};
use std::mem::size_of;
use std::ops::{Deref, Range};
use std::path::PathBuf;

use hal::adapter::Adapter;
use hal::Backend;
use hal::device::Device;
use image::RgbaImage;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use crate::graphics::wrapper::ApiWrapper;

#[derive(Debug, Clone)]
pub struct AssetsStorage {
    mesh_offset: i32,
    idx_offset: u32,
}

#[derive(Debug, Clone)]
pub struct MeshPtr {
    pub(crate)indices: Range<u32>,
    pub(crate)base_vertex: i32,
}


impl AssetsStorage {
    pub fn new() -> Result<Self, &'static str> {
        Ok(Self { mesh_offset: 0, idx_offset: 0 })
    }

    pub fn load_mesh<B: Backend>(&mut self,
                                 wrapper: &ApiWrapper<B>,
                                 mesh: Mesh,
    ) -> Result<MeshPtr, &'static str> {
        unsafe {
            let Mesh { positions, mut uvs, normals, indices } = mesh;
            let device: &B::Device = &wrapper.hal_state.device.deref();

            if uvs.len() == 0 {
                uvs = vec![0_f32; positions.len() / 3 * 2];
            }

            let flatten_mesh_vec = positions.chunks_exact(3)
                .zip(uvs.chunks_exact(2))
                .zip(normals.chunks_exact(3))
                .flat_map(|((p, uv), n): ((&[f32], &[f32]), &[f32])| {
                    [p, uv, n].concat()
                }).collect::<Vec<f32>>();
            let flatten_mesh = flatten_mesh_vec.as_slice();

            assert_eq!(positions.len() / 3, uvs.len() / 2);
            assert_eq!(positions.len() / 3, normals.len() / 3);
            {
                let mesh_len = flatten_mesh_vec.len() * size_of::<f32>();
                let bundle = &wrapper.storage.mesh_bundle;
                //hardcoded for vertex 8 (3 + 2 + 3)
                let offset = (self.mesh_offset * size_of::<f32>() as i32 * 8);
                let align = offset % 64;
                let range = (offset - align) as u64..bundle.requirements().size;
                let mesh_ptr = bundle.map_mem_range(device, range.clone())?;
                ptr::copy(
                    flatten_mesh.as_ptr() as *const u8,
                    mesh_ptr.offset(align as isize),
                    mesh_len,
                );
                bundle.flush_mem_range(device, range)?;
                bundle.unmap(device)?;
            }
            {
                let idx_len = indices.len() * size_of::<u32>();
                let bundle = &wrapper.storage.idx_bundle;
                let offset = (self.idx_offset * size_of::<u32>() as u32);
                let align = offset % 64;
                let range = (offset - align) as u64..bundle.requirements().size;
                let idx_ptr = bundle.map_mem_range(device, range.clone())?;
                ptr::copy(
                    indices.as_slice().as_ptr() as *const u8,
                    idx_ptr.offset(align as isize),
                    idx_len,
                );
                bundle.flush_mem_range(device, range)?;
                bundle.unmap(device)?;
            }

            let mesh_ptr = MeshPtr {
                indices: self.idx_offset..(self.idx_offset + indices.len() as u32),
                base_vertex: self.mesh_offset,
            };
            self.mesh_offset += (positions.len() / 3) as i32;
            self.idx_offset += indices.len() as u32;
            info!("mesh_offset{:?}", self.mesh_offset);
            info!("idx_offset{:?}", self.idx_offset);
            Ok(mesh_ptr)
        }
    }
}

fn align_to(value: u32, alignment: u32) -> u32 {
    let diff = value % alignment;
    if diff == 0 {
        return value;
    }
    return value - diff + alignment
}

pub struct AssetsLoader {
    dir: PathBuf,
}

impl AssetsLoader {
    const IMAGE_DIR: &'static str = "images";
    const MODEL_DIR: &'static str = "models";

    pub fn new(dir: &'static str) -> Result<Self, &str> {
        let dir = PathBuf::from(dir).canonicalize().map_err(|e| {
            error!("{:?}", e);
            "Error with assets loader"
        })?;
        info!("Assets location {:?}", dir);
        Ok(AssetsLoader { dir })
    }

    fn open_file(
        &self,
        name: &'static str,
        dir: &'static str,
        ext: &'static str,
    ) -> Result<(BufReader<File>, PathBuf), &'static str> {
        let mut file_name = self.dir.as_path().join(dir).join(name);
        file_name.set_extension(ext);

        let file = File::open(file_name.clone()).map_err(|e| {
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

    pub fn load_obj(&self, name: &'static str) -> Result<Mesh, &'static str> {
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
