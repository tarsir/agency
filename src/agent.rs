use std::fmt::Display;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Agent {
    pub pid: String,
    pub socket_path: PathBuf,
}

impl Display for Agent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PID {}: {}",
            &self.pid,
            self.check_agent_identities().unwrap()
        )
    }
}

impl Agent {
    pub fn print_env_commands(&self) {
        println!("export SSH_AUTH_SOCK={:?}", self.socket_path);
        println!("export SSH_AGENT_PID={}", self.pid);
    }

    fn kill_agent(&self) {
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
                    println!("Agent pid {} killed", self.pid);
                } else {
                    eprintln!("Failed to kill agent pid {}", self.pid);
                }
            }
            Err(_) => eprintln!("Failed to kill agent pid {}", self.pid),
        }
    }

    fn check_agent_identities(&self) -> Result<AgentIdentityStatus, Box<dyn std::error::Error>> {
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
                let identity_list =
                    String::from_utf8(o.stdout).unwrap_or("Something went wrong".to_string());
                let identity_list = identity_list.trim().split("\n").collect::<Vec<&str>>();
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

#[derive(Debug, Default)]
pub enum AgentIdentityStatus {
    #[default]
    NoIdentities,
    Identities(i32),
}

impl Display for AgentIdentityStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentIdentityStatus::NoIdentities => {
                write!(f, "No identities")
            }
            AgentIdentityStatus::Identities(n) => {
                write!(
                    f,
                    "{} {}",
                    n,
                    if n == &1 { "identity" } else { "identities" }
                )
            }
        }
    }
}

pub enum RunningAgentCheckStatus {
    SingleAgent(Agent),
    MultipleAgents,
    NoAgents,
}

pub fn purge_empty_agents(agents: Vec<Agent>) -> Vec<Agent> {
    let (mut empty_agents, mut other_agents): (Vec<Agent>, Vec<Agent>) = agents
        .into_iter()
        .partition(|a| match Agent::check_agent_identities(&a) {
            Ok(AgentIdentityStatus::NoIdentities) => true,
            _ => false,
        });
    if other_agents.len() == 0 {
        let empty_last = empty_agents.pop().unwrap();
        other_agents.push(empty_last);
    }

    for a in empty_agents {
        a.kill_agent();
    }

    other_agents
}

pub fn check_agents(agents: &Vec<Agent>) -> RunningAgentCheckStatus {
    match agents.len() {
        0 => RunningAgentCheckStatus::NoAgents,
        1 => RunningAgentCheckStatus::SingleAgent(agents.first().unwrap().clone()),
        _ => RunningAgentCheckStatus::MultipleAgents,
    }
}

pub fn resolve_agent_pids(agents: Vec<Agent>) -> Vec<Agent> {
    let ps_child = Command::new("ps")
        .arg("-ef")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let grep_1_child = Command::new("grep")
        .arg("ssh-agent -s")
        .stdin(Stdio::from(ps_child.stdout.unwrap()))
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let grep_2_output = Command::new("grep")
        .arg("-v")
        .arg("grep")
        .stdin(Stdio::from(grep_1_child.stdout.unwrap()))
        .output()
        .expect("failed to get running agents");

    let stdout = String::from_utf8(grep_2_output.stdout).unwrap_or_default();
    let agent_pids = stdout.split("\n").collect::<Vec<&str>>();

    let agent_pids: Vec<i32> = agent_pids
        .iter()
        .filter_map(|s| {
            s.split_whitespace()
                .nth(1)
                .map(|pid| pid.parse::<i32>().unwrap_or(-1))
        })
        .collect();

    let agents_with_inferred_pids: Vec<Agent> = agent_pids
        .iter()
        .filter_map(|p| {
            agents
                .iter()
                .min_by_key(|&a| (p - (a.pid.parse::<i32>().unwrap_or(-p))).abs())
                .map(|a| Some((p, a)))
                .unwrap_or_else(|| None)
        })
        .map(|(pid, agent)| Agent {
            pid: pid.to_string(),
            ..agent.clone()
        })
        .collect();
    agents_with_inferred_pids
}

pub fn get_current_agents() -> io::Result<Vec<Agent>> {
    let agent_dirs: Vec<Agent> = fs::read_dir("/tmp")?
        .filter_map(|res| match res {
            Ok(res) => {
                if res
                    .path()
                    .to_str()
                    .unwrap_or_default()
                    .starts_with("/tmp/ssh-")
                {
                    Some(res)
                } else {
                    None
                }
            }
            Err(_) => None,
        })
        .map(|dir| -> io::Result<Agent> {
            let socket: Option<fs::DirEntry> = fs::read_dir(dir.path())?
                .next()
                .map(|f| if let Ok(f) = f { Some(f) } else { None })
                .flatten();
            if let Some(socket) = socket {
                let pid = socket.file_name().into_string();
                let pid = if let Ok(pid) = pid {
                    pid.split(".").nth(1).unwrap_or_else(|| "N/A").to_string()
                } else {
                    "N/A".to_string()
                };
                Ok(Agent {
                    pid,
                    socket_path: socket.path(),
                })
            } else {
                Err(io::Error::new(io::ErrorKind::Other, "argh"))
            }
        })
        .filter_map(|a| a.ok())
        .collect();

    Ok(agent_dirs)
}
