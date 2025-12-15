
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub name: String,
    pub bounds: Rect,
}
