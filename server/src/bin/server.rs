use anyhow::{self, Context};

fn main() -> anyhow::Result<()> {
    let address = "[::]:0";
    let server = std::net::TcpListener::bind(address).context("creating server socket")?;
    println!("Address: {}",server.local_addr().unwrap());
    //FIXME: check if ipv6 is available !
    Ok(())
}