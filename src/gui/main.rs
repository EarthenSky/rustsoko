use std::error::Error;

mod util;
mod app;
use app::App;

// Run the main function
fn main() -> Result<(), Box<dyn Error>> {
    let mut app = App::new();
    app.run()?;
    Ok(())
}
