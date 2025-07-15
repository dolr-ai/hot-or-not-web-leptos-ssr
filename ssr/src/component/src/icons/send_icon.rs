use leptos::{either::Either, prelude::*};

#[component]
pub fn SendIcon(
    #[prop(into, optional, default = "w-full h-full".into())] class: String,
    #[prop(optional)] filled: bool,
) -> impl IntoView {
    if filled {
        Either::Left(view! {
            <svg
                class=format!("{}", class)
                viewBox="0 0 19 18"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
            >
                <path
                    d="M7.83362 10.2583L11.5459 16.0919L16.3189 1.77297L1.99999 6.54594L7.83362 10.2583ZM7.83362 10.2583L12.0763 6.01561"
                    stroke="currentColor"
                    stroke-width="1.5"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                />
            </svg>
        })
    } else {
        Either::Right(view! {
            <svg
                class=format!("{}", class)
                viewBox="0 0 19 18"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
            >
                <g clip-path="url(#clip0_335_556)">
                    <path
                        d="M7.83362 10.2583L11.5459 16.0919L16.3189 1.77297L1.99999 6.54594L7.83362 10.2583ZM7.83362 10.2583L12.0763 6.01561"
                        stroke="currentColor"
                        stroke-width="1.5"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    />
                </g>
                <defs>
                    <clipPath id="clip0_335_556">
                        <rect width="18" height="18" fill="white" transform="translate(0.5)" />
                    </clipPath>
                </defs>
            </svg>
        })
    }
}
