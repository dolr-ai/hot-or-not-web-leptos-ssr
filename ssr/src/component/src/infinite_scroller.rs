use super::bullet_loader::BulletLoader;
use leptos::{html::ElementType, prelude::*};
use leptos_use::{
    core::{IntoElementsMaybeSignal, SignalVecMarker},
    use_intersection_observer_with_options, UseIntersectionObserverOptions,
};
use std::marker::PhantomData;
use yral_canisters_common::cursored_data::{CursoredDataProvider, KeyedData, PageEntry};

pub type InferData<T> = <T as CursoredDataProvider>::Data;

/// Infinite scroller which fetches data from provider
/// and renders children
/// It will fetch new data whenever the end of the list is reached
/// also shows a loader while fetching data
/// node_ref MUST be passed to the root element if passed to the `children` fn
#[component]
pub fn InfiniteScroller<Prov, EF, N, RootNode>(
    provider: Prov,
    fetch_count: usize,
    children: EF,
    #[prop(optional, into)] empty_content: ViewFn,
    #[prop(optional, into)] custom_loader: Option<ViewFn>,
    #[prop(optional)] _iv: PhantomData<N>,
    #[prop(optional)] _rn: PhantomData<RootNode>,
) -> impl IntoView
where
    RootNode: ElementType + Clone + 'static,
    NodeRef<RootNode>: IntoElementsMaybeSignal<leptos::web_sys::Element, SignalVecMarker>,
    Prov: CursoredDataProvider + Clone + 'static + Send + Sync,
    <Prov as CursoredDataProvider>::Data: Send + Sync,
    EF: Fn(InferData<Prov>, Option<NodeRef<RootNode>>) -> N + Clone + 'static + Send + Sync,
    N: IntoView + 'static,
{
    let data = RwSignal::new(Vec::<InferData<Prov>>::new());
    let end = RwSignal::new(false);
    let loading = RwSignal::new(false);
    let cursor = RwSignal::new(0);

    let fetch_more = Action::new(move |_: &()| {
        let provider = provider.clone();
        let current_cursor = cursor.get_untracked();
        async move {
            loading.set(true);
            let PageEntry {
                data: mut fetched,
                end: list_end,
            } = match provider
                .get_by_cursor(current_cursor, current_cursor + fetch_count)
                .await
            {
                Ok(t) => t,
                Err(e) => {
                    log::warn!("failed to fetch data err {e}");
                    PageEntry {
                        data: vec![],
                        end: true,
                    }
                }
            };

            if !fetched.is_empty() {
                cursor.update(|c| *c += fetched.len());
            }

            data.update(|d| d.append(&mut fetched));
            end.set(list_end);
            loading.set(false);
        }
    });

    // Initial load
    Effect::new(move |_| {
        fetch_more.dispatch(());
    });

    let last_elem = NodeRef::<RootNode>::new();

    use_intersection_observer_with_options(
        last_elem,
        move |entry, _| {
            let Some(_visible) = entry.first().filter(|entry| entry.is_intersecting()) else {
                return;
            };

            let is_loading = loading.get_untracked();
            let reached_end = end.get_untracked();

            if !is_loading && !reached_end {
                fetch_more.dispatch(());
            }
        },
        UseIntersectionObserverOptions::default().thresholds(vec![0.1]),
    );

    let children = StoredValue::new(children);
    let loader = custom_loader.unwrap_or_else(|| BulletLoader.into());

    view! {
        <For
            each=move || data.get()
            key=KeyedData::key
            children=move |item| {
                let is_last = data
                    .with(|d| {
                        d
                            .last()
                            .map(|last| KeyedData::key(last) == KeyedData::key(&item))
                            .unwrap_or(false)
                    });
                (children.get_value())(item, if is_last { Some(last_elem) } else { None })
            }
        />

        <Show when=move || loading.get()>{loader.run()}</Show>

        <Show when=move || {
            !loading.get() && data.with(|d| d.is_empty())
        }>{empty_content.run()}</Show>
    }
}
