use component::skeleton::Skeleton;
use leptos::{prelude::*, view, IntoView};
#[component]
pub fn CardSkeleton() -> impl IntoView {
    view! {
        <div class="w-full min-h-124 snap-start snap-always rounded-2xl bg-neutral-900 flex flex-col items-center px-5">
            <div class="flex flex-col pt-14 items-center gap-6 w-full">
                <Skeleton class="size-32 rounded text-neutral-800 [--shimmer:#363636]" />
                <Skeleton class="h-5 w-24 rounded text-neutral-800 [--shimmer:#363636]" />
                <Skeleton class="h-11 w-full rounded-full text-neutral-800 [--shimmer:#363636]" />
            </div>
        </div>
    }
}
