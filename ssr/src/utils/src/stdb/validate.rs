use candid::Principal;
use spacetimedb::{Identity, ReducerContext};

pub mod consts;
mod error;

pub use error::*;

pub fn identity_from_principal(principal: Principal) -> Identity {
    Identity::from_claims(consts::AUTH_JWT_ISS, &principal.to_string())
}

pub fn validate_sender_identity(ctx: &ReducerContext, principal: Principal) -> Result<()> {
    let valid = ctx.sender == identity_from_principal(principal);
    if !valid {
        return Err(Error::InvalidIdentity);
    }

    Ok(())
}
