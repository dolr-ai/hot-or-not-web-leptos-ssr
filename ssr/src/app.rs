use crate::error_template::{AppError, ErrorTemplate};
use component::content_upload::AuthorizedUserToSeedContent;
use component::{base_route::BaseRoute, nav::NavBar};
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::hooks::use_location;
use leptos_router::{components::*, path, MatchNestedRoutes};
use page::about_us::AboutUs;
use page::internal::clear_sats::ClearSats;
use page::leaderboard::Leaderboard;
use page::post_view::PostDetailsCacheCtx;
use page::pumpdump;
use page::root::YralRootPage;
use page::terms_android::TermsAndroid;
use page::terms_ios::TermsIos;
use page::{
    err::ServerErrorPage,
    logout::Logout,
    menu::Menu,
    post_view::{single_post::SinglePost, PostView, PostViewCtx},
    privacy::PrivacyPolicy,
    profile::{
        edit::{username::ProfileUsernameEdit, ProfileEdit},
        profile_post::ProfilePost,
        LoggedInUserProfileView, ProfilePostsContext, ProfileView,
    },
    refer_earn::ReferEarn,
    settings::Settings,
    terms::TermsOfService,
    token::{info::TokenInfo, transfer::TokenTransfer},
    upload::{UploadAiPostPage, UploadOptionsPage, UploadPostPage},
    wallet::Wallet,
};
use state::app_state::AppState;
use state::app_type::AppType;
use state::hn_bet_state::HnBetState;
use state::{audio_state::AudioState, content_seed_client::ContentSeedClient};
use utils::event_streaming::events::HistoryCtx;
use utils::event_streaming::EventHistory;
use utils::mixpanel::state::MixpanelState;
use utils::types::PostParams;
use yral_canisters_common::Canisters;

#[component]
fn NotFound() -> impl IntoView {
    let mut outside_errors = Errors::default();
    outside_errors.insert_with_default_key(AppError::NotFound);
    view! { <ErrorTemplate outside_errors /> }
}

#[component(transparent)]
fn GoogleAuthRedirectHandlerRoute() -> impl MatchNestedRoutes + Clone {
    let path = path!("/auth/google_redirect");
    #[cfg(any(feature = "oauth-ssr", feature = "oauth-hydrate"))]
    {
        use page::yral_auth_redirect::YralAuthRedirectHandler;
        view! { <Route path view=YralAuthRedirectHandler /> }.into_inner()
    }
    #[cfg(not(any(feature = "oauth-ssr", feature = "oauth-hydrate")))]
    {
        view! { <Route path view=NotFound /> }.into_inner()
    }
}

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <meta name="facebook-domain-verification" content="sqtv2sr90ar0ck7t7zcklos44fw8t3" />
                <script fetchpriority="low" type="module" src="/js/sentry-init.js" async></script>
                <script
                    fetchpriority="low"
                    type="module"
                    src="/js/store-initial-url.js"
                    async
                ></script>

                <AutoReload options=options.clone() />
                <HashedStylesheet id="leptos" options=options.clone() />
                <Meta property="og:title" content="YRAL - World's first social on Bitcoin" />
                <Meta property="og:image" content="/img/common/preview.webp" />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let app_type = AppType::select();
    let app_state = AppState::from_type(&app_type);
    provide_context(app_state.clone());

    // Existing context providers
    provide_context(Canisters::default());
    provide_context(ContentSeedClient::default());
    provide_context(PostViewCtx::new());
    provide_context(ProfilePostsContext::default());
    provide_context(AuthorizedUserToSeedContent::default());
    provide_context(AudioState::default());
    provide_context(PostDetailsCacheCtx::default());

    // History Tracking
    let history_ctx = HistoryCtx::default();
    provide_context(history_ctx.clone());

    let _ = HnBetState::init();

    let _ = MixpanelState::init();

    let current_post_params = RwSignal::new(None::<PostParams>);
    provide_context(current_post_params);

    #[cfg(feature = "hydrate")]
    {
        Effect::new(move |_| {
            let loc = use_location();
            history_ctx.push(&loc.pathname.get());
        });
    }

    // Analytics
    #[cfg(feature = "ga4")]
    {
        provide_context(EventHistory::default());
    }

    view! {
        <Title text=app_state.name />

        // Favicon
        <Link
            rel="icon"
            type_="image/svg+xml"
            href=format!("/{}/favicon.svg", app_state.asset_path())
        />
        <Link rel="shortcut icon" href=format!("/{}/favicon.ico", app_state.asset_path()) />
        <Link
            rel="apple-touch-icon"
            sizes="180x180"
            href=format!("/{}/favicon-apple.png", app_state.asset_path())
        />

        // Meta
        <Meta name="apple-mobile-web-app-title" content=app_state.name />

        // App manifest
        <Link rel="manifest" href=format!("/{}/manifest.json", app_state.asset_path()) />

        <Router>
            <main class="bg-black" id="body">
                <Routes fallback=|| view! { <NotFound /> }.into_view()>
                    // auth redirect routes exist outside main context
                    <GoogleAuthRedirectHandlerRoute />
                    <Route path=path!("/") view=YralRootPage />
                    <ParentRoute path=path!("") view=BaseRoute>
                        // TODO: enable when SATS are added back
                        // <Route
                        //     path=path!("/hot-or-not/withdraw")
                        //     view=hon::withdrawal::HonWithdrawal
                        // />
                        // <Route
                        //     path=path!("/hot-or-not/withdraw/success")
                        //     view=hon::withdrawal::result::Success
                        // />
                        // <Route
                        //     path=path!("/hot-or-not/withdraw/failure")
                        //     view=hon::withdrawal::result::Failure
                        // />
                        <Route path=path!("/hot-or-not/:canister_id/:post_id") view=PostView />
                        <Route path=path!("/post/:canister_id/:post_id") view=SinglePost />
                        <Route path=path!("/profile/:canister_id/post/:post_id") view=ProfilePost />
                        <Route path=path!("/upload") view=UploadPostPage />
                        <Route path=path!("/upload-ai") view=UploadAiPostPage />
                        <Route path=path!("/upload-options") view=UploadOptionsPage />
                        <Route path=path!("/error") view=ServerErrorPage />
                        <Route path=path!("/menu") view=Menu />
                        <Route path=path!("/settings") view=Settings />
                        <Route path=path!("/settings/:action") view=Settings />
                        <Route path=path!("/refer-earn") view=ReferEarn />
                        <Route path=path!("/profile/edit") view=ProfileEdit />
                        <Route path=path!("/profile/edit/username") view=ProfileUsernameEdit />
                        <Route path=path!("/profile/:id/:tab") view=ProfileView />
                        <Route path=path!("/profile/:tab") view=LoggedInUserProfileView />
                        <Route path=path!("/terms-of-service") view=TermsOfService />
                        <Route path=path!("/privacy-policy") view=PrivacyPolicy />
                        <Route path=path!("/about-us") view=AboutUs />
                        <Route path=path!("/wallet/:id") view=Wallet />
                        <Route path=path!("/wallet") view=Wallet />
                        <Route path=path!("/leaderboard") view=Leaderboard />
                        <Route path=path!("/logout") view=Logout />
                        <Route
                            path=path!("/token/info/:token_root/:id")
                            view=TokenInfo
                        />
                        <Route path=path!("/token/info/:token_root") view=TokenInfo />
                        <Route path=path!("/token/transfer/:token_root") view=TokenTransfer />
                        <Route
                            path=path!("/pnd/withdraw")
                            view=pumpdump::withdrawal::PndWithdrawal
                        />
                        <Route
                            path=path!("/pnd/withdraw/success")
                            view=pumpdump::withdrawal::result::Success
                        />
                        <Route
                            path=path!("/pnd/withdraw/failure")
                            view=pumpdump::withdrawal::result::Failure
                        />
                        <Route path=path!("/terms-ios") view=TermsIos />
                        <Route path=path!("/terms-android") view=TermsAndroid />
                        <Route path=path!("/internal/clear-sats") view=ClearSats />
                    </ParentRoute>
                </Routes>

            </main>
            <nav>
                <NavBar />
            </nav>
        </Router>
    }
}
