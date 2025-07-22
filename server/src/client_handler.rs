use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context};
use cgroup_manager::LimitedProcess;
use tracing::{error, instrument};

use crate::agent::Agent;
use crate::constraints::Constraints;

#[derive(Debug)]
pub struct ClientHandler {
    stream: TcpStream,
    process: LimitedProcess,
}

impl ClientHandler {
    const RESPONSE_TIMEOUT_DURATION: Duration = Duration::from_secs(1);

    /// launch a child process running agent with given constraints.
    ///
    /// Child process is killed on drop. Child process's cgroup is cleaned up on drop.
    #[instrument(skip_all,fields(Agent=agent.name))]
    pub fn init(agent: Arc<Agent>, resources: &Constraints) -> anyhow::Result<ClientHandler> {
        assert_eq!(
            resources.total_ram, resources.agent_ram,
            "incorrect ram to launch agent"
        );
        assert_eq!(
            resources.cpus.len(),
            resources.cpus_per_agent,
            "incorrect cpus to launch agents"
        );

        // return early if agent has no binary
        let path = agent
            .path_to_exe
            .clone()
            .context("no path to executable")?
            .into_os_string()
            .into_string()
            .map_err(|_| anyhow!("path is not a valid string"))?;

        let listener = TcpListener::bind("127.0.0.1:0")
            .context("server error: could not create TcpListener")?;
        let port_arg = listener.local_addr()?.port().to_string();

        // trace!("launching client");

        let cpus = resources
            .cpus
            .iter()
            .map(u8::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let mut full_command = vec![
            "taskset".to_string(),
            "-c".to_string(),
            cpus.clone(),
            path,
            port_arg,
        ]
        .into_iter();
        let command = full_command.next().unwrap();
        let args = full_command.collect::<Vec<_>>();
        let max_memory = resources.total_ram;

        let mut process = LimitedProcess::launch(&command, &args, max_memory as i64, &cpus)
            .context("server error: child + cgroup creation failed")?;

        listener
            .set_nonblocking(true)
            .context("server error: setting non-blocking to true")?;

        let response_timeout = Instant::now() + Self::RESPONSE_TIMEOUT_DURATION;
        while Instant::now() < response_timeout {
            if let Ok((stream, _addr)) = listener.accept() {
                return Ok(ClientHandler { stream, process });
            }
            // at least 10 tries
            thread::sleep(Duration::from_millis(10).min(Self::RESPONSE_TIMEOUT_DURATION / 10));
        }

        //FIXME: panic
        process.try_kill(Duration::from_secs(1)).unwrap();
        Err(anyhow!("no connection made to server"))
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
            .context("server error: setting non-blocking for 'write'")?;

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
                        std::str::from_utf8(msg).unwrap_or("NON_VALID_UTF8")
                    );
                    return Err(anyhow!(
                        "msg transmission error: only {}/{} bytes sent",
                        n,
                        msg.len()
                    ));
                }
            }
            Err(e) => {
                return Err(e).context("I/O error while sending msg");
            }
        }
        self.stream
            .set_nonblocking(false)
            .context("server error: setting blocking for 'read'")?;

        self.stream
            .set_read_timeout(Some(max_duration))
            .context("server error: setting read timeout")?;

        let n = self
            .stream
            .read(buf)
            .context("error while reading stream")?;
        Ok(n)
    }

    fn kill_child_process(&mut self) -> anyhow::Result<()> {
        self.process.try_kill(Duration::from_secs(1))
    }
}

impl Drop for ClientHandler {
    fn drop(&mut self) {
        //FIXME: panic
        self.kill_child_process()
            .expect("could not kill child process");
    }
}
