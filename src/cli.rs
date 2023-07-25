use clap::{Args, Parser};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(flatten)]
    pub reducers: Reducers,

    #[arg(short, long, help = "Show the currently running agents")]
    pub show_agents: bool,

    #[arg(
        short = 'y',
        long,
        help = "Ez mode that non-interactively guarantees an agent when exactly 1 or 0 agents are running"
    )]
    pub ez: bool,

    #[arg(
        short,
        long = "purge",
        help = "Purge agents that have no identities registered"
    )]
    pub purge_empty_agents: bool,
}

#[derive(Args)]
#[group(required = false, multiple = false)]
pub struct Reducers {
    #[arg(
        short = 'n',
        long = "reduce_count",
        help = "Consolidate to one agent by number of registered identities"
    )]
    pub reduce_by_count: bool,

    #[arg(
        short = 'r',
        long = "reduce",
        help = "Consolidate to one agent with no particular method"
    )]
    pub reduce_simple: bool,
}
