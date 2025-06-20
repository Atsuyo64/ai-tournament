use anyhow::{self, Context};

use agent_interface;

fn main() -> anyhow::Result<()> {
    let addr = "127.0.0.1:0";
    let server = std::net::TcpListener::bind(addr).context("could not create server socket")?;
    println!("Address: {}", server.local_addr().unwrap());

    let agent1 = "sleep";
    let args1 = vec![server.local_addr().unwrap().port().to_string()];

    //let mut tcp_agent1 = agent_interface::TcpAgentServer::new(agent1.to_string(),args1);
    //std::net::TcpStream::set_read_timeout(&self)
    
    
    Ok(())
}
