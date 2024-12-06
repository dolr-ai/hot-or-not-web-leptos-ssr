use leptos::*;

use crate::component::buttons::Button;

#[component]
pub fn AirdropPage() -> impl IntoView {
    let (bg_img_loaded, set_bg_img_loaded) = create_signal(false);
    let (claimed, set_claimed) = create_signal(false);
    let (coin_image_loaded, set_coin_image_loaded) = create_signal(false);

    let coin_image = "https://picsum.photos/200";
    let bg_image = "https://picsum.photos/1000";
    let cloud_image = "https://picsum.photos/200";
    let parachute_image = "https://picsum.photos/200";

    let airdrop_amount = 100;
    let airdrop_from_token = "Rock_Salt";

    let handle_claim = move || {
        if !claimed.get() {
            set_claimed(true);
        } else {
            // goto wallet page
        }
    };

    view! {
        <div
            style="background: radial-gradient(circle, rgba(0,0,0,0) 0%, rgba(0,0,0,0) 75%, rgba(50,0,28,0.5) 100%);"
            class="h-screen w-screen relative bg-black items-center justify-center gap-8 text-white font-kumbh flex flex-col overflow-hidden"
        >
            <img
                alt="bg"
                src=bg_image
                on:load=move |_| {
                    set_bg_img_loaded(true);
                }
                class=move || {
                    format!(
                        "absolute inset-0 z-[1] w-full h-full object-cover transition-all duration-1000 {}",
                        if bg_img_loaded.get() {
                            "opacity-100 scale-100"
                        } else {
                            "opacity-5 scale-110"
                        },
                    )
                }
            />

            {move || {
                if bg_img_loaded.get() && !claimed.get() {
                    view! {
                        <div class="relative h-[24rem] z-[2]">
                            <div
                                style="--y: 50px"
                                class="flex flex-col items-center justify-center airdrop-parachute"
                            >
                                <img alt="Parachute" src=parachute_image class="h-72 shrink-0" />

                                <div
                                    style="background: radial-gradient(circle, rgb(244 141 199) 0%, rgb(255 255 255) 100%); box-shadow: 0px 0px 3.43px 0px #FFFFFF29;"
                                    class="p-[1px] w-16 h-16 -translate-y-8 rounded-full"
                                >
                                    <img
                                        alt="Airdrop"
                                        src=coin_image
                                        on:load=move |_| set_coin_image_loaded(true)
                                        class=move || {
                                            format!(
                                                "w-full rounded-full h-full object-cover transition-opacity {}",
                                                if coin_image_loaded.get() {
                                                    "opacity-100"
                                                } else {
                                                    "opacity-0"
                                                },
                                            )
                                        }
                                    />
                                </div>
                            </div>
                            <img
                                alt="Cloud"
                                src=cloud_image
                                style="--x: -50px"
                                class="w-12 absolute -top-10 left-0 airdrop-cloud"
                            />
                            <img
                                alt="Cloud"
                                src=cloud_image
                                style="--x: 50px"
                                class="w-16 absolute bottom-10 right-10 airdrop-cloud"
                            />
                        </div>
                    }
                } else if claimed.get() {
                    view! {
                        <div class="h-[24rem] w-full flex items-center justify-center z-[2]">
                            <div class="h-[12rem] w-[12rem] relative">
                                <AnimatedTick />
                                <div
                                    style="--duration:1500ms; background: radial-gradient(circle, rgba(27,0,15,1) 0%, rgba(0,0,0,1) 100%); box-shadow: 0px 0px 3.43px 0px #FFFFFF29;"
                                    class="p-[1px] fade-in absolute w-16 h-16 -bottom-4 -right-4 rounded-full"
                                >
                                    <img
                                        alt="Airdrop"
                                        src=coin_image
                                        on:load=move |_| set_coin_image_loaded(true)
                                        class=format!(
                                            "w-full rounded-full h-full object-cover transition-opacity {}",
                                            if coin_image_loaded.get() {
                                                "opacity-100"
                                            } else {
                                                "opacity-0"
                                            },
                                        )
                                    />
                                </div>
                            </div>
                        </div>
                    }
                } else {
                    view! { <div class="invisible" /> }
                }
            }}

            {move || {
                if bg_img_loaded.get() {
                    view! {
                        <div
                            style="--duration:1500ms"
                            class="fade-in flex text-xl font-bold z-[2] w-full flex-col gap-4 items-center justify-center px-8"
                        >
                            {if claimed.get() {
                                view! {
                                    <div class="text-center">
                                        {format!("{} {}", airdrop_amount, airdrop_from_token)} <br />
                                        <span class="font-normal">"added to wallet"</span>
                                    </div>
                                }
                            } else {
                                view! {
                                    <div class="text-center">
                                        {format!("{} {} Airdrop received received received", airdrop_amount, airdrop_from_token)}
                                    </div>
                                }
                            }}
                            <Button
                                classes="max-w-96 mx-auto".to_string()
                                alt_style=claimed.into()
                                on_click=handle_claim
                            >
                                {if claimed.get() { "Go to wallet" } else { "Claim Now" }}
                            </Button>

                        </div>
                    }
                } else {
                    view! { <div class="invisible" /> }
                }
            }}
        </div>
    }
}

#[component]
pub fn AnimatedTick() -> impl IntoView {
    view! {
        <div class="h-full w-full [perspective:800px]">
            <div class="relative h-full w-full scale-110 animate-coin-spin-horizontal rounded-full [transform-style:preserve-3d] before:absolute before:h-full before:w-full before:rounded-full
            before:bg-gradient-to-b before:from-[#FFC6F9] before:via-[#C01271] before:to-[#990D55] before:[transform-style:preserve-3d] before:[transform:translateZ(1px)]">
                <div class="absolute flex h-full w-full items-center justify-center rounded-full text-center [transform:translateZ(2rem)] p-12
                bg-gradient-to-br from-[#C01272] to-[#FF48B2]">
                    <div class="relative">
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            xmlns:xlink="http://www.w3.org/1999/xlink"
                            class="h-full w-full text-current [transform-style:preserve-3d] [transform:translateZ(10px)]"
                            viewBox="0 -3 32 32"
                            version="1.1"
                        >
                            <g stroke="none" stroke-width="1" fill="none" fill-rule="evenodd">
                                <g
                                    transform="translate(-518.000000, -1039.000000)"
                                    fill="currentColor"
                                >
                                    <path d="M548.783,1040.2 C547.188,1038.57 544.603,1038.57 543.008,1040.2 L528.569,1054.92 L524.96,1051.24 C523.365,1049.62 520.779,1049.62 519.185,1051.24 C517.59,1052.87 517.59,1055.51 519.185,1057.13 L525.682,1063.76 C527.277,1065.39 529.862,1065.39 531.457,1063.76 L548.783,1046.09 C550.378,1044.46 550.378,1041.82 548.783,1040.2"></path>
                                </g>
                            </g>
                        </svg>
                    </div>
                </div>
            </div>
        </div>
    }
}
