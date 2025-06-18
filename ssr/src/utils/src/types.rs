use candid::Principal;
use yral_canisters_client::individual_user_template::PostStatus as PostStatusCandid;

pub type PostId = (Principal, u64);

#[derive(PartialEq, Clone, Copy)]
pub struct PostParams {
    pub canister_id: Principal,
    pub post_id: u64,
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
