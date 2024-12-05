use leptos::*;

#[component]
pub fn ArrowLeftIcon(#[prop(optional, default = "w-full h-full".to_string())] classes: String) -> impl IntoView {
    view! {
        <svg class=format!("{}" ,classes) viewBox="0 0 31 30" fill="none" xmlns="http://www.w3.org/2000/svg">
            <mask
                id="mask0_500_15031"
                style="mask-type:alpha"
                maskUnits="userSpaceOnUse"
                x="0"
                y="0"
                width="31"
                height="30"
            >
                <rect x="0.75" width="30" height="30" fill="currentColor" />
            </mask>
            <g mask="url(#mask0_500_15031)">
                <path
                    d="M10.9024 22.3831C10.6176 22.6398 10.1854 22.6413 9.8988 22.3866L6.27834 19.1684C5.83082 18.7706 5.83082 18.0714 6.27834 17.6736L9.89869 14.4555C10.1853 14.2007 10.6177 14.2023 10.9024 14.4591C11.2354 14.7593 11.2335 15.2824 10.8984 15.5802L8.50245 17.7098H17.8106C18.2034 17.7098 18.5218 18.0282 18.5218 18.421C18.5218 18.8138 18.2034 19.1322 17.8106 19.1322H8.50245L10.8984 21.262C11.2336 21.5599 11.2354 22.0829 10.9024 22.3831ZM21.6013 16.9979C21.3147 17.2527 20.8823 17.2511 20.5975 16.9943C20.2646 16.6941 20.2664 16.1711 20.6015 15.8732L22.9976 13.7434H13.6893C13.2966 13.7434 12.9782 13.425 12.9782 13.0323C12.9782 12.6396 13.2966 12.3212 13.6893 12.3212H22.9976L20.6016 10.1914C20.2664 9.89355 20.2646 9.37054 20.5975 9.07027C20.8823 8.81346 21.3147 8.81192 21.6013 9.06669L25.2217 12.2848C25.6692 12.6826 25.6692 13.3818 25.2217 13.7796L21.6013 16.9979Z"
                    fill="currentColor"
                />
            </g>
        </svg>
    }
	}