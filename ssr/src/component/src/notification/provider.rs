use state::canisters::AuthState;
use thiserror::Error;
use yral_canisters_client::{
    ic::NOTIFICATION_STORE_ID,
    notification_store::{NotificationData, NotificationStore},
};
use yral_canisters_common::{
    cursored_data::{CursoredDataProvider, KeyedData, PageEntry},
    Canisters,
};

#[derive(Clone)]
pub struct NotificationDataKeyed(pub (NotificationData, bool));

impl KeyedData for NotificationDataKeyed {
    type Key = String;
    fn key(&self) -> String {
        self.0 .0.notification_id.to_string()
    }
}

#[derive(Debug, Clone, Error)]
pub struct NotificationError(pub String);

impl std::fmt::Display for NotificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone)]
pub struct NotificationProvider {
    pub auth: AuthState,
    pub canisters: Canisters<false>,
    pub last_viewed_time: u64,
}

impl CursoredDataProvider for NotificationProvider {
    type Data = NotificationDataKeyed;
    type Error = NotificationError;

    async fn get_by_cursor_inner(
        &self,
        _start: usize,
        _end: usize,
    ) -> Result<PageEntry<Self::Data>, Self::Error> {
        let cans = self
            .auth
            .auth_cans()
            .await
            .map_err(|e| NotificationError(e.to_string()))?;

        let agent = cans.authenticated_user().await.1;

        let client = NotificationStore(NOTIFICATION_STORE_ID, agent);

        let notifications = client
            .get_notifications((_end - _start + 1) as u64, _start as u64)
            .await
            .map_err(|e| NotificationError(e.to_string()))?;

        let list_end = notifications.len() < (_end - _start);

        if list_end {
            client
                .set_notification_panel_viewed()
                .await
                .map_err(|e| NotificationError(e.to_string()))
                .inspect_err(|e| {
                    leptos::logging::log!("Failed to set notification panel viewed: {}", e)
                })?;
        }

        Ok(PageEntry {
            data: notifications
                .into_iter()
                .map(|n| {
                    let is_read = self.last_viewed_time < n.created_at.secs_since_epoch;
                    leptos::logging::log!(
                        "Notification: {} is_read: {}",
                        n.notification_id,
                        is_read
                    );
                    NotificationDataKeyed((n, is_read))
                })
                .collect(),
            end: list_end,
        })
    }
}
