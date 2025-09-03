use candid::Principal;
use ic_agent::Identity;
use leptos::prelude::*;
use yral_canisters_client::{
    ic::USER_INFO_SERVICE_ID, individual_user_template::IndividualUserTemplate, sns_swap::SnsSwap,
    user_index::UserIndex, user_info_service::UserInfoService,
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

    pub async fn user_info_service(&self) -> UserInfoService<'_> {
        let agent = self.agent.get_agent().await;
        UserInfoService(USER_INFO_SERVICE_ID, agent)
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
