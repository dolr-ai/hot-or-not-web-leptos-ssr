use leptos::*;

#[component]
pub fn SendIcon(#[prop(optional, default = "w-full h-full".to_string())] classes: String, #[prop(optional)] filled: bool) -> impl IntoView {
    view! {
        <svg class=format!("{}" ,classes) viewBox="0 0 31 30" fill="none" xmlns="http://www.w3.org/2000/svg">
            <mask
                id="mask0_500_15024"
                style="mask-type:alpha"
                maskUnits="userSpaceOnUse"
                x="0"
                y="0"
                width="31"
                height="30"
            >
                <rect x="0.5" width="30" height="30" fill="currentColor" />
            </mask>
            <g mask="url(#mask0_500_15024)">
                <path
                    d="M18.9781 12.0763L18.978 12.0763L18.9776 12.0768L18.9775 12.0769C18.9304 12.1301 16.5743 14.7906 15.5453 15.9467C15.4021 16.1077 15.3455 16.3697 15.4085 16.5726L17.1255 22.1025C17.2376 22.4634 17.7416 22.4819 17.8798 22.1301L24.1026 6.29093C24.2304 5.9658 23.9092 5.64463 23.5841 5.77237L7.74489 11.9952C7.39309 12.1334 7.41156 12.6374 7.77254 12.7495L13.3015 14.4663C13.5048 14.5294 13.7676 14.4726 13.9289 14.329C15.0897 13.296 17.7645 10.9273 17.7986 10.8971L17.7987 10.897L17.7988 10.8969L17.8014 10.8946L18.9781 12.0763ZM18.9781 12.0763L18.9809 12.0731C18.981 12.073 18.981 12.073 18.981 12.0729M18.9781 12.0763L18.981 12.0729M18.0798 11.2806L18.0797 11.2807M18.0798 11.2806L18.08 11.2804L18.0797 11.2807M18.0798 11.2806C18.0798 11.2807 18.0797 11.2807 18.0797 11.2807M18.0798 11.2806L18.0797 11.2807M18.981 12.0729C18.9811 12.0729 18.9811 12.0729 18.9811 12.0729M18.981 12.0729L18.9811 12.0729M18.9811 12.0729C19.3318 11.674 19.1903 11.1832 18.9373 10.933M18.9811 12.0729L18.9373 10.933M18.9373 10.933C18.6877 10.6859 18.1985 10.5451 17.8016 10.8944L18.9373 10.933Z"
                    stroke:none=move || !filled
                    stroke:currentColor=move || filled
                    fill:none=move || !filled
                    fill:currentColor=move || filled
                    stroke-width="1.2"
                />
            </g>
        </svg>
    }
	}