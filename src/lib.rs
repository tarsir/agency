pub mod agent;
pub mod cli;

use agent::{
    running_agents::{
        check_agents, get_current_agents, purge_empty_agents, resolve_agent_pids,
        RunningAgentCheckStatus,
    },
    Agent,
};
use inquire::{Confirm, Select};
use std::io;

pub fn basic_operation() -> io::Result<()> {
    let agents: Vec<Agent> = get_current_agents()?;
    let agents = resolve_agent_pids(&agents);
    let agents = {
        if agents.len() > 1 {
            let message =
            "Found multiple running agents, would you like to terminate all but 1 without identities?";
            let response = Confirm::new(message)
                .with_default(true)
                .with_help_message("Terminates all but 1 empty agents by default")
                .prompt();

            match response {
                Ok(true) => purge_empty_agents(agents),
                Ok(false) => agents,
                Err(e) => {
                    println!(
                        "Something went wrong with the prompt; continuing without terminating"
                    );
                    eprintln!("Error: {}", e);
                    agents
                }
            }
        } else {
            agents
        }
    };

    match check_agents(&agents) {
        RunningAgentCheckStatus::SingleAgent(agent) => {
            // Print out a source-able string sequence eg:
            // export SSH_AUTH_SOCK=/tmp/ssh-Ojfuw4Y4n9Fm/agent.704
            // export SSH_AGENT_PID=705
            agent.print_env_commands();
        }
        RunningAgentCheckStatus::MultipleAgents => {
            let resp = Select::new("Multiple agents are running; you can pick an agent to print environment variables for", agents)
                .prompt();
            match resp {
                Ok(choice) => {
                    choice.print_env_commands();
                }
                Err(e) => {
                    eprintln!("Failed to select agent: {}", e);
                }
            }
        }
        RunningAgentCheckStatus::NoAgents => {
            println!(r#"No running agents; start your own with `eval $(ssh-agent -s)`"#);
        }
    }

    Ok(())
}
