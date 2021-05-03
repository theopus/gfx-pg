use core::ptr;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::mem::size_of;
use std::ops::{Deref, Range};
use std::path::PathBuf;

use futures::executor::block_on;
use hal::Backend;
use image::RgbaImage;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use crate::graphics_api;
use crate::graphics_api::v0::Vertex;
use crate::wgpu_graphics::memory::MemoryManager;
use crate::wgpu_graphics::State;

#[derive(Debug, Clone)]
pub struct AssetsStorage {
    mesh_offset: i32,
    idx_offset: u32,
}

#[derive(Debug, Clone)]
pub struct MeshPtr {
    pub(crate) indices: Range<u32>,
    pub(crate) base_vertex: i32,
}

impl PartialEq<Self> for MeshPtr {
    fn eq(&self, other: &MeshPtr) -> bool {
        self.base_vertex == other.base_vertex
    }
}

impl AssetsStorage {
    pub fn new() -> Result<Self, &'static str> {
        Ok(Self {
            mesh_offset: 0,
            idx_offset: 0,
        })
    }

    pub fn load_mesh(
        &mut self,
        api: &mut State,
        mesh: Mesh,
    ) -> Result<MeshPtr, &'static str> {
        unsafe {
            let Mesh {
                positions,
                mut uvs,
                normals,
                indices,
            } = mesh;

            if uvs.len() == 0 {
                uvs = vec![0_f32; positions.len() / 3 * 2];
            }

            let flatten_mesh_vec = positions
                .chunks_exact(3)
                .zip(uvs.chunks_exact(2))
                .zip(normals.chunks_exact(3))
                .flat_map(|((p, uv), n): ((&[f32], &[f32]), &[f32])| [p, uv, n].concat())
                .collect::<Vec<f32>>();
            let flatten_mesh = flatten_mesh_vec.as_slice();

            assert_eq!(positions.len() / 3, uvs.len() / 2);
            assert_eq!(positions.len() / 3, normals.len() / 3);
            {
                let mesh_len = flatten_mesh_vec.len() * size_of::<f32>();
                //hardcoded for vertex 8 (3 + 2 + 3)
                let offset = self.mesh_offset as usize * size_of::<f32>() * 8;
                let align = 0;
                // let align = offset % 64;
                let mut range = ((offset - align) as u64..((offset - align) + mesh_len) as u64);
                api.queue.write_buffer(
                    &api.memory_manager.mesh_buffer,
                    range.start,
                    unsafe { std::slice::from_raw_parts(flatten_mesh.as_ptr() as *const u8, mesh_len) },
                );
                info!("mesh len {:?}", mesh_len);
                info!("mesh range {:?}", range);
                // [WARN] AM I SANE WGPU MAP IS FUCKED UP
                // range = range.start / 2..range.end / 2;
                // let slice = api.memory_manager.mesh_buffer.slice(range);
                // let map_flag = slice.map_async(wgpu::MapMode::Write);
                // api.device.poll(wgpu::Maintain::Wait);
                // block_on(map_flag).unwrap();
                // let mesh_ptr = slice.get_mapped_range_mut().as_mut_ptr();
                // ptr::copy(
                //     flatten_mesh.as_ptr() as *const u8,
                //     mesh_ptr.offset(align as isize),
                //     mesh_len,
                // );
                // api.memory_manager.mesh_buffer.unmap();
            }
            {
                let idx_len = indices.len() * size_of::<u32>();
                let offset = self.idx_offset * size_of::<u32>() as u32;
                let align = 0;
                let mut range = (offset - align) as u64..((offset - align) + idx_len as u32) as u64;
                info!("idx len {:?}", idx_len);
                info!("idx range {:?}", range);
                api.queue.write_buffer(
                    &api.memory_manager.idx_buffer,
                    range.start,
                    unsafe { std::slice::from_raw_parts(indices.as_ptr() as *const u8, indices.len() * 4) },
                );
                //[WARN] AM I SANE
                // range = range.start / 2..range.end / 2;
                // let slice = api.memory_manager.idx_buffer.slice(range);
                // let map_flag = slice.map_async(wgpu::MapMode::Write);
                // api.device.poll(wgpu::Maintain::Wait);
                // block_on(map_flag).unwrap();
                // let idx_ptr = slice.get_mapped_range_mut().as_mut_ptr();
                // ptr::copy(
                //     indices.as_slice().as_ptr() as *const u8,
                //     idx_ptr.offset(align as isize),
                //     idx_len,
                // );
                // api.memory_manager.idx_buffer.unmap();
            }

            let mesh_ptr = MeshPtr {
                indices: self.idx_offset..(self.idx_offset + indices.len() as u32),
                base_vertex: self.mesh_offset,
            };

            info!("mesh_ptr {:?}", mesh_ptr);
            self.mesh_offset += (positions.len() / 3) as i32;
            self.idx_offset += indices.len() as u32;
            info!("mesh_offset{:?}", self.mesh_offset);
            info!("idx_offset{:?}", self.idx_offset);

            Ok(mesh_ptr)
        }
    }
}

//fn align_to(value: u32, alignment: u32) -> u32 {
//    let diff = value % alignment;
//    if diff == 0 {
//        return value;
//    }
//    return value - diff + alignment
//}

pub struct AssetsLoader {
    dir: PathBuf,
}

impl AssetsLoader {
    const IMAGE_DIR: &'static str = "images";
    const MODEL_DIR: &'static str = "models";

    pub fn new(dir: PathBuf) -> Result<Self, &'static str> {
        // let dir = PathBuf::from(dir).canonicalize().map_err(|e| {
        //     error!("{:?}", e);
        //     "Error with assets loader"
        // })?;
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
        let (mut models, _) = tobj::load_obj_buf(&mut buffer, |_| -> tobj::MTLLoadResult {
            Ok((Vec::new(), HashMap::new()))
        })
            .map_err(|e| {
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

#[derive(Debug)]
pub struct Mesh {
    pub positions: Vec<f32>,
    pub uvs: Vec<f32>,
    pub normals: Vec<f32>,
    pub indices: Vec<u32>,
}
