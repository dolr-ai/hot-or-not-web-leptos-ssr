use ::leptos::logging::log;
use leptos::prelude::*;
use leptos_icons::*;
use leptos_meta::*;

use component::icons::information_icon::Information;
use component::{back_btn::BackButton, buttons::HighlightedButton, title::TitleText};
use state::app_state::AppState;

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
        let segment_items: Vec<_> = (0..segments).collect();
        view! {
            <div class="flex gap-1 w-full">
                {segment_items.into_iter().map(|i| {
                    let is_active = i < progress.current;
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
    on_action: impl Fn() + 'static,
) -> impl IntoView {
    let progress_text = if segments > 0 {
        format!("Day {}/{}", progress.current, segments)
    } else {
        format!("{}/{}", progress.current, progress.total)
    };

    view! {
        <div class="flex flex-col gap-4 p-4 bg-neutral-900 rounded-lg">
            <div class="flex justify-between items-start">
                <h3 class="text-white font-semibold text-lg">{title}</h3>
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
                on_click=move || on_action()
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
    }
}

#[component]
fn DailyMissionsContent() -> impl IntoView {
    // Mock data - replace with actual data from your state management
    let login_streak = MissionProgress {
        current: 1,
        total: 7,
        completed: false,
    };

    let play_games = MissionProgress {
        current: 6,
        total: 10,
        completed: false,
    };

    let generate_videos = MissionProgress {
        current: 1,
        total: 3,
        completed: false,
    };

    let referral = MissionProgress {
        current: 1,
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
