pub struct Framebuffer {
  pub width: usize,
  pub height: usize,
  pub pixels: Vec<[u8; 4]> // RGBA
}

impl Framebuffer {
  pub fn new(width: usize, height: usize) -> Self {
    Self {
      width,
      height,
      pixels: vec![[0, 0, 0, 255]; width * height]
    }
  }

  pub fn set_pixel(&mut self, x: usize, y: usize, color: [u8; 4]) {
    let idx = y * self.width + x;
    self.pixels[idx] = color;
  }
}
