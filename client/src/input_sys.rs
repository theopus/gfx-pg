#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use winit::event::{DeviceEvent, ElementState, KeyboardInput, MouseButton, VirtualKeyCode};

use rx::{glm, RxEvent};
use rx::ecs::{Velocity, WinitEvents};
use rx::ecs::base_systems::camera3d::{ActiveCamera, CameraTarget, TargetedCamera};
use rx::ecs::base_systems::world3d::Position;
use rx::glm::Vec2;
use rx::specs::{Read, System, WriteStorage};
use rx::specs::shrev::EventChannel;
use rx::winit;
use rx::winit::event::WindowEvent;

use crate::specs::World;
use crate::systems::test::MovePad;

#[derive(Default, Debug)]
pub struct InputTestSystem {
    pub should_affect_angle: bool,
    pub should_affect_distance: bool,
    pub speed: f32,
    pub vert: f32,
    pub hor: f32,
    pub ctrl_pressed: bool,
    pub enabled: bool,
    pad: MovePad,
    reader: rx::EventReader<()>,
}

impl<'a> System<'a> for InputTestSystem {
    type SystemData = (
        Read<'a, EventChannel<RxEvent<()>>>,
        Read<'a, ActiveCamera>,
        Read<'a, CameraTarget>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, rx::Camera>,
        WriteStorage<'a, Velocity>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (channel, active, target, mut pos_st, mut cam_st, mut velocity) = data;
        let cam = active.camera_mut(&mut cam_st);
        let pos = target.target_pos_mut(&mut pos_st);

        if cam.is_none() || pos.is_none() {
            return;
        }

        let cam = cam.unwrap();
        let pos = pos.unwrap();

        let mut accum_delta = (0.0, 0.0);
        let mut accum_dist = 0_f32;

        use rx::winit::event;
        if let Some(reader_id) = &mut self.reader {
            for rx_event in channel.read(reader_id) {
                match rx_event {
                    RxEvent::WinitEvent(w_event) => match w_event {
                        event::Event::DeviceEvent {
                            event: DeviceEvent::MouseMotion { delta }, ..
                        } => {
                            if self.should_affect_angle && !self.ctrl_pressed {
                                accum_delta.0 += delta.0;
                                accum_delta.1 += delta.1;
                            }
                            if self.should_affect_distance && !self.ctrl_pressed{
                                accum_dist += delta.1 as f32;
                            }
                        }
                        event::Event::WindowEvent { event, .. } => match event {
                            WindowEvent::MouseInput { state, button, .. } => {
                                match button {
                                    MouseButton::Left => match state {
                                        ElementState::Pressed => self.should_affect_angle = true,
                                        ElementState::Released => self.should_affect_angle = false,
                                    }
                                    MouseButton::Right => match state {
                                        ElementState::Pressed => self.should_affect_distance = true,
                                        ElementState::Released => self.should_affect_distance = false,
                                    }
                                    _ => {}
                                }
                            }
                            WindowEvent::ModifiersChanged(state) => {
                                if (*state & winit::event::ModifiersState::CTRL).is_empty() {
                                    self.ctrl_pressed = false
                                } else{
                                    self.ctrl_pressed = true
                                }
                            }
                            WindowEvent::KeyboardInput {
                                input: KeyboardInput { virtual_keycode, state,  .. },
                                ..
                            } => if let Some(keycode) = virtual_keycode {
                                match keycode {
                                    VirtualKeyCode::Space => match state {
                                        ElementState::Pressed => pos.y += 1.,
                                        ElementState::Released => {}
                                    },
                                    VirtualKeyCode::C => match state {
                                        ElementState::Pressed => pos.y -= 1.,
                                        ElementState::Released => {}
                                    }
                                    VirtualKeyCode::W => self.pad.up = match state {
                                        ElementState::Pressed => true,
                                        ElementState::Released => false,
                                    },
                                    VirtualKeyCode::S => self.pad.down = match state {
                                        ElementState::Pressed => true,
                                        ElementState::Released => false,
                                    },
                                    VirtualKeyCode::A => self.pad.left = match state {
                                        ElementState::Pressed => true,
                                        ElementState::Released => false,
                                    },
                                    VirtualKeyCode::D => self.pad.right = match state {
                                        ElementState::Pressed => true,
                                        ElementState::Released => false,
                                    },
                                    VirtualKeyCode::F6 => match state {
                                        ElementState::Pressed => {
                                            self.enabled = !self.enabled;
                                        },
                                        _ => {},
                                    },
                                    _ => {}
                                }
                            },
                            _ => {}
                        },
                        _ => (),
                    },
                    _ => {}
                };
            }
        }

        if !self.enabled {
            return;
        }

        let cam_yaw = if let rx::Camera::Targeted(cam) = cam {
            cam.distance += 0.4 * accum_dist;
            cam.yaw += (0.4 * accum_delta.0) as f32;
            cam.pitch -= (0.4 * accum_delta.1) as f32;
            cam.yaw
        } else {
            0.
        };

        let degree = cam_yaw - 180.;

        let d_vec: Vec2 = self.pad.as_vec2(true);
        if self.pad.is_active() {
            self.speed = 0.5;

            let d_vec: Vec2 = glm::rotate_vec2(&d_vec, glm::radians(&glm::vec1(degree)).x);
            let move_vec = self.speed * d_vec;

            let v = match target.target_vel_mut(&mut velocity) {
                None => return,
                Some(e) => e
            };
            v.v = glm::vec3(move_vec.y, 0., move_vec.x);
        };
    }

    fn setup(&mut self, world: &mut World) {
        use rx::{
            specs::SystemData,
            specs::shrev::EventChannel,
        };
        Self::SystemData::setup(world);
        self.reader = Some(world.fetch_mut::<EventChannel<RxEvent<()>>>().register_reader());
    }
}
