use crate::state::{Collection, FeeQuote, Offer};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub founders_contract: String,
    pub image_sha256: String,
    pub quote_public_key: Binary,
    pub mint_price: Uint128,
}
#[cw_serde]
pub struct Selection {
    pub collection_id: String,
    pub revision: u64,
}
#[cw_serde]
pub enum ExecuteMsg {
    ClaimBaseForPass {
        pass_token_id: String,
        expected_owner: String,
    },
    SetSecondaryEnabled {
        enabled: bool,
    },
    BuyBase {
        quote: FeeQuote,
        signature: Binary,
    },
    PublishCollection {
        collection: Collection,
        offer: Offer,
    },
    BuyEvolutions {
        token_id: String,
        selections: Vec<Selection>,
        expected_total: Uint128,
        quote: FeeQuote,
        signature: Binary,
    },
    List {
        token_id: String,
        price: Uint128,
    },
    CancelListing {
        token_id: String,
    },
    BuyListing {
        token_id: String,
        quote: FeeQuote,
        signature: Binary,
    },
    SetPaused {
        paused: bool,
    },
    TransferNft {
        recipient: String,
        token_id: String,
    },
    SendNft {
        contract: String,
        token_id: String,
        msg: Binary,
    },
    Approve {
        spender: String,
        token_id: String,
    },
    ApproveAll {
        operator: String,
    },
    Revoke {
        spender: String,
        token_id: String,
    },
    RevokeAll {
        operator: String,
    },
    Burn {
        token_id: String,
    },
}
#[cw_serde]
pub enum QueryMsg {
    Supply {},
    Config {},
    OwnerOf {
        token_id: String,
        include_expired: Option<bool>,
    },
    NftInfo {
        token_id: String,
    },
    AllNftInfo {
        token_id: String,
        include_expired: Option<bool>,
    },
    ContractInfo {},
    NumTokens {},
    Tokens {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    AllTokens {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    AllOperators {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
        include_expired: Option<bool>,
    },
    Approvals {
        token_id: String,
        include_expired: Option<bool>,
    },
    Approval {
        token_id: String,
        spender: String,
        include_expired: Option<bool>,
    },
    ClaimStatus {
        pass_token_id: String,
    },
    Wallet {
        address: String,
    },
    Token {
        token_id: String,
    },
    Collection {
        collection_id: String,
    },
    Listing {
        token_id: String,
    },
    Listings {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}
