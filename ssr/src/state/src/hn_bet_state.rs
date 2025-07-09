use std::collections::BTreeMap;

use candid::Principal;
use leptos::{logging, prelude::*};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct VideoComparisonResult {
    pub hot_or_not: bool,
    pub current_video_score: f32,
    pub previous_video_score: f32,
}

#[derive(Default, Clone)]
pub struct HnBetState {
    state: RwSignal<BTreeMap<(Principal, u64), VideoComparisonResult>>,
}

impl HnBetState {
    pub fn init() -> Self {
        // let this = Self {
        //     state: RwSignal::new(BTreeMap::new()),
        // };
        // provide_context(this.clone());
        // this
        Self {
            state: RwSignal::new(BTreeMap::new()),
        }
    }

    pub fn get(principal: Principal, post_id: u64) -> Option<VideoComparisonResult> {
        logging::log!(
            "HnBetState::get called for principal: {}, post_id: {}",
            principal,
            post_id
        );
        // let this = use_context::<Self>().unwrap_or_else(HnBetState::init);
        let this = expect_context::<Self>();
        this.state.get().get(&(principal, post_id)).cloned()
    }

    pub fn set(principal: Principal, post_id: u64, result: VideoComparisonResult) {
        logging::log!(
            "HnBetState::set called for principal: {}, post_id: {}, result: {:?}",
            principal,
            post_id,
            result
        );
        // let this = use_context::<Self>().unwrap_or_else(HnBetState::init);
        let this = expect_context::<Self>();
        this.state.update(|state| {
            state.insert((principal, post_id), result);
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
