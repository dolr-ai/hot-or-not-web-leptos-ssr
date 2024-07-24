use candid::Principal;

use crate::{
    canister::individual_user_template::{GetPostsOfUserProfileError, Result5},
    state::canisters::Canisters,
    utils::posts::{FetchCursor, PostDetails, PostViewError},
};

pub struct ProfileVideoStream<'a, const AUTH: bool> {
    cursor: FetchCursor,
    canisters: &'a Canisters<AUTH>,
    user_canister: Principal,
}

impl<'a, const AUTH: bool> ProfileVideoStream<'a, AUTH> {
    pub fn new(
        cursor: FetchCursor,
        canisters: &'a Canisters<AUTH>,
        user_canister: Principal,
    ) -> Self {
        Self {
            cursor,
            canisters,
            user_canister,
        }
    }

    pub async fn fetch_next_profile_posts(&self) -> Result<Vec<PostDetails>, PostViewError> {
        let user = self.canisters.individual_user(self.user_canister).await?;
        let posts = user
            .get_posts_of_this_user_profile_with_pagination_cursor(
                self.cursor.start,
                self.cursor.limit,
            )
            .await?;
        match posts {
            Result5::Ok(v) => Ok(v
                .into_iter()
                .map(|details| PostDetails::from_canister_post(AUTH, self.user_canister, details))
                .collect::<Vec<PostDetails>>()),
            Result5::Err(GetPostsOfUserProfileError::ReachedEndOfItemsList) => Ok(vec![]),
            _ => Err(PostViewError::Canister(
                "user canister refused to send posts".into(),
            )),
        }
    }
}
