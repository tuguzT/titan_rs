pub enum Event {
    Created,
    Resized(Size),
    Destroyed,
}

pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

impl From<(u32, u32)> for Size {
    fn from(tuple: (u32, u32)) -> Self {
        Size::new(tuple.0, tuple.1)
    }
}

impl Into<(u32, u32)> for Size {
    fn into(self) -> (u32, u32) {
        (self.width, self.height)
    }
}