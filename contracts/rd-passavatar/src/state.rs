use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary};
use cw_storage_plus::{Item, Map};

pub const IMAGE_SHA256: &str = "f219c7ca963c30f626bbbd180f1673dc126e6a0a65438c275fec891048434318";
pub const MAX_SUPPLY: u64 = 100;
pub const COLLECTION_NAME: &str = "RD Station — Avatar Club 100";
pub const CONFIG: Item<Config> = Item::new("config_v3");
pub const NEXT_ID: Item<u64> = Item::new("next_id");
pub const COUNT: Item<u64> = Item::new("count");
pub const TOKENS: Map<&str, Token> = Map::new("tokens_v3");
pub const WALLET: Map<&Addr, String> = Map::new("wallet_v3");
pub const PASS_CLAIMS: Map<&str, String> = Map::new("pass_claims_v2");

#[cw_serde]
pub struct Config {
    pub founders_contract: Addr,
    pub image_sha256: String,
    pub max_supply: u64,
    pub collection_name: String,
}

#[cw_serde]
pub struct Token {
    pub owner: Addr,
    pub created_at: u64,
}

// Kept only for backwards-compatible schema imports in downstream tooling.
// It is not used by any execute path in the Base-only contract.
#[allow(dead_code)]
pub type LegacyBinary = Binary;
