use std::process::Command;

use ssh_agency::agent::{
    running_agents::{get_current_agents, resolve_agent_pids},
    Agent,
};
mod run_binary;

#[test]
fn purge_agents() {
    // try with 1 empty agent
    let _fresh_agent = Command::new("ssh-agent")
        .arg("-s")
        .status()
        .expect("Unable to start SSH agent");
    assert_ne!(run_binary::run(&["-p"]), String::new());

    // try with 0
    assert_eq!(run_binary::run(&["-p"]), String::new());

    // try with 1 agent with an identity loaded
    let mut agent = make_agent_with_identity();

    agent.kill_and_clean_agent();
    assert_eq!(run_binary::run(&["-p"]), String::new());
}

#[test]
fn show_agents() {
    assert_eq!(run_binary::run(&["-s"]), "No running agents");

    let mut agent = make_agent();
    let output = run_binary::run(&["-s"]);
    agent.kill_and_clean_agent();
    assert!(output.contains("No identities"));
    assert!(output.contains(&agent.pid.to_string()));
    assert!(output.contains(&agent.socket_path.to_str().unwrap()));

    let mut agent_with_ids = make_agent_with_identity();
    let output = run_binary::run(&["-s"]);
    agent_with_ids.kill_and_clean_agent();
    assert!(output.contains("1 identity"));
    assert!(output.contains(&agent_with_ids.pid.to_string()));
    assert!(output.contains(&agent_with_ids.socket_path.to_str().unwrap()));
}

#[test]
fn reduce_by_count() {
    let agents: Vec<Agent> = (1..=5).map(|s| make_agent()).collect();
    let mut agent_with_identity = make_agent_with_identity();
    assert_eq!(run_binary::run(&["-s"]).lines().count(), 6);

    run_binary::run(&["-n"]);
    let output = run_binary::run(&["-s"]);
    assert_eq!(output.lines().count(), 1);
    assert!(output.contains(&agent_with_identity.pid));
    agent_with_identity.kill_and_clean_agent();
    println!("{}", output);
    println!("{:?}", &agents);
    for mut a in agents {
        println!("{}", &a.pid);
        assert!(!output.contains(&a.pid));
        a.kill_and_clean_agent();
    }
}

#[test]
fn reduce_simple() {
    let agents: Vec<Agent> = (1..=5).map(|s| make_agent()).collect();
    assert_eq!(run_binary::run(&["-s"]).lines().count(), 5);

    run_binary::run(&["-r"]);
    assert_eq!(run_binary::run(&["-s"]).lines().count(), 1);
    for mut a in agents {
        a.kill_and_clean_agent();
    }
}

fn make_agent() -> Agent {
    let _fresh_agent = Command::new("ssh-agent")
        .arg("-s")
        .status()
        .expect("Unable to start SSH agent");
    let agents = get_current_agents().expect("Unable to get test agent");
    let agents = resolve_agent_pids(&agents);
    let agent = agents.last().unwrap().clone();
    agent
}

fn make_agent_with_identity() -> Agent {
    let _fresh_agent = Command::new("ssh-agent")
        .arg("-s")
        .status()
        .expect("Unable to start SSH agent");
    let agents = get_current_agents().expect("Unable to get test agent");
    let agents = resolve_agent_pids(&agents);
    let agent = agents.last().unwrap().clone();
    Command::new("ssh-add")
        .arg("./tests/data/id_ed25519_key")
        .env("SSH_AUTH_SOCK", &agent.socket_path)
        .env("SSH_AGENT_PID", &agent.pid)
        .status()
        .expect("Unable to add test identity");
    agent
}
