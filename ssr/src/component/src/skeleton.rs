use leptos::prelude::*;

/// Component for creating skeleton loaders, Usage is similar to shadcn's skeleton.
/// Every instance of Skeleton shimmers in unison across the page, and doesn't
/// need any additional configuration.
///
/// To control the background color use `text-*`. To control the color of the
/// shimmer use [--shimmer:color]
///
/// # Examples
///
/// ```tsx
/// <Skeleton class="text-neutral-300 [--shimmer:#FF0] w-1/5 h-4 rounded-sm" />
/// ```
///
/// ```tsx
/// <div class="flex flex-col w-full gap-2 items-center">
///   <Skeleton class="text-neutral-400 size-8 rounded-full" />
///   <Skeleton class="text-neutral-400 w-2/5 h-4 rounded-sm" />
/// </div>
/// ```
#[component]
pub fn Skeleton(#[prop(into)] class: String) -> impl IntoView {
    view! {
        <div class=format!(
            "animate-skeleton-shimmer bg-skeleton-shimmer bg-skeleton-shimmer bg-fixed {}",
            class,
        )></div>
    }
}
