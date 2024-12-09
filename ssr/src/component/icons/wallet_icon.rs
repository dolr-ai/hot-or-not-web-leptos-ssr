use leptos::*;

#[component]
pub fn WalletIcon(
    #[prop(optional, default = "w-full h-full".to_string())] classes: String,
) -> impl IntoView {
    view! {
        <svg
            class=format!("{}", classes)
            viewBox="0 0 18 19"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <path
                fill-rule="evenodd"
                clip-rule="evenodd"
                d="M10.6341 2.82015C9.9834 1.94194 8.70273 1.69553 7.758 2.38834L3.83204 5.26737C2.60119 5.56719 1.6875 6.67693 1.6875 8.00007V14.0001C1.6875 15.5534 2.9467 16.8126 4.5 16.8126H13.5C15.0533 16.8126 16.3125 15.5534 16.3125 14.0001V13.3599C17.1782 13.1151 17.8125 12.3192 17.8125 11.3751C17.8125 10.431 17.1782 9.63506 16.3125 9.39021V8.00007C16.3125 6.89468 15.6748 5.93823 14.7472 5.47854L13.9819 3.56533C13.4853 2.32363 11.9564 1.87568 10.8681 2.653L10.6341 2.82015ZM5.84324 5.18757H7.31974L9.71767 3.47477C9.41811 3.0895 8.84653 2.98516 8.42328 3.29554L5.84324 5.18757ZM11.522 3.56845C12.0167 3.21512 12.7116 3.41874 12.9374 3.98314L13.4192 5.18757H9.25526L11.522 3.56845ZM15.1875 13.4376H14.25C13.1109 13.4376 12.1875 12.5142 12.1875 11.3751C12.1875 10.236 13.1109 9.31257 14.25 9.31257H15.1875V8.00007C15.1875 7.06809 14.432 6.31257 13.5 6.31257H4.5C3.56802 6.31257 2.8125 7.06809 2.8125 8.00007V14.0001C2.8125 14.9321 3.56802 15.6876 4.5 15.6876H13.5C14.432 15.6876 15.1875 14.9321 15.1875 14.0001V13.4376ZM14.25 10.4376C13.7322 10.4376 13.3125 10.8573 13.3125 11.3751C13.3125 11.8928 13.7322 12.3126 14.25 12.3126H15.75C16.2678 12.3126 16.6875 11.8928 16.6875 11.3751C16.6875 10.8573 16.2678 10.4376 15.75 10.4376H14.25Z"
                fill="currentColor"
                style="fill:#A0A1A6;fill:color(display-p3 0.6275 0.6314 0.6510);fill-opacity:1;"
            />
        </svg>
    }
}
