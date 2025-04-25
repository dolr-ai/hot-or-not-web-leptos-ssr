use candid::Principal;
#[cfg(all(feature = "cloudflare", feature = "ssr"))]
use cf_impl::server_func::*;
#[cfg(feature = "cloudflare")]
pub use cf_impl::{publish_video, upload_video_stream};
use leptos::prelude::*;
#[cfg(all(not(feature = "cloudflare"), feature = "ssr"))]
use mock_impl::server_func::*;
#[cfg(not(feature = "cloudflare"))]
pub use mock_impl::{publish_video, upload_video_stream};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct UploadInfo {
    pub uid: String,
    pub upload_url: String,
}

#[server(GetUploadInfo)]
pub async fn get_upload_info(
    creator: Principal,
    hashtags: Vec<String>,
    description: String,
    file_name: String,
) -> Result<UploadInfo, ServerFnError> {
    // TODO(SECURITY): authenticate creator

    if description.len() < 10 {
        return Err(ServerFnError::Args(
            "Description must be at least 10 characters".into(),
        ));
    }
    if hashtags.len() > 8 {
        return Err(ServerFnError::Args("Too many hashtags".into()));
    }

    get_upload_info_impl(creator, hashtags, description, file_name).await
}

#[server(GetVideoStatus)]
pub async fn get_video_status(uid: String) -> Result<String, ServerFnError> {
    get_video_status_impl(uid).await
}

#[cfg(feature = "cloudflare")]
mod cf_impl {
    use leptos::prelude::ServerFnError;
    use yral_canisters_client::individual_user_template::{PostDetailsFromFrontend, Result1};

    use yral_canisters_common::Canisters;

    use super::UploadInfo;

    #[cfg(feature = "ssr")]
    pub mod server_func {
        use candid::Principal;
        use gob_cloudflare::{
            api::stream_videos::{CreateDownloads, DirectUpload, VideoDetails},
            CloudflareAuth,
        };
        use leptos::prelude::*;

        use consts::CF_WATERMARK_UID;

        use super::UploadInfo;
        use std::time::Duration;

        pub async fn get_upload_info_impl(
            creator: Principal,
            hashtags: Vec<String>,
            description: String,
            file_name: String,
        ) -> Result<UploadInfo, ServerFnError> {
            let cf_api: CloudflareAuth = expect_context();
            let req = DirectUpload::default()
                .creator(creator.to_text())
                .add_meta("hashtags", hashtags.join(","))
                .add_meta("description", description)
                .add_meta("fileName", file_name)
                .add_meta("uploadType", "challenge")
                .watermark(CF_WATERMARK_UID)
                .max_duration(Duration::from_secs(60));
            let res = cf_api.send_auth(req).await?;

            Ok(UploadInfo {
                uid: res.uid,
                upload_url: res.upload_url,
            })
        }

        pub async fn get_video_status_impl(uid: String) -> Result<String, ServerFnError> {
            let cf_api: CloudflareAuth = expect_context();
            let req = VideoDetails::new(uid.clone());
            let res = cf_api.send_auth(req).await?;
            let state = res.status.state;
            if state != "ready" {
                return Ok(state);
            }
            let req = CreateDownloads::new(uid);
            _ = cf_api.send_auth(req).await?;

            Ok(state)
        }
    }

    pub async fn upload_video_stream(
        _upload_res: &UploadInfo,
        _file: &gloo::file::File,
    ) -> Result<(), gloo::net::Error> {
        #[cfg(feature = "hydrate")]
        {
            use gloo::net::http::Request;
            use leptos::web_sys::FormData;
            let form = FormData::new().unwrap();
            form.append_with_blob("file", _file.as_ref()).unwrap();
            let req = Request::post(&_upload_res.upload_url).body(form).unwrap();
            req.send().await?;
        }
        Ok(())
    }

    pub async fn publish_video(
        canisters: Canisters<true>,
        hashtags: Vec<String>,
        description: String,
        uid: String,
        enable_hot_or_not: bool,
        is_nsfw: bool,
    ) -> Result<u64, ServerFnError> {
        let user = canisters.authenticated_user().await;
        let res = user
            .add_post_v_2(PostDetailsFromFrontend {
                hashtags,
                description,
                video_uid: uid,
                creator_consent_for_inclusion_in_hot_or_not: enable_hot_or_not,
                is_nsfw,
            })
            .await?;
        let post_id = match res {
            Result1::Ok(p) => p,
            Result1::Err(e) => return Err(ServerFnError::new(e)),
        };
        user.update_post_as_ready_to_view(post_id).await?;
        Ok(post_id)
    }
}

#[cfg(not(feature = "cloudflare"))]
mod mock_impl {
    use super::UploadInfo;
    use leptos::prelude::*;
    use yral_canisters_common::Canisters;

    #[cfg(feature = "ssr")]
    pub mod server_func {
        use candid::Principal;
        use leptos::prelude::*;
        use std::time::Duration;

        use super::UploadInfo;

        pub async fn get_upload_info_impl(
            _creator: Principal,
            _hashtags: Vec<String>,
            _description: String,
            _file_name: String,
        ) -> Result<UploadInfo, ServerFnError> {
            Ok(UploadInfo {
                uid: "mock".into(),
                upload_url: "http://mock.com".into(),
            })
        }

        pub async fn get_video_status_impl(_uid: String) -> Result<String, ServerFnError> {
            tokio::time::sleep(Duration::from_secs(2)).await;
            Ok("ready".into())
        }
    }

    pub async fn upload_video_stream(
        _upload_res: &UploadInfo,
        _file: &gloo::file::File,
    ) -> Result<(), gloo::net::Error> {
        use gloo::timers::future::TimeoutFuture;
        TimeoutFuture::new(1000).await;
        Ok(())
    }

    pub async fn publish_video(
        _canisters: Canisters<true>,
        _hashtags: Vec<String>,
        _description: String,
        _uid: String,
        _enable_hot_or_not: bool,
        _is_nsfw: bool,
    ) -> Result<u64, ServerFnError> {
        use gloo::timers::future::TimeoutFuture;
        TimeoutFuture::new(1000).await;
        Ok(0)
    }
}
