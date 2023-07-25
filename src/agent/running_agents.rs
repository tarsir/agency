use std::{
    fs, io,
    process::{Command, Stdio},
};

use super::{Agent, AgentIdentityStatus};

/// The possible states of agents running on the system.
pub enum RunningAgentCheckStatus {
    SingleAgent(Agent),
    MultipleAgents,
    NoAgents,
}

/// Kill and clean live agents that have no identities registered while guaranteeing at least one
/// stays alive.
pub fn purge_empty_agents_retain_one(agents: Vec<Agent>) -> Vec<Agent> {
    let (mut empty_agents, mut other_agents): (Vec<Agent>, Vec<Agent>) = agents
        .into_iter()
        .partition(|a| match Agent::check_agent_identities(&a) {
            Ok(AgentIdentityStatus::NoIdentities) => true,
            _ => false,
        });

    if other_agents.len() == 0 && !empty_agents.is_empty() {
        let empty_last = empty_agents.pop().unwrap();
        other_agents.push(empty_last);
    }

    for mut a in empty_agents {
        a.kill_and_clean_agent();
    }

    other_agents
}

/// Kill and clean all live agents that have no identities registered.
pub fn purge_empty_agents(agents: Vec<Agent>) -> Vec<Agent> {
    let (empty_agents, other_agents): (Vec<Agent>, Vec<Agent>) =
        agents
            .into_iter()
            .partition(|a| match Agent::check_agent_identities(&a) {
                Ok(AgentIdentityStatus::NoIdentities) => true,
                _ => false,
            });

    for mut a in empty_agents {
        a.kill_and_clean_agent();
    }

    other_agents
}

/// Build a `RunningAgentCheckStatus` from the list of agents.
///
/// If the list has one agent, `RunningAgentCheckStatus::SingleAgent(Agent)` will take ownership of
/// the agent.
pub fn check_agents(agents: &Vec<Agent>) -> RunningAgentCheckStatus {
    match agents.len() {
        0 => RunningAgentCheckStatus::NoAgents,
        1 => RunningAgentCheckStatus::SingleAgent(agents.first().unwrap().clone()),
        _ => RunningAgentCheckStatus::MultipleAgents,
    }
}

/// Find the pids for each agent in `agents` that .
pub fn resolve_agent_pids(agents: &Vec<Agent>) -> Vec<Agent> {
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
            is_running: true,
            socket_path: agent.socket_path.clone(),
        })
        .collect();
    agents_with_inferred_pids
}

/// Filter the agents in `all_agents` that aren't in `running_agents`.
pub fn get_dead_agents(all_agents: Vec<Agent>, running_agents: Vec<Agent>) -> Vec<Agent> {
    // TODO: why doesn't this check `is_running`?
    all_agents
        .into_iter()
        .filter(|a| {
            !running_agents
                .iter()
                .any(|r_a| r_a.socket_path == a.socket_path)
        })
        .collect()
}

/// Get a list of candidate agents from the expected SSH agent directory.
///
/// The Agents returned by this function will all be marked as not running. They will be checked
/// against the list of agent PIDs later to determine which agents are live.
pub fn get_current_agents() -> io::Result<Vec<Agent>> {
    // TODO: can this be other directories?
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
                    is_running: false,
                    socket_path: socket.path(),
                })
            } else {
                // TODO: use a better error
                Err(io::Error::new(io::ErrorKind::Other, "argh"))
            }
        })
        .filter_map(|a| a.ok())
        .collect();

    Ok(agent_dirs)
}
