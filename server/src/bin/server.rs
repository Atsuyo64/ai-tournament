use anyhow::{self, Context};

fn main() -> anyhow::Result<()> {
    let address = "127.0.0.1:0";
    let server = std::net::TcpListener::bind(address).context("creating server socket")?;

    Ok(())
}