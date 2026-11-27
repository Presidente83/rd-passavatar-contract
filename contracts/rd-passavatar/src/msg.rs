use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {
    pub founders_contract: String,
    pub image_sha256: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    ClaimBaseForPass { pass_token_id: String },
}

#[cw_serde]
pub enum QueryMsg {
    Supply {},
    Config {},
    OwnerOf { token_id: String },
    NftInfo { token_id: String },
    AllNftInfo { token_id: String },
    ContractInfo {},
    NumTokens {},
    Tokens { owner: String },
    AllTokens {},
    ClaimStatus { pass_token_id: String },
    Token { token_id: String },
}
