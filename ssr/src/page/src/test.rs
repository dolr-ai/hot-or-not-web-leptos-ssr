use leptos::prelude::*;

#[component]
pub fn Test() -> impl IntoView {
    use candid::Principal;
    use codee::string::FromToStringCodec;
    use consts::USER_PRINCIPAL_STORE;
    use leptos_use::use_cookie;

    #[cfg(feature = "stdb-backend")]
    let ctx: state::stdb_dolr_airdrop::WrappedContext = expect_context();

    #[cfg(feature = "stdb-backend")]
    let notifications = RwSignal::new(vec![]);

    let (user_principal_cookie, _) =
        use_cookie::<Principal, FromToStringCodec>(USER_PRINCIPAL_STORE);

    Effect::new(move |_| {
        #[cfg(feature = "stdb-backend")]
        {
            use state::stdb_dolr_airdrop::get_notitfication;

            notifications.set(
                get_notitfication(user_principal_cookie.get().unwrap(), &ctx.conn)
                    .unwrap_or_default(),
            )
        }
    });
    view! {
        <div>
            <h1>Test</h1>
            <div>
                <h2>Notifications</h2>
                <ul>

                    {
                        #[cfg(feature = "stdb-backend")]
                        move || {
                            let res = format!("{:?}", notifications.get());
                            view! {
                                <li>{res}</li>
                            }
                        }
                    }
                </ul>
            </div>
        </div>
    }
}
