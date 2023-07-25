pub mod files;
pub mod identities;
pub mod running_agents;
use std::fmt::Display;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

use self::identities::AgentIdentityStatus;

/// The SSH agent concept struct.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Agent {
    pub pid: String,
    pub socket_path: PathBuf,
    pub is_running: bool,
}

impl Display for Agent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PID {}: {} at {} ({})",
            &self.pid,
            self.check_agent_identities().unwrap(),
            &self.socket_path.display(),
            if self.is_running { "Running" } else { "Dead" }
        )
    }
}

impl Agent {
    /// Print the env value exports that would be set by an initialization.
    pub fn print_env_commands(&self) {
        println!("export SSH_AUTH_SOCK={:?}", self.socket_path);
        println!("export SSH_AGENT_PID={}", self.pid);
    }

    /// Kill the agent and ensure the socket paths are cleaned afterwards.
    ///
    /// If the agent was alive when this function is called, the `ssh-agent` invocation will handle
    /// the path and process clean up. If it was not alive, Agency will attempt to handle these
    /// clean up steps.
    pub fn kill_and_clean_agent(&mut self) {
        let started_as_running = self.is_running;
        self.kill_agent();
        if started_as_running {
            return;
        }

        match self.clean_dead_agent_socket() {
            Ok(()) => {
                println!(
                    "Removed dead agent's socket: {}",
                    &self.socket_path.display()
                );
            }
            Err(e) => {
                eprintln!(
                    "Unable to remove socket for agent at {}: {}",
                    &self.socket_path.display(),
                    e,
                );
            }
        }
    }

    /// Kill the agent via the `ssh-agent` tool.
    pub fn kill_agent(&mut self) {
        match Command::new("ssh-agent")
            .arg("-k")
            .env(
                "SSH_AUTH_SOCK",
                self.socket_path.to_str().unwrap_or_default(),
            )
            .env("SSH_AGENT_PID", &self.pid)
            .stdout(Stdio::null())
            .status()
        {
            Ok(status) => {
                if status.success() {
                    self.is_running = false;
                    println!("Agent pid {} killed", self.pid);
                } else {
                    eprintln!("Failed to kill agent pid {}", self.pid);
                }
            }
            Err(_) => eprintln!("Failed to kill agent pid {}", self.pid),
        }
    }

    /// Check the identities present on the agent.
    ///
    /// If the agent is not alive, this will return `AgentIdentityStatus::ConnectionRefused`.
    pub fn check_agent_identities(
        &self,
    ) -> Result<AgentIdentityStatus, Box<dyn std::error::Error>> {
        match Command::new("ssh-add")
            .arg("-l")
            .env(
                "SSH_AUTH_SOCK",
                self.socket_path.to_str().unwrap_or_default(),
            )
            // the PID may not be required, funnily enough
            .env("SSH_AGENT_PID", &self.pid)
            .output()
        {
            Ok(o) => {
                let output = String::from_utf8(o.stdout);
                let identity_list = output.unwrap_or("Something went wrong".to_string());
                let identity_list = identity_list.trim().split("\n").collect::<Vec<&str>>();

                let stderr = String::from_utf8(o.stderr).unwrap_or_default();

                if stderr
                    .trim()
                    .contains("Could not open a connection to your authentication agents.")
                {
                    return Ok(AgentIdentityStatus::ConnectionRefused);
                }

                match identity_list[..] {
                    ["The agent has no identities."] => Ok(AgentIdentityStatus::NoIdentities),
                    _ => Ok(AgentIdentityStatus::Identities(identity_list.len() as i32)),
                }
            }
            Err(e) => {
                println!("Error checking agent {}: {:?}", &self.pid, e);
                Err(Box::new(e))
            }
        }
    }
}
