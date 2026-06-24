use candid::Principal;

use yral_canisters_common::{utils::posts::PostDetails, Canisters, Error as CanistersError};

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct FixedFetchCursor<const LIMIT: u64> {
    pub start: u64,
    pub limit: u64,
}

impl<const LIMIT: u64> FixedFetchCursor<LIMIT> {
    pub fn advance(&mut self) {
        self.start += self.limit;
        self.limit = LIMIT;
    }
}

pub struct PostsRes {
    pub posts: Vec<PostDetails>,
    pub end: bool,
}

pub(crate) trait ProfVideoStream<const LIMIT: u64>: Sized {
    async fn fetch_next_posts<const AUTH: bool>(
        cursor: FixedFetchCursor<LIMIT>,
        canisters: &Canisters<AUTH>,
        user_principal: Principal,
        user_canister: Principal,
        username: Option<String>,
    ) -> Result<PostsRes, CanistersError>;
}

pub struct ProfileVideoStream<const LIMIT: u64>;

impl<const LIMIT: u64> ProfVideoStream<LIMIT> for ProfileVideoStream<LIMIT> {
    async fn fetch_next_posts<const AUTH: bool>(
        cursor: FixedFetchCursor<LIMIT>,
        canisters: &Canisters<AUTH>,
        user_principal: Principal,
        user_canister: Principal,
        username: Option<String>,
    ) -> Result<PostsRes, CanistersError> {
        let post_service = canisters.user_post_service().await;
        let posts = post_service
            .get_posts_of_this_user_profile_with_pagination_cursor(
                user_principal,
                cursor.start,
                cursor.limit,
            )
            .await?;

        let end = posts.len() < LIMIT as usize;
        let posts = posts
            .into_iter()
            .map(|post| {
                PostDetails::from_service_post_anonymous(
                    username.clone(),
                    user_canister,
                    post,
                )
            })
            .collect::<Vec<_>>();
        Ok(PostsRes { posts, end })
    }
}
