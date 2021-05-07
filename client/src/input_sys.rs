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
    pad: MovePad,
    reader: Option<rx::specs::shrev::ReaderId<RxEvent<()>>>,
}

impl<'a> System<'a> for InputTestSystem {
    type SystemData = (
        Read<'a, EventChannel<RxEvent<()>>>,
        Read<'a, ActiveCamera>,
        Read<'a, CameraTarget>,
        WriteStorage<'a, Position>,
        //        Write<'a, CameraTarget>,
        WriteStorage<'a, TargetedCamera>,
        WriteStorage<'a, Velocity>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (channel, active, target, mut position, mut camera, mut velocity) = data;

        let cam = camera.get_mut(active.0.unwrap()).unwrap();
        let pos = position.get_mut(target.0.unwrap()).unwrap();

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
                            if self.should_affect_angle {
                                accum_delta.0 += delta.0;
                                accum_delta.1 += delta.1;
                            }
                            if self.should_affect_distance {
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
                            WindowEvent::KeyboardInput {
                                input: KeyboardInput { virtual_keycode, state, .. },
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
                                    _ => {}
                                }
                            },
                            WindowEvent::Resized(size) => cam.update_aspect(size.width as f32 / size.height as f32),
                            _ => {}
                        },
                        _ => (),
                    },
                    _ => {}
                };
            }
        }


        cam.distance += 0.4 * accum_dist;
        cam.yaw += (0.4 * accum_delta.0) as f32;
        cam.pitch -= (0.4 * accum_delta.1) as f32;

        let degree = cam.yaw - 180.;

        let d_vec: Vec2 = self.pad.as_vec2(true);
        if self.pad.is_active() {
            self.speed = 0.5;

            let d_vec: Vec2 = glm::rotate_vec2(&d_vec, glm::radians(&glm::vec1(degree)).x);
            let move_vec = self.speed * d_vec;

            let mut v: &mut Velocity = velocity.get_mut(target.0.unwrap()).unwrap();
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
