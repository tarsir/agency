use std::fmt::Display;

/// The possible statuses of an agent's identity list.
///
/// An agent that is not alive returns an error when the identity list is queried, which is
/// represented here as `ConnectionRefused`.
#[derive(Debug, Default)]
pub enum AgentIdentityStatus {
    #[default]
    NoIdentities,
    Identities(i32),
    ConnectionRefused,
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
            AgentIdentityStatus::ConnectionRefused => {
                write!(f, "Connection attempt refused")
            }
        }
    }
}
