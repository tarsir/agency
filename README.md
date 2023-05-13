# ssh-agenz

ssh-agenz is a manager for SSH agents (similar to `ssh-ident` or `keychain`) that:

- identifies running agents and helps you identify redundant ones
- helps you connect to a running agents
- does those things quickly and with a lightweight, cross-compiled executable

## Usage

- purge identity-less agents
- consolidate to one agent
- "ez" mode to connect in the case of 0 or 1 agents without interaction

TODO: want to be able to add this to a shell profile script like `.bashrc` and then source the 
output if it can connect to a running agent.


