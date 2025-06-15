use leptos::prelude::*;
use leptos_icons::Icon;

use utils::icon_gen;
/// Spinner with constant size
#[component]
pub fn Spinner() -> impl IntoView {
    view! {
        <div class="animate-spin border-solid rounded-full border-t-transparent border-white border-8 h-32 w-32"></div>
    }
}

/// Spinner that fits container
#[component]
pub fn SpinnerFit() -> impl IntoView {
    view! {
        <div class="animate-spin border-solid rounded-full border-t-transparent border-primary-600 border-8 w-full h-full"></div>
    }
}

/// Spinner that takes up the whole screen with black background
#[component]
pub fn FullScreenSpinner() -> impl IntoView {
    view! {
        <div class="h-screen w-screen grid grid-cols-1 bg-black justify-items-center place-content-center">
            <Spinner />
        </div>
    }
}

icon_gen!(
    SpinnerCircleIcon,
    view_box = "0 0 21 21",
    r###"
<path opacity="0.38" d="M20.5 10.5C20.5 16.0228 16.0228 20.5 10.5 20.5C4.97715 20.5 0.5 16.0228 0.5 10.5C0.5 4.97715 4.97715 0.5 10.5 0.5C16.0228 0.5 20.5 4.97715 20.5 10.5ZM3.61817 10.5C3.61817 14.3007 6.69927 17.3818 10.5 17.3818C14.3007 17.3818 17.3818 14.3007 17.3818 10.5C17.3818 6.69927 14.3007 3.61817 10.5 3.61817C6.69927 3.61817 3.61817 6.69927 3.61817 10.5Z" fill="#FCE6F2" style="fill:#FCE6F2;fill:color(display-p3 0.9882 0.9020 0.9490);fill-opacity:1;"/>
<path d="M18.9409 10.5C19.802 10.5 20.5124 9.79783 20.3787 8.94721C20.2563 8.16837 20.0419 7.40492 19.7388 6.67317C19.2362 5.45991 18.4997 4.35752 17.5711 3.42893C16.6425 2.50035 15.5401 1.76375 14.3268 1.2612C13.5951 0.9581 12.8316 0.743716 12.0528 0.621293C11.2022 0.487588 10.5 1.19803 10.5 2.05908C10.5 2.92014 11.2067 3.60061 12.0458 3.79402C12.4165 3.87948 12.7804 3.99574 13.1336 4.14202C13.9685 4.48786 14.7272 4.99477 15.3662 5.63381C16.0052 6.27285 16.5121 7.0315 16.858 7.86644C17.0043 8.21959 17.1205 8.58347 17.206 8.95421C17.3994 9.79326 18.0799 10.5 18.9409 10.5Z" fill="#FAFAFA" style="fill:#FAFAFA;fill:color(display-p3 0.9804 0.9804 0.9804);fill-opacity:1;"/>
"###
);
#[component]
pub fn SpinnerCircle(#[prop(optional, default = "")] class: &'static str) -> impl IntoView {
    view! { <Icon icon=SpinnerCircleIcon attr:class=format!("animate-spin w-full h-full {}", class) /> }
}
#[component]
pub fn SpinnerCircleStyled(#[prop(optional, default = "")] class: &'static str) -> impl IntoView {
    view! {
        <Icon
            icon=SpinnerCircleStyledIcon
            attr:class=format!("animate-spin w-full h-full {}", class)
        />
    }
}
icon_gen!(
    SpinnerCircleStyledIcon,
    view_box = "0 0 49 48",
    r###"
<path opacity="0.38" d="M48.5 24C48.5 37.2548 37.7548 48 24.5 48C11.2452 48 0.5 37.2548 0.5 24C0.5 10.7452 11.2452 0 24.5 0C37.7548 0 48.5 10.7452 48.5 24ZM7.98361 24C7.98361 33.1218 15.3782 40.5164 24.5 40.5164C33.6218 40.5164 41.0164 33.1218 41.0164 24C41.0164 14.8782 33.6218 7.48361 24.5 7.48361C15.3782 7.48361 7.98361 14.8782 7.98361 24Z" fill="#EC55A7"/>
<path d="M44.7582 24C46.8247 24 48.5298 22.3148 48.2089 20.2733C47.9151 18.4041 47.4006 16.5718 46.6731 14.8156C45.467 11.9038 43.6992 9.25804 41.4706 7.02944C39.242 4.80083 36.5962 3.033 33.6844 1.82689C31.9282 1.09944 30.0959 0.584918 28.2267 0.291103C26.1852 -0.0297888 24.5 1.67526 24.5 3.7418C24.5 5.80834 26.1962 7.44147 28.2099 7.90566C29.0997 8.11075 29.973 8.38977 30.8205 8.74084C32.8244 9.57087 34.6452 10.7875 36.1789 12.3211C37.7125 13.8548 38.9291 15.6756 39.7592 17.6794C40.1102 18.527 40.3892 19.4003 40.5943 20.2901C41.0585 22.3038 42.6917 24 44.7582 24Z" fill="url(#paint0_linear_134_12800)"/>
<defs>
<linearGradient id="paint0_linear_134_12800" x1="45.8334" y1="6.33333" x2="17.1668" y2="42.6668" gradientUnits="userSpaceOnUse">
<stop stop-color="#FF78C1"/>
<stop offset="0.509385" stop-color="#E2017B"/>
<stop offset="1" stop-color="#5F0938"/>
</linearGradient>
</defs>

    "###
);
