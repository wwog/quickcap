#[allow(dead_code)]
#[derive(Debug)]
pub struct CaptureResult {
    /// RGB/RGBA 数据
    pub data: Vec<u8>,
    /// 图像宽度
    pub width: u32,
    /// 图像高度
    pub height: u32,
    /// 显示器 ID
    pub display_id: usize,
    /// 是否显示鼠标
    pub show_cursor: bool,
}