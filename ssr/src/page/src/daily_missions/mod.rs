use ::leptos::logging::log;
use leptos::prelude::*;
use leptos_icons::*;
use leptos_meta::*;

use component::icons::information_icon::Information;
use component::{back_btn::BackButton, buttons::HighlightedButton, title::TitleText};
use state::app_state::AppState;

pub mod mission_state;
pub mod modals;

use mission_state::{provide_mission_state, use_mission_state, MissionActions, MissionProgress};
use modals::{get_modal_config, render_modal_with_state_close};

// MissionProgress is now defined in state module

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
                    on_action();
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
    }
}

#[component]
fn DailyMissionsContent() -> impl IntoView {
    let mission_state = use_mission_state();
    let mission_state_for_signal = mission_state.clone();
    let mission_state_for_modal = mission_state.clone();
    let actions = MissionActions::new(mission_state.clone());

    // Get all missions from state
    let missions_signal = Signal::derive(move || mission_state_for_signal.get_all_missions());
    let active_modal = mission_state.active_modal;

    // Create actions
    let login_action = actions.login_action();
    let play_game_action = actions.play_game_action();
    let generate_video_action = actions.generate_video_action();
    let refer_friend_action = actions.refer_friend_action();
    let claim_reward_action = actions.claim_reward_action();
    let close_modal_action = actions.close_modal_action();

    view! {
        <div class="flex flex-col gap-6 px-4 pb-20">
            {move || {
                let missions = missions_signal.get();
                missions.into_iter().map(|mission| {
                    let mission_id = mission.config.id.clone();
                    let segments = mission.config.segments.unwrap_or(0);
                    let is_claimable = mission.is_claimable();

                    // Determine action based on mission type
                    let mission_type = mission.config.mission_type.clone();
                    let login_action = login_action.clone();
                    let play_game_action = play_game_action.clone();
                    let generate_video_action = generate_video_action.clone();
                    let refer_friend_action = refer_friend_action.clone();
                    let claim_reward_action = claim_reward_action.clone();
                    let mission_id_for_claim = mission_id.clone();

                    let on_action = move || {
                        match mission_type.as_str() {
                            "login_streak" => {
                                if is_claimable {
                                    let _ = claim_reward_action.dispatch(mission_id_for_claim.clone());
                                } else {
                                    let _ = login_action.dispatch(());
                                }
                            },
                            "play_games" => {
                                if is_claimable {
                                    let _ = claim_reward_action.dispatch(mission_id_for_claim.clone());
                                } else {
                                    let _ = play_game_action.dispatch(());
                                }
                            },
                            "ai_video" => {
                                if is_claimable {
                                    let _ = claim_reward_action.dispatch(mission_id_for_claim.clone());
                                } else {
                                    let _ = generate_video_action.dispatch(());
                                }
                            },
                            "referral" => {
                                if is_claimable {
                                    let _ = claim_reward_action.dispatch(mission_id_for_claim.clone());
                                } else {
                                    let _ = refer_friend_action.dispatch(());
                                }
                            },
                            _ => {
                                log!("Unknown mission type: {}", mission_type);
                            }
                        }
                    };

                    view! {
                        <MissionCard
                            title=mission.config.title.clone()
                            progress=mission.progress
                            button_text=mission.config.button_text.clone()
                            info_text=mission.config.info_text.clone()
                            segments=segments
                            is_claimable=is_claimable
                            reward_amount=mission.config.reward_amount
                            reward_token=mission.config.reward_token.clone()
                            mission_type=mission.config.mission_type.clone()
                            on_action=on_action
                        />
                    }
                }).collect_view()
            }}

            // Modal System
            {move || {
                active_modal.with(|modal| {
                    if let Some((mission_id, modal_type)) = modal {
                        if let Some(mission) = mission_state_for_modal.get_mission(mission_id) {
                            if let Some(config) = get_modal_config(
                                modal_type,
                                &mission.progress,
                                mission.config.reward_amount,
                                &mission.config.reward_token
                            ) {
                                let close_action = close_modal_action.clone();
                                return render_modal_with_state_close(
                                    config,
                                    RwSignal::new(true),
                                    move || {
                                        let _ = close_action.dispatch(());
                                    }
                                ).into_any();
                            }
                        }
                    }
                    view! { <div></div> }.into_any()
                })
            }}
        </div>
    }
}

#[component]
pub fn DailyMissions() -> impl IntoView {
    let app_state = use_context::<AppState>();
    let page_title = app_state.unwrap().name.to_owned() + " - Daily Missions";

    // Provide mission state context
    let _mission_state = provide_mission_state();

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
