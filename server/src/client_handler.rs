use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use anyhow::{anyhow, Context};
use cgroup_manager::LimitedProcess;
use tracing::{error, instrument, trace};

use crate::agent::Agent;
use crate::constraints::Constraints;

#[derive(Debug)]
pub struct ClientHandler {
    stream: TcpStream,
    process: LimitedProcess,
}

impl ClientHandler {
    #[instrument(skip_all,fields(Agent=agent.name))]
    pub fn init(agent: Arc<Agent>, resources: &Constraints) -> anyhow::Result<ClientHandler> {
        // return early if agent has no binary
        let path = agent.path_to_exe.clone().context("agent path is None")?.into_os_string().into_string().unwrap();
        
        let listener = TcpListener::bind("127.0.0.1:0").context("listener creation")?;
        let port_arg = listener.local_addr()?.port().to_string();

        trace!("launching client");

        let cpus = resources
            .cpus
            .iter()
            .map(u8::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let mut full_command = vec!["taskset".to_string(), "-c".to_string(), cpus.clone(), path, port_arg].into_iter();
        let command = full_command.next().unwrap();
        let args = full_command.collect::<Vec<_>>();
        let max_memory = resources.total_ram;

        let mut process = LimitedProcess::launch(&command, &args, max_memory as i64, &cpus).context("child + cgroup creation")?;

        thread::sleep(Duration::from_millis(100)); //NOTE: there is no easy way of having a "accept_with_timout"
                                                   //TODO: semi busy waiting
        listener
            .set_nonblocking(true)
            .context("setting non-blocking to true")?;

        if let Ok((stream, _addr)) = listener.accept() {
            Ok(ClientHandler { stream, process })
        } else {
            process.try_kill(Duration::from_secs(1)).unwrap();
            Err(anyhow!("error accepting connection"))
        }
    }

    #[instrument]
    pub fn send_and_recv(
        &mut self,
        msg: &[u8],
        buf: &mut [u8],
        max_duration: Duration,
    ) -> anyhow::Result<usize> {
        self.stream
            .set_nonblocking(true)
            .context("setting non-blocking for 'write'")?;

        match self.stream.write(msg) {
            Ok(0) => {
                return Err(anyhow!("connection closed by client"));
            }
            Ok(n) => {
                if n < msg.len() {
                    error!(
                        "only {}/{} bytes of {} were sent",
                        n,
                        msg.len(),
                        std::str::from_utf8(msg).unwrap()
                    );
                    return Err(anyhow!("only {}/{} bytes were sent", n, msg.len()));
                }
            }
            Err(e) => {
                return Err(e).context("writing msg");
            }
        }
        self.stream
            .set_nonblocking(false)
            .context("setting blocking for 'read'")?;

        self.stream
            .set_read_timeout(Some(max_duration))
            .context("setting read timout")?;

        let n = self
            .stream
            .read(buf)
            .context("error while reading stream")?;
        Ok(n)
    }

    pub fn kill_child_process(&mut self) -> anyhow::Result<()> {
        self.process.try_kill(Duration::from_secs(1))
    }
}
