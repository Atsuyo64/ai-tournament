use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{self, Child};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use anyhow::{anyhow, Context};
use tracing::{instrument, trace};

use crate::agent::Agent;
use crate::constraints::Constraints;

pub struct ClientHandler {
    stream: TcpStream,
    child: Child,
}

impl ClientHandler {
    #[instrument(skip_all,fields(Agent=agent.name))]
    pub fn init(
        agent: Arc<Agent>,
        _resources: &Constraints,
    ) -> anyhow::Result<ClientHandler> {
        let listener = TcpListener::bind("127.0.0.1:0").context("listener creation")?;

        let path = agent.path_to_exe.as_ref().context("agent path is None")?;
        let port_arg = listener.local_addr()?.port().to_string();

        trace!("launching client at {path:?}");
        
        //TODO: apply resource limitations
        let mut child = process::Command::new(path)
            .arg(port_arg)
            .stdout(process::Stdio::piped())
            .spawn()
            .context("child creation")?;

        thread::sleep(Duration::from_millis(100));//NOTE: there is no easy way of having a "accept_with_timout"
        //TODO: semi busy waiting
        listener
            .set_nonblocking(true)
            .context("setting non-blocking to true")?;

        if let Ok((stream, _addr)) = listener.accept() {
            Ok(ClientHandler { stream , child })
        } else {
            child.kill().unwrap();
            child.wait().unwrap();
            Err(anyhow!("error accepting connection"))
        }
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
                    return Err(anyhow!("only {}/{} bytes were sent", n, msg.len()));
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
    
    pub fn kill_child_process(&mut self) -> anyhow::Result<()> {
        self.child.kill().context("killing child")?;
        self.child.wait().map(|_|()).context("waiting for cleanup")
    }
}
