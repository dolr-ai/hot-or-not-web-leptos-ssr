//! Daily missions state management system
//!
//! This module provides comprehensive state management for daily missions including:
//! - Mission progress tracking with reactive signals
//! - User statistics and achievements
//! - Action creators for mission interactions
//! - Modal state management
//! - Persistent data structures

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct MissionProgress {
    pub current: u32,
    pub total: u32,
    pub completed: bool,
}

impl MissionProgress {
    pub const fn new(current: u32, total: u32) -> Self {
        Self {
            current,
            total,
            completed: current >= total,
        }
    }

    pub fn progress_percentage(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            ((self.current * 100) as f32 / self.total as f32).min(100.0)
        }
    }

    pub fn increment(&mut self, amount: u32) -> bool {
        let was_completed = self.completed;
        self.current = (self.current + amount).min(self.total);
        self.completed = self.current >= self.total;
        !was_completed && self.completed
    }

    pub fn reset(&mut self) {
        self.current = 0;
        self.completed = false;
    }

    pub const fn is_claimable(&self) -> bool {
        self.completed
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MissionConfig {
    pub id: String,
    pub title: String,
    pub description: String,
    pub button_text: String,
    pub info_text: String,
    pub reward_amount: u32,
    pub reward_token: String,
    pub mission_type: String,
    pub segments: Option<u32>,
    pub auto_reset_daily: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mission {
    pub config: MissionConfig,
    pub progress: MissionProgress,
    pub last_updated: Option<String>, // ISO timestamp
    pub streak_count: u32,            // For streak-based missions
    pub claimed: bool,
}

impl Mission {
    pub const fn new(config: MissionConfig, progress: MissionProgress) -> Self {
        Self {
            config,
            progress,
            last_updated: None,
            streak_count: 0,
            claimed: false,
        }
    }

    pub const fn is_claimable(&self) -> bool {
        self.progress.completed && !self.claimed
    }

    pub fn claim_reward(&mut self) -> bool {
        if self.is_claimable() {
            self.claimed = true;
            if self.config.auto_reset_daily {
                self.reset_progress();
            }
            return true;
        }
        false
    }

    pub fn reset_progress(&mut self) {
        self.progress.reset();
        self.claimed = false;
        // Note: Keep streak count for streak-based missions
    }

    pub fn increment_progress(&mut self, amount: u32) -> bool {
        let just_completed = self.progress.increment(amount);
        self.last_updated = Some(chrono::Utc::now().to_rfc3339());
        just_completed
    }
}

#[derive(Clone, Debug)]
pub struct MissionState {
    pub missions: RwSignal<HashMap<String, Mission>>,
    pub active_modal: RwSignal<Option<(String, String)>>, // (mission_id, modal_type)
    pub user_stats: RwSignal<UserStats>,
}

impl Default for MissionState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UserStats {
    pub total_yral_earned: u32,
    pub current_login_streak: u32,
    pub longest_login_streak: u32,
    pub games_played_today: u32,
    pub videos_generated_today: u32,
    pub friends_referred: u32,
    pub last_login_date: Option<String>,
}

impl MissionState {
    #[must_use]
    pub fn new() -> Self {
        let initial_missions = Self::create_default_missions();
        let sample_stats = UserStats {
            total_yral_earned: 50,
            current_login_streak: 5,
            longest_login_streak: 8,
            games_played_today: 10,
            videos_generated_today: 1,
            friends_referred: 2,
            last_login_date: Some(chrono::Utc::now().to_rfc3339()),
        };

        Self {
            missions: RwSignal::new(initial_missions),
            active_modal: RwSignal::new(None),
            user_stats: RwSignal::new(sample_stats),
        }
    }

    fn create_default_missions() -> HashMap<String, Mission> {
        let mut missions = HashMap::new();

        // Daily Login Streak Mission
        let login_config = MissionConfig {
            id: "login_streak".to_string(),
            title: "Daily Login Streak".to_string(),
            description: "Login daily to maintain your streak".to_string(),
            button_text: "Claim 5 YRAL".to_string(),
            info_text: "Hit 7 days for a bonus. Miss a day, streak resets.".to_string(),
            reward_amount: 5,
            reward_token: "YRAL".to_string(),
            mission_type: "login_streak".to_string(),
            segments: Some(7),
            auto_reset_daily: false,
        };
        missions.insert(
            "login_streak".to_string(),
            Mission::new(login_config, MissionProgress::new(5, 7)),
        );

        // Play Games Mission
        let games_config = MissionConfig {
            id: "play_games".to_string(),
            title: "Play 10 Games".to_string(),
            description: "Play games to earn tokens".to_string(),
            button_text: "Play Games".to_string(),
            info_text: "Play 10 games in 24 hours to earn 10 YRAL tokens.".to_string(),
            reward_amount: 10,
            reward_token: "YRAL".to_string(),
            mission_type: "play_games".to_string(),
            segments: None,
            auto_reset_daily: true,
        };
        missions.insert(
            "play_games".to_string(),
            Mission::new(games_config, MissionProgress::new(10, 10)),
        );

        // Generate AI Videos Mission
        let videos_config = MissionConfig {
            id: "generate_videos".to_string(),
            title: "Generate 3 AI videos".to_string(),
            description: "Create AI videos to earn tokens".to_string(),
            button_text: "Create AI Video".to_string(),
            info_text: "Generate 3 AI videos to earn 30 YRAL tokens.".to_string(),
            reward_amount: 30,
            reward_token: "YRAL".to_string(),
            mission_type: "ai_video".to_string(),
            segments: None,
            auto_reset_daily: true,
        };
        missions.insert(
            "generate_videos".to_string(),
            Mission::new(videos_config, MissionProgress::new(1, 3)),
        );

        // Referral Mission
        let referral_config = MissionConfig {
            id: "referral".to_string(),
            title: "Referral".to_string(),
            description: "Refer friends to earn tokens".to_string(),
            button_text: "Refer A Friend".to_string(),
            info_text: "Refer 3 friends to earn 15 YRAL tokens.".to_string(),
            reward_amount: 15,
            reward_token: "YRAL".to_string(),
            mission_type: "referral".to_string(),
            segments: None,
            auto_reset_daily: false,
        };
        missions.insert(
            "referral".to_string(),
            Mission::new(referral_config, MissionProgress::new(2, 3)),
        );

        missions
    }

    // Actions for updating mission progress
    pub fn increment_login_streak(&self) -> bool {
        let mut result = false;
        self.missions.update(|missions| {
            if let Some(mission) = missions.get_mut("login_streak") {
                let just_completed = mission.increment_progress(1);
                mission.streak_count += 1;

                // Update user stats
                self.user_stats.update(|stats| {
                    stats.current_login_streak = mission.streak_count;
                    stats.longest_login_streak =
                        stats.longest_login_streak.max(mission.streak_count);
                    stats.last_login_date = Some(chrono::Utc::now().to_rfc3339());
                });

                // Trigger appropriate modal
                if just_completed {
                    self.active_modal.set(Some((
                        "login_streak".to_string(),
                        "streak_complete".to_string(),
                    )));
                } else if mission.progress.current >= 5 {
                    self.active_modal.set(Some((
                        "login_streak".to_string(),
                        "almost_there".to_string(),
                    )));
                } else if mission.progress.current >= 2 {
                    self.active_modal
                        .set(Some(("login_streak".to_string(), "streak".to_string())));
                } else {
                    self.active_modal.set(Some((
                        "login_streak".to_string(),
                        "daily_reward".to_string(),
                    )));
                }

                result = just_completed;
            }
        });
        result
    }

    pub fn increment_games_played(&self) -> bool {
        let mut result = false;
        self.missions.update(|missions| {
            if let Some(mission) = missions.get_mut("play_games") {
                let just_completed = mission.increment_progress(1);

                // Update user stats
                self.user_stats.update(|stats| {
                    stats.games_played_today += 1;
                });

                // Trigger appropriate modal
                if just_completed {
                    self.active_modal.set(Some((
                        "play_games".to_string(),
                        "target_complete".to_string(),
                    )));
                } else if mission.progress.current >= mission.progress.total / 2 {
                    self.active_modal
                        .set(Some(("play_games".to_string(), "halfway".to_string())));
                }

                result = just_completed;
            }
        });
        result
    }

    pub fn increment_videos_generated(&self) -> bool {
        let mut result = false;
        self.missions.update(|missions| {
            if let Some(mission) = missions.get_mut("generate_videos") {
                let just_completed = mission.increment_progress(1);

                // Update user stats
                self.user_stats.update(|stats| {
                    stats.videos_generated_today += 1;
                });

                // Trigger appropriate modal
                if just_completed {
                    self.active_modal.set(Some((
                        "generate_videos".to_string(),
                        "completion".to_string(),
                    )));
                } else if mission.progress.current > 0 {
                    self.active_modal.set(Some((
                        "generate_videos".to_string(),
                        "progress".to_string(),
                    )));
                }

                result = just_completed;
            }
        });
        result
    }

    pub fn increment_referrals(&self) -> bool {
        let mut result = false;
        self.missions.update(|missions| {
            if let Some(mission) = missions.get_mut("referral") {
                let just_completed = mission.increment_progress(1);

                // Update user stats
                self.user_stats.update(|stats| {
                    stats.friends_referred += 1;
                });

                // Trigger appropriate modal
                if just_completed {
                    self.active_modal.set(Some((
                        "referral".to_string(),
                        "referral_complete".to_string(),
                    )));
                } else if mission.progress.current > 0 {
                    self.active_modal
                        .set(Some(("referral".to_string(), "refer_earn".to_string())));
                }

                result = just_completed;
            }
        });
        result
    }

    pub fn claim_mission_reward(&self, mission_id: &str) -> bool {
        let mut result = false;
        self.missions.update(|missions| {
            if let Some(mission) = missions.get_mut(mission_id) {
                if mission.claim_reward() {
                    // Update user stats
                    self.user_stats.update(|stats| {
                        stats.total_yral_earned += mission.config.reward_amount;
                    });

                    // Close any active modal
                    self.active_modal.set(None);
                    result = true;
                }
            }
        });
        result
    }

    pub fn reset_daily_missions(&self) {
        self.missions.update(|missions| {
            for mission in missions.values_mut() {
                if mission.config.auto_reset_daily {
                    mission.reset_progress();
                }
            }
        });

        // Reset daily stats
        self.user_stats.update(|stats| {
            stats.games_played_today = 0;
            stats.videos_generated_today = 0;
        });
    }

    pub fn close_modal(&self) {
        self.active_modal.set(None);
    }

    #[must_use]
    pub fn get_mission(&self, mission_id: &str) -> Option<Mission> {
        self.missions
            .with(|missions| missions.get(mission_id).cloned())
    }

    #[must_use]
    pub fn get_all_missions(&self) -> Vec<Mission> {
        self.missions
            .with(|missions| missions.values().cloned().collect())
    }

    // Computed signals
    #[must_use]
    pub fn total_earned_signal(&self) -> impl Fn() -> u32 + Clone {
        let stats = self.user_stats;
        move || stats.with(|stats| stats.total_yral_earned)
    }

    #[must_use]
    pub fn current_streak_signal(&self) -> impl Fn() -> u32 + Clone {
        let stats = self.user_stats;
        move || stats.with(|stats| stats.current_login_streak)
    }

    #[must_use]
    pub fn mission_signal(&self, mission_id: String) -> impl Fn() -> Option<Mission> + Clone {
        let missions = self.missions;
        move || missions.with(|missions| missions.get(&mission_id).cloned())
    }
}

// Global context provider
#[must_use]
pub fn provide_mission_state() -> MissionState {
    let state = MissionState::new();
    provide_context(state.clone());
    state
}

#[must_use]
pub fn use_mission_state() -> MissionState {
    use_context::<MissionState>().expect("MissionState context not found")
}

// Action creators for use in components
#[derive(Clone)]
pub struct MissionActions {
    state: MissionState,
}

impl MissionActions {
    pub const fn new(state: MissionState) -> Self {
        Self { state }
    }

    #[must_use]
    pub fn login_action(&self) -> Action<(), bool> {
        let state = self.state.clone();
        Action::new(move |()| {
            let state = state.clone();
            async move { state.increment_login_streak() }
        })
    }

    #[must_use]
    pub fn play_game_action(&self) -> Action<(), bool> {
        let state = self.state.clone();
        Action::new(move |()| {
            let state = state.clone();
            async move { state.increment_games_played() }
        })
    }

    #[must_use]
    pub fn generate_video_action(&self) -> Action<(), bool> {
        let state = self.state.clone();
        Action::new(move |()| {
            let state = state.clone();
            async move { state.increment_videos_generated() }
        })
    }

    #[must_use]
    pub fn refer_friend_action(&self) -> Action<(), bool> {
        let state = self.state.clone();
        Action::new(move |()| {
            let state = state.clone();
            async move { state.increment_referrals() }
        })
    }

    #[must_use]
    pub fn claim_reward_action(&self) -> Action<String, bool> {
        let state = self.state.clone();
        Action::new(move |mission_id: &String| {
            let state = state.clone();
            let mission_id = mission_id.clone();
            async move { state.claim_mission_reward(&mission_id) }
        })
    }

    #[must_use]
    pub fn close_modal_action(&self) -> Action<(), ()> {
        let state = self.state.clone();
        Action::new(move |()| {
            let state = state.clone();
            async move { state.close_modal() }
        })
    }
}
