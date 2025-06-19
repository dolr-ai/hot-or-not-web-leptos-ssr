use candid::Principal;
use spacetimedb::{Identity, ReducerContext};

pub mod consts;
mod error;

pub use error::*;

pub fn get_const_limits() -> Result<()> {
    // fetch all constant limits from db

    Ok(())

}
