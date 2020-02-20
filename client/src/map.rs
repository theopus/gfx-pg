#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use rx::assets::Mesh;

const RADIX: u32 = 10;

const MAP: &'static str =
    "11100000110000000000
11100000110000000000
00000000110000000000
11111111111111111100
11111111111111111100
00000000110000000000
00011000110000000000
00011000110000000000";

pub fn generate2d() -> Mesh {
    let step: f32 = 10.;
    let mut z_ptr: f32 = 0.;
    let mut x_ptr: f32 = 0.;

    let mut positions = Vec::new();
    let mut indices = Vec::new();

    for line in MAP.split("\n").collect::<Vec<_>>().iter() {
        info!("{:?}", line);
        for char in line.trim().chars() {
            let value = char.to_digit(RADIX).unwrap();
            if value > 0 {
                let (lu, ld, ru, rd) =
                    (
                        (x_ptr, z_ptr),
                        (x_ptr, z_ptr + step),
                        (x_ptr + step, z_ptr),
                        (x_ptr + step, z_ptr + step),
                    );
                let indices_offset = positions.len() as u32 / 3;
                //lu - 0
                {
                    positions.push(lu.0);
                    positions.push(0.);
                    positions.push(lu.1);
                }
                //ld - 1
                {
                    positions.push(ld.0);
                    positions.push(0.);
                    positions.push(ld.1);
                }
                //ru - 2
                {
                    positions.push(ru.0);
                    positions.push(0.);
                    positions.push(ru.1);
                }
                //lu - 3
                {
                    positions.push(rd.0);
                    positions.push(0.);
                    positions.push(rd.1);
                }

                indices.push(indices_offset + 1);
                indices.push(indices_offset + 0);
                indices.push(indices_offset + 2);
                indices.push(indices_offset + 1);
                indices.push(indices_offset + 2);
                indices.push(indices_offset + 3);
            }

            x_ptr += step;
        }
        z_ptr += step;
        x_ptr = 0.;
    }
    let p_len = positions.len();
    Mesh {
        positions,
        uvs: vec![],
        normals: vec![0_f32; p_len],
        indices,
    }
}