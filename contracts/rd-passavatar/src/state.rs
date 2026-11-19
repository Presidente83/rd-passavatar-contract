use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, Uint128};
use cw_storage_plus::{Item, Map};

pub const FOUNDER: &str = "terra1nv0g2ap3fmll0m29f8fqr8q5pz4hejjrkjne0j";
pub const IMAGE_SHA256: &str = "f219c7ca963c30f626bbbd180f1673dc126e6a0a65438c275fec891048434318";
pub const MAX_SUPPLY: u64 = 100;
pub const COLLECTION_NAME: &str = "RD Station — Avatar Club 100";
pub const CONFIG: Item<Config> = Item::new("config_v2");
pub const NEXT_ID: Item<u64> = Item::new("next_id");
pub const COUNT: Item<u64> = Item::new("count");
pub const TOKENS: Map<&str, Token> = Map::new("tokens_v2");
pub const WALLET: Map<&Addr, String> = Map::new("wallet_v2");
pub const PASS_CLAIMS: Map<&str, String> = Map::new("pass_claims_v1");
pub const COLLECTIONS: Map<&str, Collection> = Map::new("collections");
pub const OFFERS: Map<&str, Offer> = Map::new("offers");
pub const LISTINGS: Map<&str, Listing> = Map::new("listings");
pub const REVISION: Item<u64> = Item::new("revision");

#[cw_serde]
pub struct Config {
    pub founder: Addr,
    pub founders_contract: Addr,
    pub image_sha256: String,
    pub quote_public_key: Binary,
    pub paused: bool,
    pub secondary_enabled: bool,
    pub mint_price: Uint128,
    pub max_supply: u64,
    pub collection_name: String,
}
#[cw_serde]
pub struct Evolution {
    pub collection_id: String,
    pub applied_at: u64,
    pub asset_uri: String,
    pub asset_sha256: String,
    pub background: bool,
}
#[cw_serde]
pub struct Token {
    pub owner: Addr,
    pub created_at: u64,
    pub evolutions: Vec<Evolution>,
}
#[cw_serde]
pub struct Collection {
    pub id: String,
    pub asset_uri: String,
    pub asset_sha256: String,
    pub prerequisites: Vec<String>,
    pub incompatible: Vec<String>,
    pub founder_only: bool,
    pub background: bool,
}
#[cw_serde]
pub struct Offer {
    pub price: Uint128,
    pub starts_at: u64,
    pub ends_at: Option<u64>,
    pub remaining: Option<u64>,
    pub revision: u64,
}
#[cw_serde]
pub struct Listing {
    pub seller: Addr,
    pub price: Uint128,
    pub revision: u64,
    pub listed_at: u64,
}
// Signed by a dedicated quote service, never by a seed stored in the browser.
// This is an authenticated fee estimate, NOT an on-chain measurement of gas.
#[cw_serde]
pub struct FeeQuote {
    pub chain_id: String,
    pub contract: String,
    pub buyer: String,
    pub action: String,
    pub token_id: String,
    pub revision: u64,
    pub gross: Uint128,
    pub network_fee: Uint128,
    pub expires_at: u64,
}
