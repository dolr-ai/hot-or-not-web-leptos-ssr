use candid::{CandidType, Deserialize, Nat, Principal};

#[derive(CandidType, Deserialize, Debug)]
pub struct Transaction {
    pub to: Recipient,
    pub fee: Option<Nat>,
    pub memo: Option<Vec<u8>>,
    pub from_subaccount: Option<Vec<u8>>,
    pub created_at_time: Option<u64>,
    pub amount: Nat,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct Recipient {
    pub owner: Principal,
    pub subaccount: Option<Vec<u8>>,
}

#[derive(CandidType, Deserialize, Debug)]
pub enum TransferResult {
    Ok(Nat),
    Err(CustomTransferError),
}

#[derive(CandidType, Deserialize, Debug)]
pub enum CustomTransferError {
    GenericError { message: String, error_code: Nat },
    TemporarilyUnavailable,
    BadBurn { min_burn_amount: Nat },
    Duplicate { duplicate_of: Nat },
    BadFee { expected_fee: Nat },
    CreatedInFuture { ledger_time: u64 },
    TooOld,
    InsufficientFunds { balance: Nat },
}

#[derive(CandidType, Deserialize)]
pub struct Icrc1BalanceOfArg {
    pub owner: Principal,
    pub subaccount: Option<serde_bytes::ByteBuf>,
}
