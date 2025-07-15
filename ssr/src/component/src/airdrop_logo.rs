use leptos::prelude::*;

#[component]
pub fn AirdropLogo() -> impl IntoView {
    view! {
        <svg class="w-full h-full" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 265 255">
            <g fill="#F47421" clip-path="url(#a)">
                <path
                    d="M39 193.2c0 9 41.8 16.2 93.5 16.2s93.5-7.3 93.5-16.2c0-9-41.8-16.3-93.5-16.3S39 184.2 39 193.2Z"
                    opacity=".2"
                    style="mix-blend-mode:screen"
                ></path>
                <path
                    d="M5 200c0 13.8 57 25 127.5 25 70.4 0 127.5-11.2 127.5-25s-57-25-127.5-25C62.1 175 5 186.2 5 200Z"
                    opacity=".1"
                    style="mix-blend-mode:screen"
                ></path>
                <path
                    d="M68.9 188c0 4.6 28.5 8.5 63.6 8.5s63.6-3.9 63.6-8.6c0-4.7-28.5-8.5-63.6-8.5s-63.6 3.8-63.6 8.5Z"
                    opacity=".4"
                    style="mix-blend-mode:screen"
                ></path>
                <path
                    d="M132.5 180.9c18.5 0 33.5 2 33.5 4.5s-15 4.5-33.5 4.5-33.5-2-33.5-4.5 15-4.5 33.5-4.5Z"
                    opacity=".5"
                    style="mix-blend-mode:screen"
                ></path>
            </g>
            <g opacity=".7">
                <path
                    fill="url(#b)"
                    d="M2 37.8h-.3c-1 0-1.7.6-1.7 1.3v101.5c0 .7.7 1.3 1.7 1.3H2c1 0 1.7-.6 1.7-1.3V39.1c0-.7-.8-1.3-1.7-1.3Z"
                    style="mix-blend-mode:color-dodge"
                ></path>
                <path
                    fill="url(#c)"
                    d="M47.1 110.3h-.3c-1 0-1.7.5-1.7 1.3v60.7c0 .7.8 1.2 1.7 1.2h.3c1 0 1.7-.5 1.7-1.2v-60.7c0-.8-.7-1.3-1.7-1.3Z"
                ></path>
                <path
                    fill="url(#d)"
                    d="M263.3 16.6h-.3c-1 0-1.7.6-1.7 1.3v60.7c0 .7.8 1.3 1.7 1.3h.3c1 0 1.7-.6 1.7-1.3V17.9c0-.7-.7-1.3-1.7-1.3Z"
                    style="mix-blend-mode:color-dodge"
                ></path>
                <path
                    fill="url(#e)"
                    d="M87.2 1h-.4c-1 0-1.8.5-1.8 1v49c0 .5.8 1 1.8 1h.4c1 0 1.8-.5 1.8-1V2c0-.5-.8-1-1.8-1Z"
                ></path>
                <path
                    fill="url(#f)"
                    d="M236.4 112.3h-.3c-1 0-1.7.6-1.7 1.3v60.7c0 .7.8 1.3 1.7 1.3h.3c1 0 1.7-.6 1.7-1.3v-60.7c0-.7-.8-1.3-1.7-1.3Z"
                ></path>
                <path
                    fill="url(#g)"
                    d="M210.6 38h-.2c-.8 0-1.4.4-1.4 1v75c0 .6.6 1 1.4 1h.2c.8 0 1.4-.4 1.4-1V39c0-.6-.6-1-1.4-1Z"
                ></path>
            </g>
            <path
                fill="url(#h)"
                d="M184.2 95.5c-3-25.1-28.9-42.8-57.8-39.4-29 3.4-50 26.6-47 51.7l1.4 11c3 25.2 28.7 41 57.6 37.6 29-3.4 50.2-24.7 47.1-49.8L184.3 96v-.6Z"
            ></path>
            <mask
                id="i"
                width="107"
                height="63"
                x="79"
                y="94"
                maskUnits="userSpaceOnUse"
                style="mask-type:luminance"
            >
                <path
                    fill="#fff"
                    d="M185.6 106.6c3 25.1-18.3 46.4-47.2 49.8-29 3.4-54.6-12.4-57.6-37.5l-1.5-12.3 104.8-12.3 1.5 12.3Z"
                ></path>
            </mask>
            <g mask="url(#i)">
                <path
                    fill="url(#j)"
                    d="m89.5 174.5-1.4.2-7.5-61.9 1.4-.1 7.5 61.8Zm-5.3-62h-1.4l7.4 62 1.4-.2-7.4-61.9Zm2.6-.4-1.4.2 7.4 61.8 1.4-.2-7.4-61.8Zm2.9-.3-1.4.1 7.4 61.9 1.4-.2-7.4-61.8Zm3.1-.4-1.4.2 7.4 61.8 1.4-.2-7.4-61.8Zm3.6-.4-1.4.1 7.4 61.9 1.4-.2-7.4-61.8Zm4-.5-1.3.2 7.4 61.8 1.4-.2-7.4-61.8Zm4.6-.5-1.4.1L111 172l1.4-.2L105 110Zm4.9-.6-1.4.2 7.4 61.8 1.4-.2-7.4-61.8Zm5.7-.7-1.4.2 7.4 61.8 1.4-.1-7.4-61.9Zm6.4-.7-1.4.1L128 170l1.4-.2L122 108Zm7-.8-1.4.1 7.4 61.9 1.4-.2-7.4-61.8Zm55-6.5-1.5.2 7.4 61.8 1.4-.1-7.4-61.9Zm-2.2.3-1.4.1 7.4 61.9 1.4-.2-7.4-61.8Zm-2.6.3-1.4.1 7.4 61.9 1.4-.2-7.4-61.8Zm-3 .3-1.3.2 7.4 61.8 1.4-.1-7.4-61.9Zm-3 .4-1.4.1 7.4 61.9 1.4-.2-7.4-61.8Zm-3.6.4-1.4.2 7.4 61.8 1.4-.2-7.4-61.8Zm-4 .5-1.5.1 7.4 61.9 1.4-.2-7.4-61.8Zm-4.6.5-1.4.2 7.4 61.8 1.4-.1-7.4-61.9Zm-4.9.6-1.4.1 7.4 61.9 1.4-.2-7.4-61.8Zm-5.7.6-1.4.2 7.4 61.9 1.4-.2-7.4-61.9Zm-6.4.8-1.4.2 7.4 61.8 1.4-.2-7.4-61.8Zm-7 .8-1.4.2 7.4 61.8 1.4-.1-7.4-61.9Z"
                ></path>
                <path
                    fill="url(#k)"
                    d="m90 174.4-1.5.2-7.4-61.8 1.4-.2 7.4 61.8Zm-5.3-62-1.4.1 7.4 61.9 1.4-.2-7.4-61.9Zm2.5-.4-1.4.2 7.5 61.8 1.4-.1-7.5-61.9Zm3-.3-1.4.2 7.4 61.8 1.4-.2-7.4-61.8Zm3.1-.4-1.4.2 7.4 61.8 1.4-.1-7.4-61.9Zm3.6-.4-1.4.2 7.4 61.8 1.4-.2-7.4-61.8Zm4-.5-1.4.2 7.5 61.8 1.4-.1-7.5-61.9Zm4.6-.5-1.4.1 7.4 61.9 1.4-.2-7.4-61.8Zm4.8-.6-1.4.2 7.4 61.8 1.4-.1-7.4-61.9Zm5.7-.7-1.4.2 7.4 61.9 1.4-.2-7.4-61.9Zm6.5-.7-1.4.2 7.4 61.8 1.4-.2-7.4-61.8Zm7-.8-1.4.1 7.4 61.9 1.4-.2-7.4-61.8Zm54.9-6.5-1.4.2 7.4 61.8 1.4-.1-7.4-61.9Zm-2.1.3-1.4.1 7.4 61.9 1.4-.2-7.4-61.8Zm-2.6.3-1.4.2 7.4 61.8 1.4-.2-7.4-61.8Zm-3 .3-1.3.2 7.4 61.8 1.4-.1-7.4-61.9Zm-3 .4-1.5.2 7.4 61.8 1.4-.2-7.4-61.8Zm-3.7.4-1.4.2 7.5 61.8 1.4-.1-7.5-61.9Zm-4 .5-1.4.2 7.4 61.8 1.4-.2-7.4-61.8Zm-4.5.5-1.4.2 7.4 61.8 1.4-.1-7.4-61.9Zm-5 .6-1.3.2 7.4 61.8 1.4-.2-7.4-61.8Zm-5.6.7-1.4.1 7.4 61.9 1.4-.2-7.4-61.8Zm-6.4.7-1.4.2 7.4 61.8 1.4-.1-7.4-61.9Zm-7 .8-1.5.2 7.5 61.8 1.4-.1-7.5-61.9Z"
                ></path>
            </g>
            <path
                fill="url(#l)"
                d="M137.3 147.2c29-3.4 50-26.5 47-51.7-3.1-25.1-29-42.7-58-39.4-28.8 3.4-49.9 26.6-46.8 51.7 3 25.1 28.9 42.8 57.8 39.4Z"
            ></path>
            <path
                fill="url(#m)"
                d="M137 144.9c28-3.3 48.4-25.6 45.5-49.8-2.9-24.2-28-41.2-56-37.9-28 3.3-48.4 25.6-45.5 49.8 3 24.2 28 41.2 56 37.9Z"
            ></path>
            <mask
                id="n"
                width="103"
                height="90"
                x="80"
                y="56"
                maskUnits="userSpaceOnUse"
                style="mask-type:luminance"
            >
                <path
                    fill="#fff"
                    d="M137 144.9c28-3.3 48.4-25.6 45.5-49.8-2.9-24.2-28-41.2-56-37.9-28 3.3-48.4 25.6-45.5 49.8 3 24.2 28 41.2 56 37.9Z"
                ></path>
            </mask>
            <g mask="url(#n)">
                <path
                    fill="#fff"
                    d="M181.6 124.9 75.6 101l3.6-12 106 23.8-3.6 12ZM181.5 135 75.5 111l.4-4.6 105.9 23.8-.3 4.7ZM189.4 76 83.5 52l3.8-1.6 106 23.8-3.9 1.6Z"
                    opacity=".3"
                ></path>
            </g>
            <path
                fill="url(#o)"
                d="M184.2 95.5c-3-25.1-28.9-42.8-57.8-39.4-29 3.4-50 26.6-47 51.7l1.4 11c3 25.2 28.7 41 57.6 37.6 29-3.4 50.2-24.7 47.1-49.8L184.3 96v-.6Z"
            ></path>
            <path
                fill="#FFF399"
                d="M135.6 133.4c21.1-2.5 36.5-19 34.3-36.8-2-17.8-21-30.2-42-27.7-21 2.4-36.4 18.9-34.3 36.7 2.1 17.8 21 30.2 42 27.8Z"
            ></path>
            <path
                fill="url(#p)"
                d="M135.5 132.2c-21 2.5-39.9-9.7-42-27.2-2-17.5 13.4-33.7 34.4-36.2 21-2.4 39.9 9.8 42 27.2 2 17.5-13.3 33.7-34.4 36.2Z"
            ></path>
            <g fill="#F28B51" style="mix-blend-mode:color-burn">
                <path d="M131.6 80s-2 6.2-1.2 7.5c.7 1.2 6.3 4.8 7 7 .8 2.1-.8 5.3 5.2 12.9 2.3 2.8-1.4 13.1-4 14.8-2.8 1.9-14.8 2-19.3-2.2-3.4-3.3-3.3-10-2.7-12 .6-1.9 8.3-9.5 10.5-11.6 2.2-2.1-1.7-4.2-1.7-6 0-1.7 2.5-8.6 6.2-10.5ZM142.7 99.9c-1.2.5-2.8-2.3-2.5-3.7.3-1.4 3-2.9 3.4-6.6 0 0 1.6 9.2-1 10.3ZM136 88.2c-1.4-1.4-2.1-3.4-1.5-4.8.6-1.5 3.2-4.2 3.4-5 0 0-1.7 4.8-1.2 6.3s.4 4.6-.7 3.5Z"></path>
                <path d="M121.6 98.9s3.5-2.3 2.6-4.7c-.8-2.4-1-4.4-.6-6 0 0-4.4 8.3-2 10.7Z"></path>
            </g>
            <path
                fill="url(#q)"
                d="M128.2 71.5c20.3-2.4 38.7 10 41.8 26.5l-.1-2c-2.1-17.4-21-29.6-42-27.1-21 2.4-36.5 18.7-34.4 36.1 0 .7.2 1.3.3 2-.8-16.8 14.2-33.1 34.4-35.5Z"
            ></path>
            <defs>
                <linearGradient
                    id="b"
                    x1="1.8"
                    x2="1.8"
                    y1="141.9"
                    y2="37.8"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop stop-color="#fff" stop-opacity="0"></stop>
                    <stop offset="1" stop-color="#fff"></stop>
                </linearGradient>
                <linearGradient
                    id="c"
                    x1="47"
                    x2="47"
                    y1="173.5"
                    y2="110.3"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop stop-color="#fff" stop-opacity="0"></stop>
                    <stop offset="1" stop-color="#fff"></stop>
                </linearGradient>
                <linearGradient
                    id="d"
                    x1="263.2"
                    x2="263.2"
                    y1="79.9"
                    y2="16.6"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop stop-color="#fff" stop-opacity="0"></stop>
                    <stop offset="1" stop-color="#fff"></stop>
                </linearGradient>
                <linearGradient
                    id="e"
                    x1="87"
                    x2="87"
                    y1="52"
                    y2="1"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop stop-color="#fff" stop-opacity="0"></stop>
                    <stop offset="1" stop-color="#fff"></stop>
                </linearGradient>
                <linearGradient
                    id="f"
                    x1="236.2"
                    x2="236.2"
                    y1="175.6"
                    y2="112.3"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop stop-color="#fff" stop-opacity="0"></stop>
                    <stop offset="1" stop-color="#fff"></stop>
                </linearGradient>
                <linearGradient
                    id="g"
                    x1="210.5"
                    x2="210.5"
                    y1="115"
                    y2="38"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop stop-color="#fff" stop-opacity="0"></stop>
                    <stop offset="1" stop-color="#fff"></stop>
                </linearGradient>
                <linearGradient
                    id="h"
                    x1="80"
                    x2="184.8"
                    y1="112.4"
                    y2="99.8"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop stop-color="#E8A928"></stop>
                    <stop offset=".4" stop-color="#C8690F"></stop>
                    <stop offset=".6" stop-color="#C8690F"></stop>
                    <stop offset="1" stop-color="#F8C935"></stop>
                </linearGradient>
                <linearGradient
                    id="j"
                    x1="98.9"
                    x2="172.1"
                    y1="183.4"
                    y2="91.4"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop offset="0" stop-color="#E8B328"></stop>
                    <stop offset=".4" stop-color="#E1800F"></stop>
                    <stop offset=".6" stop-color="#E1800F"></stop>
                    <stop offset=".9" stop-color="#F8D335"></stop>
                </linearGradient>
                <linearGradient
                    id="k"
                    x1="99.4"
                    x2="172.6"
                    y1="183.3"
                    y2="91.3"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop stop-color="#D8891B"></stop>
                    <stop offset=".4" stop-color="#B75D0F"></stop>
                    <stop offset=".6" stop-color="#B75D0F"></stop>
                    <stop offset="1" stop-color="#E4AF2C"></stop>
                </linearGradient>
                <linearGradient
                    id="l"
                    x1="126.4"
                    x2="137.1"
                    y1="56.1"
                    y2="147.2"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop stop-color="#F8E765"></stop>
                    <stop offset="1" stop-color="#F2B52D" stop-opacity=".8"></stop>
                </linearGradient>
                <linearGradient
                    id="m"
                    x1="126.5"
                    x2="136.8"
                    y1="57.2"
                    y2="144.9"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop stop-color="#FFFF82"></stop>
                    <stop offset="1" stop-color="#FFD83B"></stop>
                </linearGradient>
                <linearGradient
                    id="o"
                    x1="119.8"
                    x2="143.8"
                    y1="160.5"
                    y2="57.4"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop offset=".2" stop-color="#FFF37D" stop-opacity="0"></stop>
                    <stop offset=".7" stop-color="#fff" stop-opacity=".7"></stop>
                    <stop offset="1" stop-color="#FFF37D" stop-opacity="0"></stop>
                </linearGradient>
                <linearGradient
                    id="p"
                    x1="127.9"
                    x2="135.3"
                    y1="68.8"
                    y2="132.2"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop stop-color="#F8E253"></stop>
                    <stop offset="1" stop-color="#F7C332"></stop>
                </linearGradient>
                <linearGradient
                    id="q"
                    x1="131.9"
                    x2="128"
                    y1="102.4"
                    y2="68.8"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop stop-color="#F8C935"></stop>
                    <stop offset="1" stop-color="#C8690F"></stop>
                </linearGradient>
                <clipPath id="a">
                    <path fill="#fff" d="M260 175v50H5v-50z"></path>
                </clipPath>
            </defs>
        </svg>
    }
}
