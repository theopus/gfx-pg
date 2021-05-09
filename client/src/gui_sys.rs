use itertools::Itertools;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use rx::egui;
use rx::glm;
use rx::specs::{Builder, Component, Join, System, VecStorage, WorldExt};
use rx::specs;

use crate::Camera;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct EcsUiWidget {
    name: &'static str,
    visible: bool,
}

pub trait EcsUiWidgetSystem: 'static + Sized + for<'c> specs::RunNow<'c> {
    fn should_draw(widget_st: &specs::ReadStorage<EcsUiWidget>) -> bool {
        let info = Self::widget_info();
        for w in widget_st.join() {
            if w.name.eq(info.name) && w.visible {
                return true;
            }
        }
        return false;
    }

    fn name() -> &'static str;

    fn visible() -> bool {
        false
    }

    fn widget_info() -> EcsUiWidget {
        EcsUiWidget {
            name: Self::name(),
            visible: Self::visible(),
        }
    }

    fn register_widget(self, dispatcher: &mut specs::DispatcherBuilder, world: &mut specs::World) {
        world.create_entity().with(Self::widget_info()).build();
        dispatcher.add_thread_local(self);
    }
}

impl EcsUiWidget {
    pub fn new(name: &'static str, visible: bool) -> Self {
        EcsUiWidget { name, visible }
    }
    pub fn name(&self) -> &'static str {
        self.name
    }
    pub fn visible(&self) -> bool {
        self.visible
    }
}

#[derive(Default)]
pub struct EcsUiSystem;

impl EcsUiWidgetSystem for EcsUiSystem {
    fn name() -> &'static str {
        "EcsUiSystem"
    }

    fn visible() -> bool {
        true
    }
}

impl<'a> specs::System<'a> for EcsUiSystem {
    type SystemData = (
        specs::Read<'a, rx::EguiCtx>,
        specs::WriteStorage<'a, EcsUiWidget>,
    );

    fn run(&mut self, (gui, mut widget_st): Self::SystemData) {
        if let Some(gui) = gui.as_ref() {
            egui::Window::new("EcsWidgets").show(gui, |ui| {
                for widget in (&mut widget_st).join() {
                    if widget.name.eq(Self::name()) {
                        //skip self
                        continue;
                    }
                    ui.checkbox(&mut widget.visible, widget.name);
                    // info!("checkbox: {:?}", widget.visible);
                }
            });
        }
    }

    fn setup(&mut self, world: &mut specs::World) {
        use rx::{
            specs::SystemData,
            specs::shrev::EventChannel,
        };
        Self::SystemData::setup(world);
        // self.reader = Some(world.fetch_mut::<EventChannel<rx::RxEvent<()>>>().register_reader());
    }
}


pub struct CameraUiSystem;

impl EcsUiWidgetSystem for CameraUiSystem {
    fn name() -> &'static str {
        "Camera"
    }

    fn visible() -> bool {
        true
    }
}

impl<'a> specs::System<'a> for CameraUiSystem {
    type SystemData = (
        specs::Read<'a, rx::CameraTarget>,
        specs::Read<'a, rx::ActiveCamera>,
        specs::Read<'a, rx::EguiCtx>,
        specs::ReadStorage<'a, EcsUiWidget>,
        specs::WriteStorage<'a, rx::Camera>,
        specs::WriteStorage<'a, rx::Position>,
    );

    fn run(&mut self, (cam_target, active_cam, gui, mut widget_st, mut cam_st, mut pos_st): Self::SystemData) {
        if Self::should_draw(&widget_st) {
            gui.as_ref().map(|ctx| {
                active_cam.camera_mut(&mut cam_st).map(|cam| {
                    egui::Window::new("Camera").show(ctx, |ui| {
                        match cam {
                            Camera::Targeted(t) => {
                                let mut size = [0., 0.].into();
                                let mut hor_size = ui.horizontal_wrapped(|ui| {
                                    size = ui.label("Targeted camera:").rect.size();
                                    egui::Grid::new("CameraGrid_1").striped(true).show(ui, |ui| {
                                        ui.label("Distance:");
                                        ui.add(egui::DragValue::new(&mut t.distance).speed(0.1));
                                        ui.end_row();
                                        ui.label("Yaw:");
                                        ui.add(egui::DragValue::new(&mut t.yaw).speed(0.1));
                                        ui.end_row();
                                        ui.label("Pitch:");
                                        ui.add(egui::DragValue::new(&mut t.pitch).speed(0.1));
                                        ui.end_row();
                                        active_cam.camera_pos_mut(&mut pos_st).map(|pos| {
                                            ui.label("Position:");
                                            ui.vertical(|ui| {
                                                ui.set_min_width(60.);
                                                ui.label(format!("X: {:.2}", pos.x));
                                                ui.label(format!("Y: {:.2}", pos.y));
                                                ui.label(format!("Z: {:.2}", pos.z));
                                            });
                                            ui.end_row();
                                        });
                                    });
                                }).response.rect.width();
                                ui.add_sized([hor_size, 1.], egui::Separator::default().horizontal());
                                ui.horizontal_wrapped(|ui| {
                                    cam_target.target_pos_mut(&mut pos_st).map(|pos| {
                                        ui.add_sized(size, egui::Label::new("Camera target:"));
                                        egui::Grid::new("CameraGrid_2").striped(true).show(ui, |ui| {
                                            ui.label("Position:");
                                            ui.vertical(|ui| {
                                                ui.set_min_width(60.);
                                                ui.horizontal(|ui|{
                                                    ui.label("X:");
                                                    ui.add(egui::DragValue::new(&mut pos.x)
                                                        .fixed_decimals(2)
                                                        .speed(0.1));
                                                });
                                                ui.horizontal(|ui|{
                                                    ui.label("Y:");
                                                    ui.add(egui::DragValue::new(&mut pos.y)
                                                        .fixed_decimals(2)
                                                        .speed(0.1));
                                                });
                                                ui.horizontal(|ui|{
                                                    ui.label("Z:");
                                                    ui.add(egui::DragValue::new(&mut pos.z)
                                                        .fixed_decimals(2)
                                                        .speed(0.1));
                                                });
                                            });
                                            ui.end_row();
                                        });
                                    });
                                });
                            }
                            Camera::Free => {}
                        }
                    });
                });
            });
        }
    }
}

#[derive(Default)]
pub struct ScreenClickUiSystem {
    reader: rx::EventReader<()>,
    click_events_gui: Option<Vec<rx::ScreenClickEvent>>,
}

impl EcsUiWidgetSystem for ScreenClickUiSystem {
    fn name() -> &'static str {
        "ScreenClicks"
    }
}

impl<'a> specs::System<'a> for ScreenClickUiSystem {
    type SystemData = (
        specs::Read<'a, rx::EventChannelReader<()>>,
        specs::Read<'a, rx::EguiCtx>,
        specs::ReadStorage<'a, EcsUiWidget>
    );

    fn run(&mut self, (events, gui, widgets): Self::SystemData) {
        if self.click_events_gui.is_none() {
            self.click_events_gui = Some(Vec::with_capacity(5));
        } else {
            if self.click_events_gui.as_ref().unwrap().len() > 5 {
                self.click_events_gui.as_mut().unwrap().drain(5..);
            }
        };
        if let Some(reader_id) = &mut self.reader {
            for rx_e in &mut events.read(reader_id) {
                match rx_e {
                    rx::RxEvent::EcsEvent(
                        rx::EcsEvent::ScreenClick(scr_e)
                    ) => {
                        self.click_events_gui.as_mut().unwrap().insert(0, scr_e.clone());
                    }
                    _ => {}
                }
            }
        }
        if Self::should_draw(&widgets) {
            if let (Some(screen_click_event), Some(gui_ctx)) = (self.click_events_gui.as_ref(), gui.as_ref()) {
                use ::rx::egui;
                let r = &gui_ctx.input().screen_rect;
                egui::Window::new("ScreenClickEvent").default_pos(r.max).show(gui_ctx, |ui| {
                    egui::Grid::new("events_grid")
                        .striped(true)
                        .spacing([0.0, 8.0])
                        .show(ui, |ui| {
                            for (i, e) in screen_click_event.iter().enumerate() {
                                ui.label(format!("w: {:.2}; h: {:.2};", e.screen_pos.0, e.screen_pos.1));
                                ui.label(format!("x: {:.2}; y: {:.2}; z:{:.2};", e.world_vec.x, e.world_vec.y, e.world_vec.z));
                                ui.end_row();
                            }
                        });
                });
            }
        }
    }
    fn setup(&mut self, world: &mut specs::World) {
        use rx::{
            specs::SystemData,
            specs::shrev::EventChannel,
        };
        Self::SystemData::setup(world);
        self.reader = Some(world.fetch_mut::<EventChannel<rx::RxEvent<()>>>().register_reader());
    }
}