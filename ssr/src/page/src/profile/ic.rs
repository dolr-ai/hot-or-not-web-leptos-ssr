use leptos::prelude::*;
use leptos_icons::*;
use utils::profile::PROFILE_CHUNK_SZ;
use yral_canisters_common::cursored_data::CursoredDataProvider;

use component::{
    bullet_loader::BulletLoader,
    infinite_scroller::{InferData, InfiniteScroller},
};
use leptos::html;
#[component]
pub fn ProfileStream<Prov, EF, N>(
    provider: Prov,
    children: EF,
    empty_graphic: icondata::Icon,
    #[prop(into)] empty_text: String,
) -> impl IntoView
where
    Prov: CursoredDataProvider + Clone + Send + Sync + 'static,
    Prov::Data: Send + Sync,
    EF: Fn(InferData<Prov>, Option<NodeRef<html::Div>>) -> N + Clone + Send + Sync + 'static,
    N: IntoView + 'static,
{
    view! {
        <div class="flex flex-row flex-wrap gap-y-3 justify-center w-full">
            <InfiniteScroller
                provider
                fetch_count=PROFILE_CHUNK_SZ
                children
                empty_content=move || {
                    view! {
                        <div class="flex flex-col gap-2 justify-center items-center pt-9 w-full">
                            <Icon attr:class="w-36 h-36" icon=empty_graphic />
                            <span class="text-lg text-white">{empty_text.clone()}</span>
                        </div>
                    }
                }

                custom_loader=move || {
                    view! {
                        <div class="flex justify-center items-center pt-9 w-full">
                            <BulletLoader />
                        </div>
                    }
                }
            />

        </div>
    }
}
