use quickcap::App;

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
    let app = App::new();
    app.run();
}
