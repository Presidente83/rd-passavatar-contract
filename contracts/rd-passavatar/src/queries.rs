use crate::{msg::QueryMsg, state::*};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{entry_point, to_json_binary, Binary, Deps, Env, Order, StdResult};

#[cw_serde]
pub struct Access { pub owner: String, pub approvals: Vec<String> }

#[cw_serde]
pub struct NftInfo { pub token_uri: Option<String>, pub extension: Extension }

#[cw_serde]
pub struct Extension { pub image_sha256: String, pub created_at: u64 }

#[cw_serde]
pub struct TokensResponse { pub tokens: Vec<String> }

fn access(t: &Token) -> Access { Access { owner: t.owner.to_string(), approvals: vec![] } }

fn nft(token_id: &str, t: &Token) -> NftInfo {
    NftInfo {
        token_uri: Some(format!("https://rdstation.org/api/passavatar/metadata/{token_id}")),
        extension: Extension { image_sha256: IMAGE_SHA256.into(), created_at: t.created_at },
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Supply {} => {
            #[cw_serde]
            struct Supply { minted: u64, remaining: u64, max_supply: u64, next_token_id: Option<String> }
            let minted = COUNT.load(deps.storage)?;
            let next = if minted < MAX_SUPPLY { Some(format!("AV-{:06}", NEXT_ID.load(deps.storage)?)) } else { None };
            to_json_binary(&Supply { minted, remaining: MAX_SUPPLY - minted, max_supply: MAX_SUPPLY, next_token_id: next })
        }
        QueryMsg::Config {} => to_json_binary(&CONFIG.load(deps.storage)?),
        QueryMsg::OwnerOf { token_id } => to_json_binary(&access(&TOKENS.load(deps.storage, &token_id)?)),
        QueryMsg::NftInfo { token_id } => to_json_binary(&nft(&token_id, &TOKENS.load(deps.storage, &token_id)?)),
        QueryMsg::AllNftInfo { token_id } => {
            #[cw_serde] struct R { access: Access, info: NftInfo }
            let t = TOKENS.load(deps.storage, &token_id)?;
            to_json_binary(&R { access: access(&t), info: nft(&token_id, &t) })
        }
        QueryMsg::ContractInfo {} => {
            #[cw_serde] struct R { name: String, symbol: String }
            to_json_binary(&R { name: COLLECTION_NAME.into(), symbol: "RDAV".into() })
        }
        QueryMsg::NumTokens {} => {
            #[cw_serde] struct R { count: u64 }
            to_json_binary(&R { count: COUNT.load(deps.storage)? })
        }
        QueryMsg::Tokens { owner } => {
            let owner = deps.api.addr_validate(&owner)?;
            to_json_binary(&TokensResponse { tokens: WALLET.may_load(deps.storage, &owner)?.into_iter().collect() })
        }
        QueryMsg::AllTokens {} => {
            let tokens = TOKENS.keys(deps.storage, None, None, Order::Ascending).collect::<StdResult<Vec<_>>>()?;
            to_json_binary(&TokensResponse { tokens })
        }
        QueryMsg::ClaimStatus { pass_token_id } => to_json_binary(&PASS_CLAIMS.may_load(deps.storage, &pass_token_id)?),
        QueryMsg::Token { token_id } => to_json_binary(&TOKENS.load(deps.storage, &token_id)?),
    }
}
