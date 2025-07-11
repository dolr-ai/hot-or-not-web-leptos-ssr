use leptos::prelude::*;
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct VideoComparisonResult {
    pub hot_or_not: bool,
    pub current_video_score: f32,
    pub previous_video_score: f32,
}

#[derive(Default, Clone)]
pub struct HnBetState {
    state: RwSignal<BTreeMap<String, VideoComparisonResult>>,
}

impl HnBetState {
    pub fn init() -> Self {
        let this = Self {
            state: RwSignal::new(BTreeMap::new()),
        };
        provide_context(this.clone());
        this
    }

    pub fn get(video_uid: String) -> Option<VideoComparisonResult> {
        let this = use_context::<Self>().unwrap_or_else(HnBetState::init);
        this.state.get().get(&video_uid).cloned()
    }

    pub fn set(video_uid: String, result: VideoComparisonResult) {
        let this = use_context::<Self>().unwrap_or_else(HnBetState::init);
        this.state.update(|state| {
            state.insert(video_uid, result);
        });
    }
}

impl VideoComparisonResult {
    pub fn parse_video_comparison_result(value_str: &str) -> Result<VideoComparisonResult, String> {
        let trimmed = value_str.trim_matches(|c| c == '(' || c == ')');
        let parts: Vec<&str> = trimmed.split(',').collect();

        if parts.len() != 3 {
            return Err(format!(
                "Expected 3 fields in result, got {}: {:?}",
                parts.len(),
                parts
            ));
        }

        let hot_or_not = match parts[0] {
            "t" => true,
            "f" => false,
            other => return Err(format!("Unexpected boolean value: {other}")),
        };

        let current_video_score: f32 = parts[1]
            .parse()
            .map_err(|e| format!("Failed to parse current_video_score: {e}"))?;

        let previous_video_score: f32 = parts[2]
            .parse()
            .map_err(|e| format!("Failed to parse previous_video_score: {e}"))?;

        Ok(VideoComparisonResult {
            hot_or_not,
            current_video_score,
            previous_video_score,
        })
    }
}
