use std::mem::size_of;

use glm::{
    Mat4,
    Vec3,
};
use nalgebra::Matrix4;

use crate::window::winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};

pub type Vertex2D = [f32; 2];
pub type TriangleVertex = [f32; (2 + 3)];

#[derive(Debug, Clone, Copy)]
pub struct Triangle {
    pub points: [[f32; 2]; 3],
    pub colors: [[f32; 3]; 3],
}

impl Triangle {
    pub fn points_flat(self) -> [f32; 6] {
        let [[a, b], [c, d], [e, f]] = self.points;
        [a, b, c, d, e, f]
    }
    pub fn vertex_attribs(self) -> [f32; 3 * (2 + 3)] {
        let [[a, b], [c, d], [e, f]] = self.points;
        let [
        [r0, g0, b0],
        [r1, g1, b1],
        [r2, g2, b2]
        ] = self.colors;
        [
            a, b, r0, g0, b0, // red
            c, d, r1, g1, b1, // green
            e, f, r2, g2, b2, // blue
        ]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Quad {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}


impl Quad {
    pub fn vertex_attributes(self) -> [f32; 4 * (2 + 3 + 2)] {
        let x = self.x;
        let y = self.y;
        let w = self.w;
        let h = self.h;
            #[cfg_attr(rustfmt, rustfmt_skip)]
        [
            // X    Y    R    G    B                  U    V
            x, y + h, 1.0, 0.0, 0.0, /* red     */ 0.0, 1.0, /* bottom left */
            x, y, 0.0, 1.0, 0.0, /* green   */ 0.0, 0.0, /* top left */
            x + w, y, 0.0, 0.0, 1.0, /* blue    */ 1.0, 0.0, /* top right */
            x + w, y + h, 1.0, 0.0, 1.0, /* magenta */ 1.0, 1.0, /* bottom right */
        ]
    }
}


pub struct Camera {
    pub pos: Vec3,
    pub rot: Vec3,
    pub fov: f32,
    pub projection: Mat4,
}
//poor-ass camera impl
impl Camera {
    pub fn update(&mut self, e: &Event<()>) {
        const MOVE_DELTA: f32 = 0.1;

        match e {
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput {
                    input: KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(code),
                        ..
                    }, ..
                }, ..
            } => {
                match code {
                    //x
                    VirtualKeyCode::A => self.pos -= glm::vec3(MOVE_DELTA, 0., 0.),
                    VirtualKeyCode::D => self.pos += glm::vec3(MOVE_DELTA, 0., 0.),
                    //y
                    //note: projection matrix is flipping Y values after they pass through the View matrix.
                    VirtualKeyCode::R => self.pos -= glm::vec3(0., MOVE_DELTA, 0.),
                    VirtualKeyCode::F => self.pos += glm::vec3(0., MOVE_DELTA, 0.),
                    //z
                    VirtualKeyCode::W => self.pos -= glm::vec3(0.0, 0., MOVE_DELTA),
                    VirtualKeyCode::S => self.pos += glm::vec3(0.0, 0., MOVE_DELTA),
                    //reset
                    VirtualKeyCode::Back => self.pos = glm::vec3(0., 0., 5.),
                    _ => ()
                }
            }
            _ => (),
        };
    }

    pub fn default_with_aspect(aspect_ratio: f32) -> Self {
        Self {
            pos: glm::vec3(0., 0., 5.),
            rot: glm::vec3(0., 0., 0.),
            fov: 45.,
            projection: glm::perspective(
                aspect_ratio, glm::radians(&glm::vec1(45.)).x,
                0.1, 1000., ),
        }
    }

    fn get_view(pos: &Vec3, rot: Option<&Vec3>) -> Mat4 {
        let mut mtx: Matrix4<f32> = glm::identity();
        mtx = glm::translate(&mtx, &glm::vec3(pos.x, pos.y, pos.z)); // camera translate
        if let Some(rot) = rot { //camera rot
            mtx = glm::rotate(&mtx, glm::radians(&glm::vec1(rot.x)).x, &glm::vec3(1., 0., 0.));
            mtx = glm::rotate(&mtx, glm::radians(&glm::vec1(rot.y)).x, &glm::vec3(0., 1., 0.));
            mtx = glm::rotate(&mtx, glm::radians(&glm::vec1(rot.z)).x, &glm::vec3(0., 0., 1.));
        }
        glm::inverse(&mtx)
    }

    pub fn view_projection(&self) -> Mat4 {
        &self.projection * &Camera::get_view(&self.pos, Some(&self.rot))
    }
}

impl Default for Camera {
    fn default() -> Self {
        Camera::default_with_aspect(6. / 4.)
    }
}

pub fn cast_slice<T, U>(ts: &[T]) -> Option<&[U]> {
    use core::mem::align_of;
    // Handle ZST (this all const folds)
    if size_of::<T>() == 0 || size_of::<U>() == 0 {
        if size_of::<T>() == size_of::<U>() {
            unsafe {
                return Some(core::slice::from_raw_parts(
                    ts.as_ptr() as *const U,
                    ts.len(),
                ));
            }
        } else {
            return None;
        }
    }
    // Handle alignments (this const folds)
    if align_of::<U>() > align_of::<T>() {
        // possible mis-alignment at the new type (this is a real runtime check)
        if (ts.as_ptr() as usize) % align_of::<U>() != 0 {
            return None;
        }
    }
    if size_of::<T>() == size_of::<U>() {
        // same size, so we direct cast, keeping the old length
        unsafe {
            Some(core::slice::from_raw_parts(
                ts.as_ptr() as *const U,
                ts.len(),
            ))
        }
    } else {
        // we might have slop, which would cause us to fail
        let byte_size = size_of::<T>() * ts.len();
        let (new_count, new_overflow) = (byte_size / size_of::<U>(), byte_size % size_of::<U>());
        if new_overflow > 0 {
            return None;
        } else {
            unsafe {
                Some(core::slice::from_raw_parts(
                    ts.as_ptr() as *const U,
                    new_count,
                ))
            }
        }
    }
}