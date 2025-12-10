use env_logger;
use log::LevelFilter;
use quickcap::App;
fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(LevelFilter::Info)
        .init();
    log::info!("QuickCap started");
    let app = App::new();
    app.run();
}
