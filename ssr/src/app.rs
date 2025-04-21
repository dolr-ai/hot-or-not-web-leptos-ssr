use component::content_upload::AuthorizedUserToSeedContent;
use page::about_us::AboutUs;
use page::icpump::ai::ICPumpAi;
use page::icpump::ICPumpLanding;
use page::post_view::PostDetailsCacheCtx;
use page::pumpdump::{withdrawal, PndProfilePage};
use state::app_type::AppType;
// use crate::page::wallet::TestIndex;
use crate::error_template::{AppError, ErrorTemplate};
use component::{base_route::BaseRoute, nav::NavBar};
use page::{
    err::ServerErrorPage,
    leaderboard::Leaderboard,
    logout::Logout,
    menu::Menu,
    post_view::{single_post::SinglePost, PostView, PostViewCtx},
    privacy::PrivacyPolicy,
    profile::{profile_post::ProfilePost, ProfilePostsContext, ProfileView},
    refer_earn::ReferEarn,
    root::RootPage,
    settings::Settings,
    terms::TermsOfService,
    token::{
        create::{CreateToken, CreateTokenCtx, CreateTokenSettings},
        create_token_faq::CreateTokenFAQ,
        info::TokenInfo,
        transfer::TokenTransfer,
    },
    upload::UploadPostPage,
    wallet::Wallet,
};
use state::app_state::AppState;

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::hooks::use_location;
use leptos_router::{components::*, path, MatchNestedRoutes};
use page::terms_ios::TermsIos;
use state::{audio_state::AudioState, content_seed_client::ContentSeedClient};
use utils::event_streaming::events::HistoryCtx;
use utils::event_streaming::EventHistory;
use yral_canisters_common::Canisters;

#[component]
fn NotFound() -> impl IntoView {
    let mut outside_errors = Errors::default();
    outside_errors.insert_with_default_key(AppError::NotFound);
    view! { <ErrorTemplate outside_errors/> }
}

#[component(transparent)]
fn GoogleAuthRedirectHandlerRoute() -> impl MatchNestedRoutes + Clone {
    let path = path!("/auth/google_redirect");
    #[cfg(any(feature = "oauth-ssr", feature = "oauth-hydrate"))]
    {
        // if show_preview_component() {
        //     use crate::page::preview_google_redirect::PreviewGoogleRedirectHandler;
        //     view! { <Route path view=PreviewGoogleRedirectHandler/> }.into_inner()
        // } else {

        // }
        use page::google_redirect::GoogleRedirectHandler;
        view! { <Route path view=GoogleRedirectHandler/> }.into_inner()
    }
    #[cfg(not(any(feature = "oauth-ssr", feature = "oauth-hydrate")))]
    {
        view! { <Route path view=NotFound/> }.into_inner()
    }
}

#[component(transparent)]
fn GoogleAuthRedirectorRoute() -> impl MatchNestedRoutes + Clone {
    let path = path!("/auth/perform_google_redirect");
    #[cfg(any(feature = "oauth-ssr", feature = "oauth-hydrate"))]
    {
        // if show_preview_component() {
        //     use crate::page::preview_google_redirect::PreviewGoogleRedirector;
        //     view! { <Route path view=PreviewGoogleRedirector/> }.into_inner()
        // } else {

        // }
        use page::google_redirect::GoogleRedirector;
        view! { <Route path view=GoogleRedirector/> }.into_inner()
    }
    #[cfg(not(any(feature = "oauth-ssr", feature = "oauth-hydrate")))]
    {
        view! { <Route path view=NotFound/> }.into_inner()
    }
}

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <Script async_="true">
                    {r#"
                    (function(w,d,s,l,i){
                        w[l]=w[l]||[];
                        w[l].push({'gtm.start': new Date().getTime(),event:'gtm.js'});
                        var f=d.getElementsByTagName(s)[0], 
                        j=d.createElement(s),dl=l!='dataLayer'?'&l='+l:'';
                        j.async=true;
                        j.src='https://www.googletagmanager.com/gtm.js?id='+i+dl;
                        f.parentNode.insertBefore(j,f);
                    })(window,document,'script','dataLayer','GTM-MNBWSPVJ');
                    "#}
                </Script>
                <AutoReload options=options.clone() />
                <HashedStylesheet id="leptos" options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
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
    provide_context(PostViewCtx::default());
    provide_context(ProfilePostsContext::default());
    provide_context(AuthorizedUserToSeedContent::default());
    provide_context(AudioState::default());
    provide_context(CreateTokenCtx::default());
    provide_context(PostDetailsCacheCtx::default());

    // History Tracking
    let history_ctx = HistoryCtx::default();
    provide_context(history_ctx.clone());

    #[cfg(feature = "hydrate")]
    {
        Effect::new(move |_| {
            let loc = use_location();
            history_ctx.push(&loc.pathname.get());
        });
    }

    // Analytics
    let enable_ga4_script = RwSignal::new(false);
    #[cfg(feature = "ga4")]
    {
        enable_ga4_script.set(true);
        provide_context(EventHistory::default());
    }

    view! {

            <Title text=app_state.name/>

            // Favicon
            <Link rel="icon" type_="image/svg+xml" href=format!("/{}/favicon.svg", app_state.asset_path()) />
            <Link rel="shortcut icon" href=format!("/{}/favicon.ico", app_state.asset_path()) />
            <Link rel="apple-touch-icon" sizes="180x180" href=format!("/{}/favicon-apple.png", app_state.asset_path()) />

            // Meta
            <Meta name="apple-mobile-web-app-title" content=app_state.name />

            // App manifest
            <Link rel="manifest" href=format!("/{}/manifest.json", app_state.asset_path())/>

            // GA4 Global Site Tag (gtag.js) - Google Analytics
            // G-6W5Q2MRX0E to test locally | G-PLNNETMSLM
            <Show when=enable_ga4_script>
                <Script
                    async_="true"
                    src=concat!("https://www.googletagmanager.com/gtag/js?id=", "G-PLNNETMSLM")
                />
                <Script>
                    {r#"
                window.dataLayer = window.dataLayer || [];
                function gtag(){dataLayer.push(arguments);}
                gtag('js', new Date());
                gtag('config', 'G-PLNNETMSLM');
                "#}
                </Script>
            </Show>
            <Script
            async_="true"
            src="https://sentry.yral.com/js-sdk-loader/3f7d672f8461961bd7b6bec57acf7f18.min.js"
            crossorigin="anonymous"
            ></Script>

            <Router>
            <main class="bg-black" id="body">
                <Routes fallback=|| view! { <NotFound/> }.into_view()>
                    // auth redirect routes exist outside main context
                    <GoogleAuthRedirectHandlerRoute/>
                    <GoogleAuthRedirectorRoute/>
                    <ParentRoute path=path!("") view=BaseRoute>
                        <Route path=path!("/") view=RootPage/>
                        <Route path=path!("/hot-or-not/:canister_id/:post_id") view=PostView/>
                        <Route path=path!("/post/:canister_id/:post_id") view=SinglePost/>
                        <Route path=path!("/profile/:canister_id/post/:post_id") view=ProfilePost/>
                        <Route path=path!("/pnd/profile") view=PndProfilePage/>
                        <Route path=path!("/upload") view=UploadPostPage/>
                        <Route path=path!("/error") view=ServerErrorPage/>
                        <Route path=path!("/menu") view=Menu/>
                        <Route path=path!("/settings") view=Settings/>
                        <Route path=path!("/refer-earn") view=ReferEarn/>
                        <Route path=path!("/profile/:id/:tab") view=ProfileView/>
                        <Route path=path!("/profile/:tab") view=ProfileView/>
                        <Route path=path!("/terms-of-service") view=TermsOfService/>
                        <Route path=path!("/privacy-policy") view=PrivacyPolicy/>
                        <Route path=path!("/about-us") view=AboutUs/>
                        <Route path=path!("/wallet/:id") view=Wallet/>
                        <Route path=path!("/wallet") view=Wallet/>
                        <Route path=path!("/leaderboard") view=Leaderboard/>
                        <Route path=path!("/logout") view=Logout/>
                        <Route path=path!("/token/create") view=CreateToken/>
                        <Route path=path!("/token/create/settings") view=CreateTokenSettings/>
                        <Route path=path!("/token/create/faq") view=CreateTokenFAQ/>
                        <Route path=path!("/token/info/:token_root/:key_principal") view=TokenInfo/>
                        <Route path=path!("/token/info/:token_root") view=TokenInfo/>
                        <Route path=path!("/token/transfer/:token_root") view=TokenTransfer/>
                        <Route path=path!("/board") view=ICPumpLanding/>
                        <Route path=path!("/icpump-ai") view=ICPumpAi/>
                        <Route path=path!("/pnd/withdraw") view=withdrawal::PndWithdrawal />
                        <Route path=path!("/pnd/withdraw/success") view=withdrawal::result::Success />
                        <Route path=path!("/pnd/withdraw/failure") view=withdrawal::result::Failure />
                        <Route path=path!("/terms-ios") view=TermsIos/>
                        // {
                        //     #[cfg(any(feature = "local-bin", feature = "local-lib"))]
                        //     view! {
                        //         <Route path=path!("/pnd/test/:token_root") view=crate::page::pumpdump::PndTest />
                        //     }
                        // }
                    // <Route path="/test" view=TestIndex/>
                    </ParentRoute>
                </Routes>

            </main>
            <nav>
                <NavBar/>
            </nav>
        </Router>
    }
}
