use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::Child;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use anyhow::{anyhow, Context};

use crate::agent::Agent;
use crate::available_resources;

pub struct ClientHandler {
    stream: TcpStream,
    child: Child,
}

impl ClientHandler {
    pub fn init(
        agent: Arc<Agent>,
        _resources: available_resources::MatchResourceLimit,
    ) -> anyhow::Result<ClientHandler> {
        let listener = TcpListener::bind("127.0.0.1:0").context("listener creation")?;

        let path = agent.path_to_exe.as_ref().context("agent path is None")?;
        let port_arg = listener.local_addr()?.port().to_string();

        println!("path {path:?}");

        //TODO: apply resource limitations
        //REVIEW: communication with piped sdtio ?
        let child = std::process::Command::new(path)
            .arg(port_arg)
            .spawn()
            .context("child creation")?;

        thread::sleep(Duration::from_millis(100));

        listener
            .set_nonblocking(true)
            .context("setting non-blocking to true")?;

        let (stream, _addr) = listener.accept().context("accepting connection")?;

        Ok(ClientHandler { stream , child })
    }

    pub fn send_and_recv(&mut self, msg: &[u8],buf: &mut[u8], max_duration: Duration) -> anyhow::Result<usize> {
        self.stream
            .set_nonblocking(true)
            .context("setting non-blocking for 'write'")?;
        match self.stream.write(msg) {
            Ok(0) => {
                return Err(anyhow!("connection closed by client"));
            }
            Ok(n) => {
                if n < msg.len() {
                    return Err(anyhow!("only {} bytes were sent out of {}", n, msg.len()));
                }
            }
            Err(e) => {
                return Err(e).context("writing msg");
            }
        }
        self.stream.set_nonblocking(false).context("setting blocking for 'read'")?;

        self.stream.set_read_timeout(Some(max_duration)).context("setting read timout")?;
        
        let n = self.stream.read(buf).context("error while reading stream")?;
        Ok(n)
    }
    
    pub fn kill_child(&mut self) -> anyhow::Result<()> {
        self.child.kill().context("killing child")
    }
}
