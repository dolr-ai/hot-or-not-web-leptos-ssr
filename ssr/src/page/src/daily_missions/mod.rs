use ::leptos::logging::log;
use leptos::prelude::*;
use leptos_icons::*;
use leptos_meta::*;

use component::icons::information_icon::Information;
use component::{back_btn::BackButton, buttons::HighlightedButton, title::TitleText};
use state::app_state::AppState;

pub mod modals;
use modals::{icons, ButtonConfig, ButtonStyle, UniversalModal};

#[derive(Clone, Copy)]
pub struct MissionProgress {
    pub current: u32,
    pub total: u32,
    pub completed: bool,
}

impl MissionProgress {
    pub fn progress_percentage(&self) -> u32 {
        if self.total == 0 {
            0
        } else {
            ((self.current as f32 / self.total as f32) * 100.0) as u32
        }
    }
}

#[component]
fn ProgressBar(
    progress: MissionProgress,
    #[prop(optional, default = 0)] segments: u32,
) -> impl IntoView {
    let progress_percentage = progress.progress_percentage();

    if segments > 0 {
        // Segmented progress bar (for login streak)
        let segment_items: Vec<_> = (0..segments as i32).collect();
        view! {
            <div class="flex gap-1 w-full">
                {segment_items.into_iter().map(|i| {
                    let is_active = (i as u32) < progress.current;
                    view! {
                        <div class=format!(
                            "flex-1 h-2 rounded-full {}",
                            if is_active {
                                "bg-gradient-to-r from-green-400 to-green-500"
                            } else {
                                "bg-neutral-700"
                            }
                        )></div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        }
        .into_any()
    } else {
        // Continuous progress bar
        view! {
            <div class="w-full bg-neutral-700 rounded-full h-2">
                <div
                    class="bg-gradient-to-r from-green-400 to-green-500 h-2 rounded-full transition-all duration-300"
                    style=format!("width: {}%", progress_percentage)
                ></div>
            </div>
        }.into_any()
    }
}

#[component]
fn MissionCard(
    title: String,
    progress: MissionProgress,
    button_text: String,
    info_text: String,
    #[prop(optional, default = 0)] segments: u32,
    #[prop(optional, default = false)] is_claimable: bool,
    #[prop(optional, default = 30)] reward_amount: u32,
    #[prop(optional, default = "YRAL".to_string())] reward_token: String,
    #[prop(optional, default = "ai_video".to_string())] mission_type: String,
    on_action: impl Fn() + 'static,
) -> impl IntoView {
    let progress_text = if segments > 0 {
        format!("Day {}/{}", progress.current, segments)
    } else {
        format!("{}/{}", progress.current, progress.total)
    };

    // Clone title for display
    let title_for_display = title.clone();

    // Simple modal state
    let show_modal = RwSignal::new(false);
    let current_modal = RwSignal::new("".to_string());

    view! {
        <div class="flex flex-col gap-4 p-4 bg-neutral-900 rounded-lg">
            <div class="flex justify-between items-start">
                <h3 class="text-white font-semibold text-lg">{title_for_display}</h3>
                <span class="text-neutral-400 text-sm">{progress_text}</span>
            </div>

            <ProgressBar progress=progress segments=segments />

            <HighlightedButton
                classes=if is_claimable {
                    "".to_string()
                } else {
                    "opacity-75".to_string()
                }
                alt_style=false
                disabled=!is_claimable && !progress.completed
                on_click=move || {
                    // Determine which modal to show based on mission state
                    let modal_type = if progress.completed {
                        match mission_type.as_str() {
                            "play_games" => "target_complete",
                            "referral" => "referral_complete",
                            "login_streak" => "streak_complete",
                            _ => "completion"
                        }
                    } else if progress.current > 0 && progress.current < progress.total {
                        match mission_type.as_str() {
                            "ai_video" => "progress",
                            "play_games" if progress.current >= progress.total / 2 => "halfway",
                            "referral" => "refer_earn",
                            "login_streak" if progress.current >= 5 => "almost_there",
                            "login_streak" if progress.current >= 2 => "streak",
                            _ => {
                                on_action();
                                return;
                            }
                        }
                    } else if mission_type == "login_streak" {
                        "daily_reward"
                    } else {
                        on_action();
                        return;
                    };

                    current_modal.set(modal_type.to_string());
                    show_modal.set(true);
                }
            >
                {if is_claimable {
                    view! {
                        <div class="flex items-center gap-2">
                            <span>{button_text.clone()}</span>
                            <Icon icon=Information attr:class="w-4 h-4" />
                        </div>
                    }.into_any()
                } else {
                    view! { <span>{button_text}</span> }.into_any()
                }}
            </HighlightedButton>

            <div class="flex items-center gap-2 text-neutral-400 text-sm">
                <Icon icon=Information attr:class="w-4 h-4" />
                <span>{info_text}</span>
            </div>
        </div>

        // Universal Modal System
        {move || {
            if !show_modal.get() {
                return view! { <div></div> }.into_any();
            }

            let modal_type = current_modal.get();
            match modal_type.as_str() {
                "progress" => {
                    let remaining = progress.total - progress.current;
                    view! {
                        <UniversalModal
                            show=show_modal
                            title=format!("{} Down, {} to Go!", progress.current, remaining)
                            description=format!("Awesome start—you've generated your first AI video. Create {} more to get <span class='font-semibold text-white'>{} {}</span>.<br />Keep the creativity flowing!", remaining, reward_amount, reward_token)
                            svg_content=icons::video_cards_icon
                            buttons=vec![ButtonConfig {
                                text: "View Missions".to_string(),
                                style: ButtonStyle::Secondary,
                                on_click: Box::new(|| {
                                    leptos::logging::log!("View missions clicked");
                                }),
                            }]
                            on_close=move || show_modal.set(false)
                        />
                    }.into_any()
                },
                "completion" => {
                    view! {
                        <UniversalModal
                            show=show_modal
                            title="Mission Complete!".to_string()
                            description="You crushed your mission!<br />Your reward is ready. Tap below to grab it!".to_string()
                            svg_content=icons::video_cards_complete_icon
                            buttons=vec![ButtonConfig {
                                text: format!("Claim {} {}", reward_amount, reward_token),
                                style: ButtonStyle::Primary,
                                on_click: Box::new(|| {
                                    leptos::logging::log!("Claim reward clicked");
                                }),
                            }]
                            on_close=move || show_modal.set(false)
                        />
                    }.into_any()
                },
                "halfway" => {
                    let remaining = progress.total - progress.current;
                    view! {
                        <UniversalModal
                            show=show_modal
                            title="You are halfway there!".to_string()
                            description=format!("You've played {} games today—just {} more to go to win <span class='font-semibold text-white'>{} {}</span>.<br />Complete your mission within the next 24 hours.", progress.current, remaining, reward_amount, reward_token)
                            svg_content=icons::medal_icon
                            buttons=vec![ButtonConfig {
                                text: "View Missions".to_string(),
                                style: ButtonStyle::Secondary,
                                on_click: Box::new(|| {
                                    leptos::logging::log!("View missions clicked");
                                }),
                            }]
                            on_close=move || show_modal.set(false)
                        />
                    }.into_any()
                },
                "target_complete" => {
                    view! {
                        <UniversalModal
                            show=show_modal
                            title="Mission Complete!".to_string()
                            description="You crushed 10 games in a day!<br />Your reward is ready. Tap below to grab it!".to_string()
                            svg_content=icons::target_complete_icon
                            buttons=vec![ButtonConfig {
                                text: format!("Claim {} {}", reward_amount, reward_token),
                                style: ButtonStyle::Primary,
                                on_click: Box::new(|| {
                                    leptos::logging::log!("Claim target reward clicked");
                                }),
                            }]
                            on_close=move || show_modal.set(false)
                        />
                    }.into_any()
                },
                "almost_there" => {
                    let remaining = progress.total - progress.current;
                    view! {
                        <UniversalModal
                            show=show_modal
                            title="You are almost there!".to_string()
                            description=format!("Keep up your daily streak. Only {} more logins to hit the {}-day milestone and win <span class='font-semibold text-white'>{} {}!</span><br /><span class='text-sm'>Miss a day and your streak resets.</span>", remaining, progress.total, reward_amount, reward_token)
                            progress_bar={(progress.current, progress.total)}
                            svg_content=icons::target_with_card_icon
                            buttons=vec![
                                ButtonConfig {
                                    text: format!("Claim 5 {}", reward_token),
                                    style: ButtonStyle::Primary,
                                    on_click: Box::new(|| {
                                        leptos::logging::log!("Claim daily reward");
                                    }),
                                },
                                ButtonConfig {
                                    text: "View Missions".to_string(),
                                    style: ButtonStyle::Secondary,
                                    on_click: Box::new(|| {
                                        leptos::logging::log!("View missions clicked");
                                    }),
                                }
                            ]
                            on_close=move || show_modal.set(false)
                        />
                    }.into_any()
                },
                "streak" => {
                    view! {
                        <UniversalModal
                            show=show_modal
                            title="You're on a streak!".to_string()
                            description=format!("Earn 5 {} for logging in today.<br />Keep the streak going to win additional 30 {} tokens on Day {}!<br /><span class='text-sm'>Miss a day and your streak resets.</span>", reward_token, reward_token, progress.total)
                            progress_bar={(progress.current, progress.total)}
                            svg_content=icons::flame_icon
                            buttons=vec![
                                ButtonConfig {
                                    text: format!("Claim 5 {}", reward_token),
                                    style: ButtonStyle::Primary,
                                    on_click: Box::new(|| {
                                        leptos::logging::log!("Claim streak reward");
                                    }),
                                },
                                ButtonConfig {
                                    text: "View Missions".to_string(),
                                    style: ButtonStyle::Secondary,
                                    on_click: Box::new(|| {
                                        leptos::logging::log!("View missions clicked");
                                    }),
                                }
                            ]
                            on_close=move || show_modal.set(false)
                        />
                    }.into_any()
                },
                "daily_reward" => {
                    view! {
                        <UniversalModal
                            show=show_modal
                            title="Your Daily Reward Awaits!".to_string()
                            description=format!("Claim your 5 {} tokens for today and keep your streak alive!<br /><span class='text-sm'>Miss a day and your streak resets.</span>", reward_token)
                            svg_content=icons::lightning_coin_icon
                            buttons=vec![
                                ButtonConfig {
                                    text: format!("Claim 5 {}", reward_token),
                                    style: ButtonStyle::Primary,
                                    on_click: Box::new(|| {
                                        leptos::logging::log!("Claim daily reward");
                                    }),
                                },
                                ButtonConfig {
                                    text: "View Missions".to_string(),
                                    style: ButtonStyle::Secondary,
                                    on_click: Box::new(|| {
                                        leptos::logging::log!("View missions clicked");
                                    }),
                                }
                            ]
                            on_close=move || show_modal.set(false)
                        />
                    }.into_any()
                },
                "streak_complete" => {
                    view! {
                        <UniversalModal
                            show=show_modal
                            title="Streak Complete!".to_string()
                            description=format!("You've reached the 7th day!<br />Claim now to complete your streak and get your {} {} tokens.", reward_amount, reward_token)
                            svg_content=icons::lightning_coin_flame_icon
                            buttons=vec![ButtonConfig {
                                text: format!("Claim {} {}", reward_amount, reward_token),
                                style: ButtonStyle::Primary,
                                on_click: Box::new(|| {
                                    leptos::logging::log!("Claim streak completion reward");
                                }),
                            }]
                            on_close=move || show_modal.set(false)
                        />
                    }.into_any()
                },
                "refer_earn" => {
                    let remaining = progress.total - progress.current;
                    view! {
                        <UniversalModal
                            show=show_modal
                            title="Refer & Earn!".to_string()
                            description=format!("Awesome start—you've started your referral journey. Invite {} more friends and win <span class='font-semibold text-white'>{} {} tokens</span>.", remaining, reward_amount, reward_token)
                            progress_bar={(progress.current, progress.total)}
                            svg_content=icons::megaphone_icon
                            buttons=vec![ButtonConfig {
                                text: "View Missions".to_string(),
                                style: ButtonStyle::Secondary,
                                on_click: Box::new(|| {
                                    leptos::logging::log!("View missions clicked");
                                }),
                            }]
                            on_close=move || show_modal.set(false)
                        />
                    }.into_any()
                },
                "referral_complete" => {
                    view! {
                        <UniversalModal
                            show=show_modal
                            title="Mission Complete!".to_string()
                            description=format!("You've referred 3 friends — now it's time to claim your <span class='font-semibold text-white'>{} {}</span>.", reward_amount, reward_token)
                            svg_content=icons::megaphone_complete_icon
                            buttons=vec![ButtonConfig {
                                text: format!("Claim {} {}", reward_amount, reward_token),
                                style: ButtonStyle::Primary,
                                on_click: Box::new(|| {
                                    leptos::logging::log!("Claim referral reward");
                                }),
                            }]
                            on_close=move || show_modal.set(false)
                        />
                    }.into_any()
                },
                _ => view! { <div></div> }.into_any()
            }
        }}
    }
}

#[component]
fn DailyMissionsContent() -> impl IntoView {
    // Mock data - replace with actual data from your state management
    let login_streak = MissionProgress {
        current: 5,
        total: 7,
        completed: false,
    };

    let play_games = MissionProgress {
        current: 10,
        total: 10,
        completed: true,
    };

    let generate_videos = MissionProgress {
        current: 1,
        total: 3,
        completed: false,
    };

    let referral = MissionProgress {
        current: 2,
        total: 3,
        completed: false,
    };

    view! {
        <div class="flex flex-col gap-6 px-4 pb-20">
            <MissionCard
                title="Daily Login Streak".to_string()
                progress=login_streak
                button_text="Claim 5 YRAL".to_string()
                info_text="Hit 7 days for a bonus. Miss a day, streak resets.".to_string()
                segments=7
                is_claimable=true
                reward_amount=5
                reward_token="YRAL".to_string()
                mission_type="login_streak".to_string()
                on_action=|| {
                    // Handle claim login streak
                    log!("Claim login streak");
                }
            />

            <MissionCard
                title="Play 10 Games".to_string()
                progress=play_games
                button_text="Play Games".to_string()
                info_text="Play 10 games in 24 hours to earn 10 YRAL tokens.".to_string()
                reward_amount=10
                reward_token="YRAL".to_string()
                mission_type="play_games".to_string()
                on_action=|| {
                    // Handle play games
                    log!("Play games");
                }
            />

            <MissionCard
                title="Generate 3 AI videos".to_string()
                progress=generate_videos
                button_text="Create AI Video".to_string()
                info_text="Generate 3 AI videos to earn 30 YRAL tokens.".to_string()
                reward_amount=30
                reward_token="YRAL".to_string()
                mission_type="ai_video".to_string()
                on_action=|| {
                    // Handle create AI video
                    log!("Create AI video");
                }
            />

            <MissionCard
                title="Referral".to_string()
                progress=referral
                button_text="Refer A Friend".to_string()
                info_text="Refer 3 friends to earn 15 YRAL tokens.".to_string()
                reward_amount=15
                reward_token="YRAL".to_string()
                mission_type="referral".to_string()
                on_action=|| {
                    // Handle referral
                    log!("Refer a friend");
                }
            />
        </div>
    }
}

#[component]
pub fn DailyMissions() -> impl IntoView {
    let app_state = use_context::<AppState>();
    let page_title = app_state.unwrap().name.to_owned() + " - Daily Missions";

    view! {
        <Title text=page_title.clone() />
        <div class="flex flex-col min-h-screen bg-black text-white">
            <div class="flex-none pt-2 pb-4 px-2">
                <TitleText justify_center=false>
                    <div class="flex flex-row justify-between bg-transparent">
                        <BackButton fallback="/menu".to_string() />
                        <span class="text-lg font-bold text-white">Daily Missions</span>
                        <div></div>
                    </div>
                </TitleText>
            </div>

            <div class="flex-1">
                <DailyMissionsContent />
            </div>
        </div>
    }
}
