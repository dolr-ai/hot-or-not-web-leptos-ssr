use candid::Principal;
use ic_agent::AgentError;
use indexmap::IndexSet;
use leptos::prelude::*;
use yral_canisters_client::{
    ic::USER_INFO_SERVICE_ID, individual_user_template::Result6, user_post_service::Result3,
};

use yral_canisters_common::{
    cursored_data::{CursoredDataProvider, PageEntry},
    utils::posts::PostDetails,
    Canisters,
};

pub const PROFILE_CHUNK_SZ: usize = 10;

#[derive(Clone)]
pub struct PostsProvider {
    canisters: Canisters<false>,
    video_queue: RwSignal<IndexSet<PostDetails>>,
    start_index: RwSignal<usize>,
    user_principal: Principal,
    user_canister: Principal,
}

impl PostsProvider {
    pub fn new(
        canisters: Canisters<false>,
        video_queue: RwSignal<IndexSet<PostDetails>>,
        start_index: RwSignal<usize>,
        user_principal: Principal,
        user_canister: Principal,
    ) -> Self {
        Self {
            canisters,
            video_queue,
            user_principal,
            start_index,
            user_canister,
        }
    }
}

impl CursoredDataProvider for PostsProvider {
    type Data = PostDetails;
    type Error = AgentError;

    async fn get_by_cursor_inner(
        &self,
        start: usize,
        end: usize,
    ) -> Result<PageEntry<PostDetails>, AgentError> {
        match self.user_canister {
            USER_INFO_SERVICE_ID => {
                let post_service_canister = self.canisters.user_post_service().await;
                let limit = end - start;
                let posts_res = post_service_canister
                    .get_posts_of_this_user_profile_with_pagination(
                        self.user_principal,
                        start as u64,
                        limit as u64,
                    )
                    .await?;

                let posts = match posts_res {
                    Result3::Ok(posts) => posts,
                    Result3::Err(get_posts_of_user_profile_error) => {
                        log::warn!("failed to get posts {get_posts_of_user_profile_error:?}");
                        return Ok(PageEntry {
                            data: vec![],
                            end: true,
                        });
                    }
                };

                let list_end = posts.len() < (end - start);

                let post_details: Vec<PostDetails> = posts
                    .into_iter()
                    .map(|post| PostDetails::from_service_post_anonymous(self.user_canister, post))
                    .collect();

                let post_details_indexset: IndexSet<PostDetails> =
                    post_details.iter().cloned().collect();
                self.video_queue.update_untracked(|vq| {
                    vq.extend(post_details_indexset);
                });

                Ok(PageEntry {
                    data: post_details,
                    end: list_end,
                })
            }
            _ => {
                let user = self.canisters.individual_user(self.user_canister).await;
                let limit = end - start;
                let posts = user
                    .get_posts_of_this_user_profile_with_pagination_cursor(
                        start as u64,
                        limit as u64,
                    )
                    .await?;
                let posts = match posts {
                    Result6::Ok(v) => v,
                    Result6::Err(_) => {
                        log::warn!("failed to get posts");
                        return Ok(PageEntry {
                            data: vec![],
                            end: true,
                        });
                    }
                };
                let list_end = posts.len() < (end - start);
                self.start_index.update_untracked(|c| *c = end);
                let post_details: Vec<PostDetails> = posts
                    .into_iter()
                    .map(|details| {
                        PostDetails::from_canister_post(false, self.user_canister, details)
                    })
                    .collect();
                let post_details_indexset: IndexSet<PostDetails> =
                    post_details.iter().cloned().collect();
                self.video_queue.update_untracked(|vq| {
                    vq.extend(post_details_indexset);
                });
                Ok(PageEntry {
                    data: post_details,
                    end: list_end,
                })
            }
        }
    }
}
