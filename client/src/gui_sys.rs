use rx::specs::System;
use rx::specs;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

#[derive(Default)]
pub struct GuiSystem {
    reader: rx::EventReader<()>,
    click_events_gui: Option<Vec<rx::ScreenClickEvent>>,
}

impl<'a> specs::System<'a> for GuiSystem {
    type SystemData = (
        specs::Read<'a, rx::EventChannelReader<()>>,
        specs::Read<'a, rx::EguiCtx>
    );

    fn run(&mut self, (events, gui): Self::SystemData) {

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

        if let (Some(screen_click_event), Some(gui_ctx)) = (self.click_events_gui.as_ref(), gui.as_ref()) {
            use ::rx::egui;
            let r = &gui_ctx.input().screen_rect;
            egui::Window::new("ScreenClickEvent").default_pos(r.max).show(gui_ctx, |ui| {
                egui::Grid::new("events_grid")
                    .striped(true)
                    .spacing([0.0, 8.0])
                    .show(ui, |ui|{
                        for (i, e) in screen_click_event.iter().enumerate() {
                            ui.label(format!("w: {:.2}; h: {:.2};", e.screen_pos.0, e.screen_pos.1));
                            ui.label(format!("x: {:.2}; y: {:.2}; z:{:.2};", e.world_vec.x, e.world_vec.y, e.world_vec.z));
                            ui.end_row();
                        }
                });
            });
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