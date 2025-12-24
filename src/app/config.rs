#[derive(Debug)]
pub struct AppConfig {
    debug: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self { debug: false }
    }
}

const DEBUG_ARG: &str = "--debug";


impl AppConfig {
    pub fn from_args() -> Self {
        let args = std::env::args().collect::<Vec<String>>();
        let debug = args.contains(&DEBUG_ARG.to_string());
        Self { debug }
    }

    pub fn is_debug(&self) -> bool {
        self.debug
    }
}

pub struct AppConfigBuilder {
    config: AppConfig,
}

impl AppConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: AppConfig::default(),
        }
    }

    /// 是否开启调试模式，调试模式下，蒙层不会设置最前方(macos为窗口保护程序级别，windows为置顶窗口)
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.config.debug = debug;
        self
    }

    pub fn build(self) -> AppConfig {
        self.config
    }
}
