use std::sync::mpsc;

use tao::{event::WindowEvent, window::Window};

#[allow(dead_code)]
pub struct AppWindow {
    window: Window,
    pub channel:(Sender<T>, Receiver<T>),
}

impl AppWindow {
    pub fn new(window: Window, channel:(Sender<T>, Receiver<T>)) -> Self {
        Self { window, channel }
    }
}