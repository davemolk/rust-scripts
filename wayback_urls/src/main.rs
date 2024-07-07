mod client;
use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let args = client::Args::parse();
    let client = client::WaybackClient::new(args);
    client.run()?;
    Ok(())
}
