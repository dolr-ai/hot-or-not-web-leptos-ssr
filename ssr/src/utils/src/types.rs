use candid::Principal;
use serde::{Deserialize, Serialize};
use yral_canisters_client::individual_user_template::PostStatus as PostStatusCandid;
use yral_types::delegated_identity::DelegatedIdentityWire;

pub type PostId = (Principal, String);

#[derive(PartialEq, Clone)]
pub struct PostParams {
    pub canister_id: Principal,
    pub post_id: String,
}

#[derive(PartialEq, Debug, Eq)]
pub enum PostStatus {
    BannedForExplicitness,
    BannedDueToUserReporting,
    Uploaded,
    CheckingExplicitness,
    ReadyToView,
    Transcoding,
    Deleted,
}

impl From<&PostStatusCandid> for PostStatus {
    fn from(status: &PostStatusCandid) -> Self {
        match status {
            PostStatusCandid::BannedForExplicitness => PostStatus::BannedForExplicitness,
            PostStatusCandid::BannedDueToUserReporting => PostStatus::BannedDueToUserReporting,
            PostStatusCandid::Uploaded => PostStatus::Uploaded,
            PostStatusCandid::CheckingExplicitness => PostStatus::CheckingExplicitness,
            PostStatusCandid::ReadyToView => PostStatus::ReadyToView,
            PostStatusCandid::Transcoding => PostStatus::Transcoding,
            PostStatusCandid::Deleted => PostStatus::Deleted,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NewIdentity {
    pub id_wire: DelegatedIdentityWire,
    pub fallback_username: Option<String>,
    pub email: Option<String>,
}

impl NewIdentity {
    pub fn new_without_username(id: DelegatedIdentityWire) -> Self {
        Self {
            id_wire: id,
            fallback_username: None,
            email: None,
        }
    }
}
