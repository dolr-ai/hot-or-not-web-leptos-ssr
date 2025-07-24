use candid::Principal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct VideoModel {
    pub id: String,
    pub name: String,
    pub description: String,
    pub duration: String,
    pub cost_sats: u64,
}

impl VideoModel {
    pub fn get_models() -> Vec<Self> {
        vec![
            VideoModel {
                id: "pollo_1_6".to_string(),
                name: "Pollo 1.6".to_string(),
                description: "Better, faster and cheaper".to_string(),
                duration: "60 Sec".to_string(),
                cost_sats: 5,
            },
            VideoModel {
                id: "cling_2_1_master".to_string(),
                name: "Cling 2.1 Master".to_string(),
                description: "Enhanced visual realism and motion".to_string(),
                duration: "60 Mins".to_string(),
                cost_sats: 25,
            },
            VideoModel {
                id: "cling_2_1".to_string(),
                name: "Cling 2.1".to_string(),
                description: "Enhanced visual realism and motion".to_string(),
                duration: "2 Mins".to_string(),
                cost_sats: 15,
            },
        ]
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct VideoGenerationParams {
    pub user_principal: Principal,
    pub prompt: String,
    pub model: VideoModel,
    pub image_data: Option<String>,
}

impl Default for VideoGenerationParams {
    fn default() -> Self {
        Self {
            user_principal: Principal::anonymous(),
            prompt: String::new(),
            model: VideoModel::default(),
            image_data: None,
        }
    }
}