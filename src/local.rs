use crate::window::UserInput;

#[derive(Debug, Clone, Copy, Default)]
pub struct LocalState {
  pub frame_width: u32,
  pub frame_height: u32,
  pub mouse_x: i32,
  pub mouse_y: i32,
}
impl LocalState {
  pub fn update_from_input(&mut self, input: UserInput) {
    if let Some(frame_size) = input.new_frame_size {
      self.frame_width = frame_size.0;
      self.frame_height = frame_size.1;
    }
    if let Some(position) = input.new_mouse_position {
      self.mouse_x = position.0;
      self.mouse_y = position.1;
    }
  }
}