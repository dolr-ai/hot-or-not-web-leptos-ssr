use crate::wallet::airdrop::dolr_airdrop::send_airdrop_to_user;
use candid::{Nat, Principal};
use leptos::prelude::*;

#[server(input = server_fn::codec::Json)]
async fn send_money_impl() -> Result<(), ServerFnError> {
    let tushar_principal =
        Principal::from_text("v6uzq-up7cy-os5rl-oxyp6-vodok-prm66-lbrwu-r6qes-otn5a-kwfeb-hae")
            .expect("valid");
    // .123dolr
    let amount = Nat::from(123_000_usize);

    send_airdrop_to_user(tushar_principal, amount).await?;

    Ok(())
}

#[component]
pub fn TestSend() -> impl IntoView {
    let send_money = Action::new_unsync(move |&()| async move { send_money_impl().await });

    let pending = send_money.pending();
    let value = send_money.value();
    view! {
        <button on:click=move |_| {
            send_money.dispatch(());
        }>send money</button>
        <pre>pending = {pending}</pre>
        <pre>value = {move || format!("{:#?}", value.get())}</pre>
    }
}
