use std::fmt::Display;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Default, Clone)]
struct Agent {
    pub pid: String,
    pub socket_path: PathBuf,
}

impl Agent {
    fn print_env_commands(&self) {
        println!("export SSH_AUTH_SOCK={:?}", self.socket_path);
        println!("export SSH_AGENT_PID={}", self.pid);
    }

    fn check_agent_identities(agent: &Self) {
        match Command::new("ssh-add")
            .arg("-l")
            .env("SSH_AUTH_SOCK", agent.socket_path.to_str().unwrap_or_default())
            .env("SSH_AGENT_PID", &agent.pid)
            .output() {
                Ok(o) => {
                    let identity_list = String::from_utf8(o.stdout).unwrap_or("Something went wrong".to_string());
                    let identity_list = identity_list.trim()
                        .split("\n")
                        .collect::<Vec<&str>>();
                    match identity_list[..] {
                        ["The agent has no identities."] => AgentIdentityStatus::NoIdentities,
                        _ => AgentIdentityStatus::MultipleIdentities(identity_list.len() as i32),
                    };
                },
                Err(e) => {
                    println!("Error checking agent {}: {:?}", &agent.pid, e)
                }
            }
    }
}

enum AgentIdentityStatus {
    NoIdentities,
    MultipleIdentities(i32),
}

impl Display for AgentIdentityStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentIdentityStatus::NoIdentities => {
                write!(f, "No identities")
            },
            AgentIdentityStatus::MultipleIdentities(n) => {
                write!(f, "{} {}", n, if n == &1 { "identity" } else { "identities" })
            },
        }
    }
}

enum RunningAgentCheckStatus {
    SingleAgent(Agent),
    MultipleAgents,
    NoAgents,
}

fn main() -> io::Result<()> {
    let agents: Vec<Agent> = get_current_agents()?;
    println!("{:?}", agents);
    agents[0].print_env_commands();
    agents.iter().for_each(|a| Agent::check_agent_identities(a));
    // match check_agents(agents) {
    //     RunningAgentCheckStatus::SingleAgent(agent) => {
    //         // Tell the user to do something similar to this:
    //         // SSH_AUTH_SOCK=/tmp/ssh-Ojfuw4Y4n9Fm/agent.704; export SSH_AUTH_SOCK;
    //         // SSH_AGENT_PID=705; export SSH_AGENT_PID;
    //         // echo Agent pid 705;
    //         println!("")
    //     },
    //     RunningAgentCheckStatus::MultipleAgents => todo!(),
    //     RunningAgentCheckStatus::NoAgents => todo!(),
    // }
    Ok(())
}

fn check_agents(agents: Vec<Agent>) -> RunningAgentCheckStatus {
    RunningAgentCheckStatus::NoAgents
}

fn get_current_agents() -> io::Result<Vec<Agent>> {
    let agent_dirs: Vec<Agent> =
        fs::read_dir("/tmp")?
            .filter_map(|res| {
                match res {
                    Ok(res) => {
                        if res.path().to_str().unwrap_or_default().starts_with("/tmp/ssh-") {
                            Some(res)
                        } else {
                            None
                        }
                    },
                    Err(_) => None
                }
            })
            .map(|dir| -> io::Result<Agent> {
                let socket: Option<fs::DirEntry> = fs::read_dir(dir.path())?.next().map(|f| {
                    if let Ok(f) = f {
                        Some(f)
                    } else {
                        None
                    }
                }).flatten();
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
