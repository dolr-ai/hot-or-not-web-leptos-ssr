use crate::wallet::airdrop::dolr_airdrop::send_airdrop_to_user;
use candid::{Nat, Principal};
use leptos::prelude::*;
use yral_canisters_client::sns_ledger::TransferResult;

#[server(input = server_fn::codec::Json)]
async fn send_money_impl() -> Result<String, ServerFnError> {
    let tushar_principal =
        Principal::from_text("v6uzq-up7cy-os5rl-oxyp6-vodok-prm66-lbrwu-r6qes-otn5a-kwfeb-hae")
            .expect("valid");
    // .123dolr
    let amount = Nat::from(123_000_usize);

    let (res, admin_principal) = send_airdrop_to_user(tushar_principal, amount).await?;

    if admin_principal.to_text()
        != "zg7n3-345by-nqf6o-3moz4-iwxql-l6gko-jqdz2-56juu-ja332-unymr-fqe"
    {
        return Err(ServerFnError::new(format!(
            "unknown admin: {admin_principal}"
        )));
    }

    match res {
        TransferResult::Ok(nat) => Ok(format!("{nat}")),
        TransferResult::Err(err) => Err(ServerFnError::new(format!("transfer error: {err:?}"))),
    }
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
