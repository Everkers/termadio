use anyhow::Result;
use crate::ui::App;

pub async fn run() -> Result<()> {
    let mut app = App::new()?;
    app.run().await
}