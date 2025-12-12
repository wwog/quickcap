use std::sync::Arc;

#[allow(dead_code)]
pub struct Frame {
    pub data: Arc<Vec<u8>>,
    pub width: u32,
    pub height: u32,
}