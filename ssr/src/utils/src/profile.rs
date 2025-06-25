use candid::Principal;
use consts::MAX_VIDEO_ELEMENTS_FOR_FEED;
use ic_agent::AgentError;
use indexmap::IndexSet;
use leptos::prelude::*;
use yral_canisters_client::individual_user_template::Result6;

use yral_canisters_common::{
    cursored_data::{CursoredDataProvider, PageEntry},
    utils::posts::PostDetails,
    Canisters,
};

use crate::posts::FeedPostCtx;

pub const PROFILE_CHUNK_SZ: usize = 10;

#[derive(Clone)]
pub struct PostsProvider {
    canisters: Canisters<false>,
    video_queue: RwSignal<IndexSet<PostDetails>>,
    // video_queue_for_feed: RwSignal<Vec<FeedPostCtx>>,
    start_index: RwSignal<usize>,
    user: Principal,
}

impl PostsProvider {
    pub fn new(
        canisters: Canisters<false>,
        video_queue: RwSignal<IndexSet<PostDetails>>,
        // video_queue_for_feed: RwSignal<Vec<FeedPostCtx>>,
        start_index: RwSignal<usize>,
        user: Principal,
    ) -> Self {
        Self {
            canisters,
            video_queue,
            // video_queue_for_feed,
            start_index,
            user,
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
        let user = self.canisters.individual_user(self.user).await;
        let limit = end - start;
        let posts = user
            .get_posts_of_this_user_profile_with_pagination_cursor(start as u64, limit as u64)
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
            .map(|details| PostDetails::from_canister_post(false, self.user, details))
            .collect();
        let post_details_indexset: IndexSet<PostDetails> = post_details.iter().cloned().collect();
        self.video_queue.update_untracked(|vq| {
            // let start_len = vq.len();
            vq.extend(post_details_indexset);
            // let end_len = vq.len();

            // Update video_queue_for_feed for newly added posts
            // if end_len > start_len {
            //     self.video_queue_for_feed.update_untracked(|vqf| {
            //         for i in start_len..end_len {
            //             if i >= MAX_VIDEO_ELEMENTS_FOR_FEED {
            //                 break;
            //             }
            //             if i < vqf.len() {
            //                 if let Some(post) = vq.get_index(i) {
            //                     vqf[i].value.set(Some(post.clone()));
            //                 }
            //             }
            //         }
            //     });
            // }
        });
        Ok(PageEntry {
            data: post_details,
            end: list_end,
        })
    }
}
