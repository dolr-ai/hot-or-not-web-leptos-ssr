#[derive(Clone, Debug, PartialEq)]
pub enum AppType {
    YRAL,
}

impl AppType {
    pub fn select() -> Self {
        AppType::YRAL
    }
}
