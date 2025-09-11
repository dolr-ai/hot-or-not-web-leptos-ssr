use candid::Principal;
use yral_canisters_client::individual_user_template::{GetPostsOfUserProfileError, Result6};

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
        user_canister: Principal,
        username: Option<String>,
    ) -> Result<PostsRes, CanistersError>;
}

pub struct ProfileVideoStream<const LIMIT: u64>;

impl<const LIMIT: u64> ProfVideoStream<LIMIT> for ProfileVideoStream<LIMIT> {
    async fn fetch_next_posts<const AUTH: bool>(
        cursor: FixedFetchCursor<LIMIT>,
        canisters: &Canisters<AUTH>,
        user_canister: Principal,
        username: Option<String>,
    ) -> Result<PostsRes, CanistersError> {
        let user = canisters.individual_user(user_canister).await;
        let posts = user
            .get_posts_of_this_user_profile_with_pagination_cursor(cursor.start, cursor.limit)
            .await?;
        match posts {
            Result6::Ok(v) => {
                let end = v.len() < LIMIT as usize;
                let posts = v
                    .into_iter()
                    .map(|mut details| {
                        details.created_by_unique_user_name = username.clone();
                        PostDetails::from_canister_post(
                            AUTH,
                            username.clone(),
                            user_canister,
                            details,
                        )
                    })
                    .collect::<Vec<_>>();
                Ok(PostsRes { posts, end })
            }
            Result6::Err(GetPostsOfUserProfileError::ReachedEndOfItemsList) => Ok(PostsRes {
                posts: vec![],
                end: true,
            }),
            _ => Err(CanistersError::YralCanister(
                "user canister refused to send posts".into(),
            )),
        }
    }
}
