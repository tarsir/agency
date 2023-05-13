use agency::agent::{
    get_current_agents, get_dead_agents, purge_empty_agents, resolve_agent_pids, Agent,
};
use agency::basic_operation;
use agency::cli::Cli;
use clap::Parser;
use std::io;
use std::process::Command;

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let agents: Vec<Agent> = get_current_agents()?;
    let mut running_agents = resolve_agent_pids(&agents);
    let dead_agents = get_dead_agents(agents.clone(), running_agents.clone());

    for a in dead_agents.iter() {
        match a.clean_dead_agent_socket() {
            Ok(()) => {
                println!("Removed dead agent's socket: {}", &a.socket_path.display());
            }
            Err(e) => {
                eprintln!(
                    "Unable to remove socket for agent at {}: {}",
                    &a.socket_path.display(),
                    e,
                );
            }
        }
    }

    if cli.ez {
        match &agents[..] {
            [agent] => agent.print_env_commands(),
            [] => {
                let start_agent_output = Command::new("ssh-agent").arg("-s").status();
                if let Ok(s) = start_agent_output {
                    if !s.success() {
                        println!("echo Failed to start new agent");
                    }
                }
            }
            _ => {
                println!("echo Too many running agents");
            }
        }
        return Ok(());
    }

    if cli.show_agents {
        for a in running_agents {
            println!("{}", a);
        }
        return Ok(());
    }

    if cli.purge_empty_agents {
        purge_empty_agents(agents);
        return Ok(());
    }

    let reducers = &cli.reducers;

    if running_agents.len() <= 1 {
        return Ok(());
    }

    if reducers.reduce_simple {
        let kill_queue: Vec<Agent> = running_agents.drain(1..).collect();
        for mut a in kill_queue {
            a.kill_agent();
        }
        return Ok(());
    }

    if reducers.reduce_by_count {
        running_agents.sort_unstable_by(|a, b| {
            let a_identities = match a.check_agent_identities().unwrap_or_default() {
                agency::agent::AgentIdentityStatus::NoIdentities => 0,
                agency::agent::AgentIdentityStatus::Identities(c) => c,
                agency::agent::AgentIdentityStatus::ConnectionRefused => -1,
            };

            let b_identities = match b.check_agent_identities().unwrap_or_default() {
                agency::agent::AgentIdentityStatus::NoIdentities => 0,
                agency::agent::AgentIdentityStatus::Identities(c) => c,
                agency::agent::AgentIdentityStatus::ConnectionRefused => -1,
            };

            b_identities.cmp(&a_identities)
        });
        let kill_queue: Vec<Agent> = running_agents.drain(1..).collect();
        for mut a in kill_queue {
            a.kill_and_clean_agent();
        }
        return Ok(());
    }

    basic_operation()?;
    Ok(())
}
