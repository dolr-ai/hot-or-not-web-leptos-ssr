use std::pin::Pin;

use candid::Principal;
use futures::{stream::FuturesOrdered, Stream, StreamExt};
use leptos::prelude::*;

use state::canisters::AuthState;
use utils::{
    host::show_nsfw_content,
    ml_feed::{get_ml_feed_coldstart_clean, get_ml_feed_coldstart_nsfw, get_ml_feed_mixed_v2},
    posts::FetchCursor,
};
use yral_canisters_common::{utils::posts::PostDetails, Canisters, Error as CanistersError};

type PostsStream<'a> = Pin<Box<dyn Stream<Item = Vec<Result<PostDetails, CanistersError>>> + 'a>>;

#[derive(Debug, Eq, PartialEq)]
pub enum FeedResultType {
    PostCache,
    MLFeedCache,
    MLFeed,
    MLFeedColdstart,
}

pub struct FetchVideosRes<'a> {
    pub posts_stream: PostsStream<'a>,
    pub end: bool,
    pub res_type: FeedResultType,
}

pub struct VideoFetchStream<
    'a,
    const AUTH: bool,
    CanFun: for<'x> AsyncFn(&'x Canisters<AUTH>, &'x AuthState) -> Result<Principal, ServerFnError>,
    UserPrincipalFunc: for<'x> AsyncFn(&'x Canisters<AUTH>, &'x AuthState) -> Result<Principal, ServerFnError>,
> {
    canisters: &'a Canisters<AUTH>,
    auth: AuthState,
    cursor: FetchCursor,
    // TODO: combine both later for better ergonomics
    user_canister: CanFun,
    user_principal: UserPrincipalFunc,
}

async fn user_principal_unauth(
    _canisters: &Canisters<false>,
    auth: &AuthState,
) -> Result<Principal, ServerFnError> {
    if let Some(user_principal_id) = auth.user_principal_if_available() {
        return Ok(user_principal_id);
    }
    let cans = auth.cans_wire().await?;
    Ok(cans.profile_details.principal)
}

async fn user_canister_unauth(
    _canisters: &Canisters<false>,
    auth: &AuthState,
) -> Result<Principal, ServerFnError> {
    if let Some(user_canister_id) = auth.user_canister_if_available() {
        return Ok(user_canister_id);
    }

    let cans = auth.cans_wire().await?;
    Ok(cans.user_canister)
}

async fn user_principal_auth(
    canisters: &Canisters<true>,
    _auth: &AuthState,
) -> Result<Principal, ServerFnError> {
    Ok(canisters.user_principal())
}

async fn user_canister_auth(
    canisters: &Canisters<true>,
    _auth: &AuthState,
) -> Result<Principal, ServerFnError> {
    Ok(canisters.user_canister())
}

#[allow(clippy::type_complexity)]
pub fn new_video_fetch_stream(
    canisters: &Canisters<false>,
    auth: AuthState,
    cursor: FetchCursor,
) -> VideoFetchStream<
    '_,
    false,
    impl AsyncFn(&Canisters<false>, &AuthState) -> Result<Principal, ServerFnError>,
    impl AsyncFn(&Canisters<false>, &AuthState) -> Result<Principal, ServerFnError>,
> {
    VideoFetchStream {
        canisters,
        auth,
        cursor,
        user_canister: user_canister_unauth,
        user_principal: user_principal_unauth,
    }
}

#[allow(clippy::type_complexity)]
pub fn new_video_fetch_stream_auth(
    canisters: &Canisters<true>,
    auth: AuthState,
    cursor: FetchCursor,
) -> VideoFetchStream<
    '_,
    true,
    impl AsyncFn(&Canisters<true>, &AuthState) -> Result<Principal, ServerFnError>,
    impl AsyncFn(&Canisters<true>, &AuthState) -> Result<Principal, ServerFnError>,
> {
    VideoFetchStream {
        canisters,
        auth,
        cursor,
        user_canister: user_canister_auth,
        user_principal: user_principal_auth,
    }
}

impl<
        'a,
        const AUTH: bool,
        CanFun: AsyncFn(&Canisters<AUTH>, &AuthState) -> Result<Principal, ServerFnError>,
        UserPrincipalFunc: AsyncFn(&Canisters<AUTH>, &AuthState) -> Result<Principal, ServerFnError>,
    > VideoFetchStream<'a, AUTH, CanFun, UserPrincipalFunc>
{
    async fn user_canister(&self) -> Result<Principal, ServerFnError> {
        (self.user_canister)(self.canisters, &self.auth).await
    }

    async fn user_principal(&self) -> Result<Principal, ServerFnError> {
        (self.user_principal)(self.canisters, &self.auth).await
    }

    pub async fn fetch_post_uids_ml_feed_chunked(
        &self,
        chunks: usize,
        _allow_nsfw: bool,
        video_queue: Vec<PostDetails>,
    ) -> Result<FetchVideosRes<'a>, ServerFnError> {
        let user_canister_id = self.user_canister().await?;
        let user_principal_id = self.user_principal().await?;

        let top_posts = get_ml_feed_mixed_v2(
            user_canister_id,
            user_principal_id,
            self.cursor.limit as u32,
            video_queue.clone(),
        )
        .await
        .map_err(|e| ServerFnError::new(format!("Error fetching ml feed: {e:?}")))?;

        let end = false;
        let chunk_stream = top_posts
            .into_iter()
            .map(move |item| {
                self.canisters.get_post_details_with_nsfw_info(
                    item.canister_id,
                    item.post_id,
                    item.nsfw_probability,
                )
            })
            .collect::<FuturesOrdered<_>>()
            .filter_map(|res| async { res.transpose() })
            .chunks(chunks);

        Ok(FetchVideosRes {
            posts_stream: Box::pin(chunk_stream),
            end,
            res_type: FeedResultType::MLFeed,
        })
    }

    pub async fn fetch_post_uids_mlfeed_cache_chunked(
        &self,
        chunks: usize,
        allow_nsfw: bool,
        video_queue: Vec<PostDetails>,
    ) -> Result<FetchVideosRes<'a>, ServerFnError> {
        let user_canister_id = self.user_canister().await?;

        let show_nsfw = allow_nsfw || show_nsfw_content();
        let top_posts = if show_nsfw {
            get_ml_feed_coldstart_nsfw(
                user_canister_id,
                self.cursor.limit as u32,
                video_queue.clone(),
            )
            .await
            .map_err(|e| ServerFnError::new(format!("Error fetching ml feed: {e:?}")))?
        } else {
            get_ml_feed_coldstart_clean(
                user_canister_id,
                self.cursor.limit as u32,
                video_queue.clone(),
            )
            .await
            .map_err(|e| ServerFnError::new(format!("Error fetching ml feed: {e:?}")))?
        };

        let end = false;
        let chunk_stream = top_posts
            .into_iter()
            .map(move |item| {
                self.canisters.get_post_details_with_nsfw_info(
                    item.canister_id,
                    item.post_id,
                    item.nsfw_probability,
                )
            })
            .collect::<FuturesOrdered<_>>()
            .filter_map(|res| async { res.transpose() })
            .chunks(chunks);

        Ok(FetchVideosRes {
            posts_stream: Box::pin(chunk_stream),
            end,
            res_type: FeedResultType::MLFeedCache,
        })
    }

    pub async fn fetch_post_uids_hybrid(
        &mut self,
        chunks: usize,
        allow_nsfw: bool,
        video_queue: Vec<PostDetails>,
    ) -> Result<FetchVideosRes<'a>, ServerFnError> {
        let res = self
            .fetch_post_uids_ml_feed_chunked(chunks, allow_nsfw, video_queue.clone())
            .await;

        res
    }
}
