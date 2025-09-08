use leptos::prelude::RwSignal;
use yral_canisters_common::utils::posts::PostDetails;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct FetchCursor {
    pub start: u64,
    pub limit: u64,
}

impl Default for FetchCursor {
    fn default() -> Self {
        Self {
            start: 0,
            limit: 10,
        }
    }
}

impl FetchCursor {
    pub fn advance(&mut self) {
        self.start += self.limit;
        self.limit = 25;
    }

    pub fn set_limit(&mut self, limit: u64) {
        self.limit = limit;
    }

    pub fn advance_and_set_limit(&mut self, limit: u64) {
        self.start += self.limit;
        self.limit = limit;
    }
}

// pub fn get_feed_component_identifier() -> impl Fn() -> Option<&'static str> {
//     move || {
//         let loc = get_host();

//         if loc == "yral.com"
//             || loc == "localhost:3000"
//             || loc == "hotornot.wtf"
//             || loc.contains("go-bazzinga-hot-or-not-web-leptos-ssr.fly.dev")
//         // || loc == "hot-or-not-web-leptos-ssr-staging.fly.dev"
//         {
//             Some("PostViewWithUpdatesMLFeed")
//         } else {
//             Some("PostViewWithUpdates")
//         }
//     }
// }

#[derive(Clone, Default)]
pub struct FeedPostCtx<DetailResolver: Sync + Send + 'static = PostDetails> {
    pub key: usize,
    pub value: RwSignal<Option<DetailResolver>>,
}
