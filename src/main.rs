use quickcap::{App, AppConfig};

fn main() {
    let app = App::new(Some(AppConfig::from_args()));
    app.run();
}
