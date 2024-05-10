mod client;

use crate::client::LunchClient;
use anyhow::Result;

fn main() -> Result<()> {
    let client = LunchClient::new();
    client.run()?;
    Ok(())
}
