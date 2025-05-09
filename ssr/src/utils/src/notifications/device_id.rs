use leptos::prelude::*;
use leptos::server;
use std::env;
#[cfg(feature = "ga4")]
#[server]
pub async fn send_principal_and_token_offchain(
    device_id: String,
    principal_id: String,
) -> Result<(), ServerFnError> {
    use crate::off_chain;
    use tonic::metadata::MetadataValue;
    use tonic::transport::Channel;
    use tonic::Request;

    let channel: Channel = expect_context();

    let mut off_chain_agent_grpc_auth_token = env::var("GRPC_AUTH_TOKEN").expect("GRPC_AUTH_TOKEN");
    // removing whitespaces and new lines for proper parsing
    off_chain_agent_grpc_auth_token.retain(|c| !c.is_whitespace());

    let token: MetadataValue<_> = format!("Bearer {off_chain_agent_grpc_auth_token}").parse()?;

    let mut client = off_chain::off_chain_client::OffChainClient::with_interceptor(
        channel,
        move |mut req: Request<()>| {
            req.metadata_mut().insert("authorization", token.clone());
            Ok(req)
        },
    );

    let request = tonic::Request::new(off_chain::BindDeviceToPrincipalRequest {
        device_id,
        principal_id,
    });

    client.bind_device_to_principal(request).await?;

    Ok(())
}
