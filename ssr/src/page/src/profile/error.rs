use ic_agent::AgentError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProfilePostViewError {
    #[error("IC agent error {0}")]
    Agent(#[from] AgentError),
    #[error("Canister error {0}")]
    Canister(String),
}