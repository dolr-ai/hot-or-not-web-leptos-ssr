use super::app_type::AppType;

#[derive(Clone)]
pub struct AppState {
    pub app_type: AppType,
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub theme_color: &'static str,
    pub assets_dir: &'static str,
}

impl AppState {
    pub fn from_type(app_type: &AppType) -> Self {
        match app_type {
            AppType::HotOrNot => Self {
                app_type: AppType::HotOrNot,
                id: "hotornot",
                name: "Hot Or Not",
                description: "Vote on the hottest content and earn rewards",
                theme_color: "#FF4500",
                assets_dir: "hotornot",
            },
            AppType::YRAL => Self {
                app_type: AppType::YRAL,
                id: "yral",
                name: "YRAL",
                description: "The First App to Host Creative Short Video Challenges",
                theme_color: "#E20479",
                assets_dir: "yral",
            },
        }
    }

    pub fn asset_path(&self) -> String {
        format!("img/{}", self.assets_dir)
    }
}
