export function set_sentry_user(user_principal) {
    if (!window.Sentry) {
        return;
    }

    Sentry.onLoad(function () {
        Sentry.setUser(user_principal ? {
            id: user_principal
        } : null)
    })
}

export function set_sentry_user_canister(user_canister) {
    if (!window.Sentry) {
        return;
    }

    Sentry.onLoad(function () {
        Sentry.setTag("user_canister", user_canister ? user_canister : null)
    })
}