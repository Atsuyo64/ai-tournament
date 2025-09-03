use std::{process::Child, time::Duration};

use anyhow::{self, bail, Context};

use super::create_process;

#[derive(Debug)]
pub struct LimitedProcess {
    pub child: Child,
    cleaned_up: bool,
}

impl LimitedProcess {
    pub fn launch(
        _command: &str,
        _args: &[String],
        _max_memory: i64,
        _cpus: &str,
        _allow_stderr: bool,
    ) -> anyhow::Result<LimitedProcess> {
        bail!("cgroups only available on linux")
    }

    pub fn try_kill(&mut self, __max_duration: Duration) -> anyhow::Result<()> {
        self.child.kill().context("could not kill process")?;
        self.cleaned_up = true;
        Ok(())
    }

    pub fn launch_without_container(
        command: &str,
        args: &[String],
        allow_stderr: bool,
    ) -> anyhow::Result<LimitedProcess> {
        let child =
            create_process(command, args, allow_stderr).context("could not create process")?;

        Ok(LimitedProcess {
            child,
            cleaned_up: false,
        })
    }

    /// Will print out as much info as possible
    #[allow(dead_code)]
    pub(crate) fn try_debug_cgroup(&mut self) {}
}

impl Drop for LimitedProcess {
    fn drop(&mut self) {
        static CLEANUP_DURATION: Duration = Duration::from_secs(1);
        if !self.cleaned_up {
            // warn!(
            //     "Process {} was not cleaned up before dropping. Trying to clean up for up to {:?}...",
            //     self.child.id(),
            //     CLEANUP_DURATION
            // );
            match self.try_kill(CLEANUP_DURATION) {
                Ok(_) => { /* happy dance */ }
                Err(e) => {
                    if std::env::var("DEBUG_CGROUP").is_ok() {
                        self.try_debug_cgroup();
                    }
                    panic!("could not kill process/cgroup on LimitedProcess::drop: {e}");
                }
            }
        }
    }
}
