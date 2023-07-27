# ssh-agency

`ssh-agency` is a manager for SSH agents (similar to `ssh-ident` or `keychain`) that:

- identifies running agents and helps you identify redundant ones
- helps you connect to a running agent
- provides a non-interactive mode to start your shell sessions with an SSH
  agent easily if there are 0 or 1 existing agents

## Installation

Right now the only installation method aside from building from source is `cargo install`:

```sh
cargo install ssh-agency
```

## Usage

```
CLI helping you manage SSH agents when they get gnarly

Usage: ssh-agency [OPTIONS]

Options:
  -n, --reduce_count  Consolidate to one agent by number of registered identities
  -r, --reduce        Consolidate to one agent with no particular method
  -s, --show-agents   Show the currently running agents
  -y, --ez            Ez mode that non-interactively guarantees an agent when exactly 1 or 0 agents are running
  -p, --purge         Purge agents that have no identities registered
  -h, --help          Print help
  -V, --version       Print version
```

In all options, `ssh-agency` will clear agents that have lingering socket paths but
no process (dead agents).

### Run with no options

Run without options, `ssh-agency` will start an interactive dialog to let the user
clear the running agents without registered identities, and pick an agent to
print the `export` statements to enable the agent in the current environment.

### `-n/--reduce_count`: Consolidate by number of identities

Removes and cleans up all running agents except the agent with the highest
number of registered identities.

### `-r/--reduce`: Consolidate at random

Removes and cleans up all running agents except one, with no particular method
of determining the survivor.

### `-s/--show-agents`: Show running agents

Shows all the running agents and the number of identities registered to each.

### `-y/--ez`: Non-interactive "easy" mode

The "ez" mode is intended to be a scriptable drop-in replacement for tools like
[ssh-ident](https://github.com/ssh-ident/ssh-ident1) or
[keychain](https://linux.die.net/man/1/keychain). In this mode, `ssh-agency` will:

1. Find any running agents
2a. If there is exactly one agent, it will use this agent.
2b. If there are no agents, it will create an agent to use.
2c. If there are more than one agent, it will exit with an error.
3. Print the `export` statements to enable the agent in the current environment.

This option is best used in a scripting scenario or as part of your shell
startup to connect to a running agent if one exists from a previous terminal
session, or create a new one for initial sessions.

### `-p/--purge`: Purge all identity-less agents

Removes and cleans up all running agents that do not have registered identities.

