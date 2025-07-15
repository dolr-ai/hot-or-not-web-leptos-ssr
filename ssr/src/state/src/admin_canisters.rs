use candid::Principal;
use ic_agent::Identity;
use leptos::prelude::*;
use yral_canisters_client::{
    individual_user_template::IndividualUserTemplate, sns_swap::SnsSwap, user_index::UserIndex,
};
use yral_canisters_common::agent_wrapper::AgentWrapper;

#[derive(Clone)]
pub struct AdminCanisters {
    agent: AgentWrapper,
}

impl AdminCanisters {
    pub fn new(key: impl Identity + 'static) -> Self {
        Self {
            agent: AgentWrapper::build(|b| b.with_identity(key)),
        }
    }

    pub fn principal(&self) -> Principal {
        self.agent.principal().unwrap()
    }

    pub async fn get_agent(&self) -> &ic_agent::Agent {
        self.agent.get_agent().await
    }

    pub async fn user_index_with(&self, idx_principal: Principal) -> UserIndex<'_> {
        let agent = self.agent.get_agent().await;
        UserIndex(idx_principal, agent)
    }

    pub async fn individual_user_for(
        &self,
        user_canister: Principal,
    ) -> IndividualUserTemplate<'_> {
        let agent = self.agent.get_agent().await;
        IndividualUserTemplate(user_canister, agent)
    }

    pub async fn sns_swap(&self, swap_canister: Principal) -> SnsSwap<'_> {
        let agent = self.agent.get_agent().await;
        SnsSwap(swap_canister, agent)
    }
}

pub fn admin_canisters() -> AdminCanisters {
    expect_context()
}
