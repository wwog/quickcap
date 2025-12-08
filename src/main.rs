use env_logger;
use log::LevelFilter;
use quickcap::App;

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(LevelFilter::Info)
        .init();
    log::info!("Starting QuickCap");
    let app = App::new();
    // app.run();
}
