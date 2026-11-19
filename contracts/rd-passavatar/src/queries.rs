use crate::{msg::QueryMsg, state::*};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{entry_point, to_json_binary, Binary, Deps, Env, Order, StdError, StdResult};
use cw_storage_plus::Bound;

#[cw_serde]
pub struct Access {
    pub owner: String,
    pub approvals: Vec<String>,
}
#[cw_serde]
pub struct NftInfo {
    pub token_uri: Option<String>,
    pub extension: Extension,
}
#[cw_serde]
pub struct Extension {
    pub image_sha256: String,
    pub created_at: u64,
    pub evolutions: Vec<Evolution>,
}
#[cw_serde]
pub struct WalletResponse {
    pub token_id: Option<String>,
}
#[cw_serde]
pub struct TokensResponse {
    pub tokens: Vec<String>,
}
fn access(t: &Token) -> Access {
    Access {
        owner: t.owner.to_string(),
        approvals: vec![],
    }
}
fn nft(id: &str, t: &Token) -> NftInfo {
    NftInfo {
        token_uri: Some(format!(
            "https://rdstation.org/api/passavatar/metadata/{id}"
        )),
        extension: Extension {
            image_sha256: IMAGE_SHA256.into(),
            created_at: t.created_at,
            evolutions: t.evolutions.clone(),
        },
    }
}
#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Supply {} => {
            #[cw_serde]
            struct Supply {
                minted: u64,
                remaining: u64,
                max_supply: u64,
                next_token_id: Option<String>,
            }
            let minted = COUNT.load(deps.storage)?;
            to_json_binary(&Supply {
                minted,
                remaining: MAX_SUPPLY.saturating_sub(minted),
                max_supply: MAX_SUPPLY,
                next_token_id: if minted < MAX_SUPPLY {
                    Some(format!("AV-{:06}", NEXT_ID.load(deps.storage)?))
                } else {
                    None
                },
            })
        }
        QueryMsg::Config {} => to_json_binary(&CONFIG.load(deps.storage)?),
        QueryMsg::OwnerOf { token_id, .. } => {
            to_json_binary(&access(&TOKENS.load(deps.storage, &token_id)?))
        }
        QueryMsg::NftInfo { token_id } => {
            to_json_binary(&nft(&token_id, &TOKENS.load(deps.storage, &token_id)?))
        }
        QueryMsg::AllNftInfo { token_id, .. } => {
            #[cw_serde]
            struct R {
                access: Access,
                info: NftInfo,
            }
            let t = TOKENS.load(deps.storage, &token_id)?;
            to_json_binary(&R {
                access: access(&t),
                info: nft(&token_id, &t),
            })
        }
        QueryMsg::ContractInfo {} => {
            #[cw_serde]
            struct R {
                name: String,
                symbol: String,
            }
            to_json_binary(&R {
                name: COLLECTION_NAME.into(),
                symbol: "RDAV".into(),
            })
        }
        QueryMsg::NumTokens {} => {
            #[cw_serde]
            struct R {
                count: u64,
            }
            to_json_binary(&R {
                count: COUNT.load(deps.storage)?,
            })
        }
        QueryMsg::ClaimStatus { pass_token_id } => {
            to_json_binary(&PASS_CLAIMS.may_load(deps.storage, &pass_token_id)?)
        }
        QueryMsg::Wallet { address } => to_json_binary(&WalletResponse {
            token_id: WALLET.may_load(deps.storage, &deps.api.addr_validate(&address)?)?,
        }),
        QueryMsg::Tokens {
            owner,
            start_after,
            limit,
        } => {
            let id = WALLET.may_load(deps.storage, &deps.api.addr_validate(&owner)?)?;
            to_json_binary(&TokensResponse {
                tokens: id
                    .into_iter()
                    .filter(|id| limit != Some(0) && start_after.as_ref().is_none_or(|s| id > s))
                    .collect(),
            })
        }
        QueryMsg::AllTokens { start_after, limit } => {
            let ids = TOKENS
                .keys(
                    deps.storage,
                    start_after.as_deref().map(Bound::exclusive),
                    None,
                    Order::Ascending,
                )
                .take(limit.unwrap_or(30).min(100) as usize)
                .collect::<StdResult<Vec<_>>>()?;
            to_json_binary(&TokensResponse { tokens: ids })
        }
        QueryMsg::AllOperators { owner, .. } => {
            deps.api.addr_validate(&owner)?;
            #[cw_serde]
            struct R {
                operators: Vec<String>,
            }
            to_json_binary(&R { operators: vec![] })
        }
        QueryMsg::Approvals { token_id, .. } => {
            TOKENS.load(deps.storage, &token_id)?;
            #[cw_serde]
            struct R {
                approvals: Vec<String>,
            }
            to_json_binary(&R { approvals: vec![] })
        }
        QueryMsg::Approval { .. } => Err(StdError::not_found("approval")),
        QueryMsg::Token { token_id } => to_json_binary(&TOKENS.load(deps.storage, &token_id)?),
        QueryMsg::Collection { collection_id } => {
            #[cw_serde]
            struct R {
                collection: Collection,
                offer: Offer,
            }
            to_json_binary(&R {
                collection: COLLECTIONS.load(deps.storage, &collection_id)?,
                offer: OFFERS.load(deps.storage, &collection_id)?,
            })
        }
        QueryMsg::Listing { token_id } => {
            to_json_binary(&LISTINGS.may_load(deps.storage, &token_id)?)
        }
        QueryMsg::Listings { start_after, limit } => to_json_binary(
            &LISTINGS
                .range(
                    deps.storage,
                    start_after.as_deref().map(Bound::exclusive),
                    None,
                    Order::Ascending,
                )
                .take(limit.unwrap_or(30).min(100) as usize)
                .collect::<StdResult<Vec<_>>>()?,
        ),
    }
}
