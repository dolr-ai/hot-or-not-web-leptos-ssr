//! Mission modal configurations for daily missions
//!
//! This module provides pre-configured modal setups for different mission states:
//! - Progress tracking modals
//! - Completion celebration modals
//! - Streak milestone modals
//! - Referral progress modals
//!
//! Each modal configuration includes title, description, icon, buttons, and optional progress bars.

use super::super::mission_state::MissionProgress;
use super::universal_modal::{icons, ButtonConfig, ButtonStyle, UniversalModal};
use leptos::prelude::*;

/// Configuration for a mission modal
pub struct ModalConfig {
    pub title: String,
    pub description: String,
    pub svg_content: fn() -> AnyView,
    pub buttons: Vec<ButtonConfig>,
    pub progress_bar: Option<(u32, u32)>,
}

/// Creates modal configuration for AI video progress
#[must_use]
pub fn create_progress_modal(
    progress: &MissionProgress,
    reward_amount: u32,
    reward_token: &str,
) -> ModalConfig {
    let remaining = progress.total - progress.current;

    ModalConfig {
        title: format!("{} Down, {} to Go!", progress.current, remaining),
        description: format!(
            "Awesome start—you've generated your first AI video. Create {} more to get <span class='font-semibold text-white'>{} {}</span>.<br />Keep the creativity flowing!",
            remaining, reward_amount, reward_token
        ),
        svg_content: icons::video_cards_icon,
        buttons: vec![ButtonConfig {
            text: "View Missions".to_string(),
            style: ButtonStyle::Secondary,
            on_click: Box::new(|| {
                leptos::logging::log!("View missions clicked");
            }),
        }],
        progress_bar: None,
    }
}

/// Creates modal configuration for mission completion
#[must_use]
pub fn create_completion_modal(reward_amount: u32, reward_token: &str) -> ModalConfig {
    ModalConfig {
        title: "Mission Completed!".to_string(),
        description: "You crushed your mission!<br />Your reward is ready. Tap below to grab it!"
            .to_string(),
        svg_content: icons::video_cards_complete_icon,
        buttons: vec![ButtonConfig {
            text: format!("Claim {} {}", reward_amount, reward_token),
            style: ButtonStyle::Primary,
            on_click: Box::new(|| {
                leptos::logging::log!("Claim reward clicked");
            }),
        }],
        progress_bar: None,
    }
}

/// Creates modal configuration for halfway progress in games
#[must_use]
pub fn create_halfway_modal(
    progress: &MissionProgress,
    reward_amount: u32,
    reward_token: &str,
) -> ModalConfig {
    let remaining = progress.total - progress.current;

    ModalConfig {
        title: "You are halfway there!".to_string(),
        description: format!(
            "You've played {} games today—just {} more to go to win <span class='font-semibold text-white'>{} {}</span>.<br />Complete your mission within the next 24 hours.",
            progress.current, remaining, reward_amount, reward_token
        ),
        svg_content: icons::medal_icon,
        buttons: vec![ButtonConfig {
            text: "View Missions".to_string(),
            style: ButtonStyle::Secondary,
            on_click: Box::new(|| {
                leptos::logging::log!("View missions clicked");
            }),
        }],
        progress_bar: None,
    }
}

/// Creates modal configuration for target completion
#[must_use]
pub fn create_target_complete_modal(reward_amount: u32, reward_token: &str) -> ModalConfig {
    ModalConfig {
        title: "Mission Complete!".to_string(),
        description:
            "You crushed 10 games in a day!<br />Your reward is ready. Tap below to grab it!"
                .to_string(),
        svg_content: icons::target_complete_icon,
        buttons: vec![ButtonConfig {
            text: format!("Claim {} {}", reward_amount, reward_token),
            style: ButtonStyle::Primary,
            on_click: Box::new(|| {
                leptos::logging::log!("Claim target reward clicked");
            }),
        }],
        progress_bar: None,
    }
}

/// Creates modal configuration for almost complete streak
#[must_use]
pub fn create_almost_there_modal(
    progress: &MissionProgress,
    reward_amount: u32,
    reward_token: &str,
) -> ModalConfig {
    let remaining = progress.total - progress.current;

    ModalConfig {
        title: "You are almost there!".to_string(),
        description: format!(
            "Keep up your daily streak. Only {} more logins to hit the {}-day milestone and win <span class='font-semibold text-white'>{} {}!</span><br /><span class='text-sm'>Miss a day and your streak resets.</span>",
            remaining, progress.total, reward_amount, reward_token
        ),
        svg_content: icons::target_with_card_icon,
        buttons: vec![
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
        ],
        progress_bar: Some((progress.current, progress.total)),
    }
}

/// Creates modal configuration for active streak
#[must_use]
pub fn create_streak_modal(progress: &MissionProgress, reward_token: &str) -> ModalConfig {
    ModalConfig {
        title: "You're on a streak!".to_string(),
        description: format!(
            "Earn 5 {} for logging in today.<br />Keep the streak going to win additional 30 {} tokens on Day {}!<br /><span class='text-sm'>Miss a day and your streak resets.</span>",
            reward_token, reward_token, progress.total
        ),
        svg_content: icons::flame_icon,
        buttons: vec![
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
        ],
        progress_bar: Some((progress.current, progress.total)),
    }
}

/// Creates modal configuration for daily reward
#[must_use]
pub fn create_daily_reward_modal(reward_token: &str) -> ModalConfig {
    ModalConfig {
        title: "Your Daily Reward Awaits!".to_string(),
        description: format!(
            "Claim your 5 {} tokens for today and keep your streak alive!<br /><span class='text-sm'>Miss a day and your streak resets.</span>",
            reward_token
        ),
        svg_content: icons::lightning_coin_icon,
        buttons: vec![
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
        ],
        progress_bar: None,
    }
}

/// Creates modal configuration for streak completion
#[must_use]
pub fn create_streak_complete_modal(reward_amount: u32, reward_token: &str) -> ModalConfig {
    ModalConfig {
        title: "Streak Complete!".to_string(),
        description: format!(
            "You've reached the 7th day!<br />Claim now to complete your streak and get your {} {} tokens.",
            reward_amount, reward_token
        ),
        svg_content: icons::lightning_coin_flame_icon,
        buttons: vec![ButtonConfig {
            text: format!("Claim {} {}", reward_amount, reward_token),
            style: ButtonStyle::Primary,
            on_click: Box::new(|| {
                leptos::logging::log!("Claim streak completion reward");
            }),
        }],
        progress_bar: None,
    }
}

/// Creates modal configuration for referral progress
#[must_use]
pub fn create_refer_earn_modal(
    progress: &MissionProgress,
    reward_amount: u32,
    reward_token: &str,
) -> ModalConfig {
    let remaining = progress.total - progress.current;

    ModalConfig {
        title: "Refer & Earn!".to_string(),
        description: format!(
            "Awesome start—you've started your referral journey. Invite {} more friends and win <span class='font-semibold text-white'>{} {} tokens</span>.",
            remaining, reward_amount, reward_token
        ),
        svg_content: icons::megaphone_icon,
        buttons: vec![ButtonConfig {
            text: "View Missions".to_string(),
            style: ButtonStyle::Secondary,
            on_click: Box::new(|| {
                leptos::logging::log!("View missions clicked");
            }),
        }],
        progress_bar: Some((progress.current, progress.total)),
    }
}

/// Creates modal configuration for referral completion
#[must_use]
pub fn create_referral_complete_modal(reward_amount: u32, reward_token: &str) -> ModalConfig {
    ModalConfig {
        title: "Mission Complete!".to_string(),
        description: format!(
            "You've referred 3 friends — now it's time to claim your <span class='font-semibold text-white'>{} {}</span>.",
            reward_amount, reward_token
        ),
        svg_content: icons::megaphone_complete_icon,
        buttons: vec![ButtonConfig {
            text: format!("Claim {} {}", reward_amount, reward_token),
            style: ButtonStyle::Primary,
            on_click: Box::new(|| {
                leptos::logging::log!("Claim referral reward");
            }),
        }],
        progress_bar: None,
    }
}

/// Main function to get modal configuration based on modal type
#[must_use]
pub fn get_modal_config(
    modal_type: &str,
    progress: &MissionProgress,
    reward_amount: u32,
    reward_token: &str,
) -> Option<ModalConfig> {
    match modal_type {
        "progress" => Some(create_progress_modal(progress, reward_amount, reward_token)),
        "completion" => Some(create_completion_modal(reward_amount, reward_token)),
        "halfway" => Some(create_halfway_modal(progress, reward_amount, reward_token)),
        "target_complete" => Some(create_target_complete_modal(reward_amount, reward_token)),
        "almost_there" => Some(create_almost_there_modal(
            progress,
            reward_amount,
            reward_token,
        )),
        "streak" => Some(create_streak_modal(progress, reward_token)),
        "daily_reward" => Some(create_daily_reward_modal(reward_token)),
        "streak_complete" => Some(create_streak_complete_modal(reward_amount, reward_token)),
        "refer_earn" => Some(create_refer_earn_modal(
            progress,
            reward_amount,
            reward_token,
        )),
        "referral_complete" => Some(create_referral_complete_modal(reward_amount, reward_token)),
        _ => None,
    }
}

/// Creates and renders a modal based on configuration
#[must_use]
pub fn render_modal_with_state_close(
    config: ModalConfig,
    show: RwSignal<bool>,
    on_close: impl Fn() + 'static + Copy + Send + Sync,
) -> impl IntoView {
    if let Some(progress_bar) = config.progress_bar {
        view! {
            <UniversalModal
                show=show
                title=config.title
                description=config.description
                svg_content=config.svg_content
                buttons=config.buttons
                progress_bar=progress_bar
                on_close=on_close
            />
        }
        .into_any()
    } else {
        view! {
            <UniversalModal
                show=show
                title=config.title
                description=config.description
                svg_content=config.svg_content
                buttons=config.buttons
                on_close=on_close
            />
        }
        .into_any()
    }
}

/// Creates and renders a modal based on configuration (legacy version)
#[must_use]
pub fn render_modal(config: ModalConfig, show: RwSignal<bool>) -> impl IntoView {
    render_modal_with_state_close(config, show, move || show.set(false))
}
