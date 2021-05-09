use crate::run::FrameUpdate;
use rx::winit;
use rx::winit::event;
use rx::egui;

pub struct InfoLayer {
    frames: u32,
    frame_rate: f32,
    elapsed: std::time::Duration,
    size_d: winit::dpi::PhysicalSize<u32>,
    cursor_pos: winit::dpi::PhysicalPosition<f64>
}

impl Default for InfoLayer {
    fn default() -> Self {
        Self {
            frames: 0,
            frame_rate: 0.0,
            elapsed: Default::default(),
            size_d: winit::dpi::PhysicalSize { width: 0, height: 0 },
            cursor_pos: winit::dpi::PhysicalPosition { x: 0., y: 0. }
        }
    }
}


impl<T: Clone + Send> rx::Layer<T> for InfoLayer {

    fn on_update(&mut self, upd: FrameUpdate<T>) {
        let mut did_resized = false;
        for rx_e in upd.events {
            match rx_e {
                rx::RxEvent::WinitEvent(event) => match event {
                    event::Event::WindowEvent { event: event::WindowEvent::Resized(size), .. } => {
                        did_resized = true;
                        self.size_d = *size
                    }
                    event::Event::WindowEvent { event: event::WindowEvent::CursorMoved { position, .. }, .. } => {
                        self.cursor_pos = position.clone();
                    }
                    _ => {}
                }
                _ => {}
            }
        }
        self.elapsed += upd.elapsed;
        self.frames += 1;
        if self.elapsed >= std::time::Duration::from_millis(100) {
            self.frame_rate = self.frames as f32 * 0.5 + self.frame_rate * 0.5;
            self.frames = 0;
            self.elapsed -= std::time::Duration::from_millis(100)
        }

        let mut wnd = egui::Window::new("info")
            .collapsible(false)
            .resizable(false);
        if did_resized {
            wnd = wnd.current_pos((self.size_d.width as f32 - 190., 0.));
        }
        wnd.show(&upd.egui_ctx, |ui| {
            egui::Grid::new("info_grid").min_col_width(180.).striped(true).show(ui, |ui| {
                ui.label(format!("Frame time: {} ms", upd.elapsed.as_millis()));
                ui.end_row();
                ui.label(format!("Frames: {:.2} /sec", self.frame_rate * 10.));
                ui.end_row();
                ui.label(format!("Size: {}x{}", self.size_d.width, self.size_d.height));
                ui.end_row();
                ui.label(format!("Cursor: x: {:.2} y: {:.2}", self.cursor_pos.x, self.cursor_pos.y));
                ui.end_row();
            });
        });
    }

    fn name(&self) -> &'static str {
        "info_layer"
    }
}
