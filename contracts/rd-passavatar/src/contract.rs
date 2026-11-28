use crate::{
    msg::{ExecuteMsg, InstantiateMsg},
    state::{Config, Token, CONFIG, COUNT, IMAGE_SHA256, MAX_SUPPLY, NEXT_ID, PASS_CLAIMS, TOKENS, WALLET},
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{entry_point, Addr, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult};

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("invalid payment")]
    Payment,
    #[error("unauthorized")]
    Unauthorized,
    #[error("wallet already owns an avatar")]
    Occupied,
    #[error("already minted or claimed")]
    Duplicate,
    #[error("invalid configuration or input")]
    Invalid,
    #[error("supply exhausted")]
    Exhausted,
}

#[cw_serde]
enum FounderQuery {
    OwnerOf { token_id: String },
}

// Founders Core v1 returns additional metadata alongside the owner. Keep this
// response type deliberately permissive: the claim path needs only `owner`,
// and a future metadata field must not make every claim fail.
#[derive(serde::Deserialize)]
struct FounderOwner {
    owner: String,
}

fn no_funds(info: &MessageInfo) -> Result<(), Error> {
    if !info.funds.is_empty() { return Err(Error::Payment); }
    Ok(())
}

fn eligible_pass_id(id: &str) -> bool {
    id.len() == 3 && id.bytes().all(|b| b.is_ascii_digit())
        && id.parse::<u16>().is_ok_and(|n| (1..=100).contains(&n))
}

fn pass_owner(deps: Deps, config: &Config, id: &str) -> StdResult<Option<FounderOwner>> {
    deps.querier.query_wasm_smart(
        &config.founders_contract,
        &FounderQuery::OwnerOf { token_id: id.into() },
    )
}

fn mint(deps: DepsMut, env: &Env, wallet: &Addr, id: &str) -> Result<(), Error> {
    if TOKENS.has(deps.storage, id) || WALLET.has(deps.storage, wallet) { return Err(Error::Duplicate); }
    TOKENS.save(deps.storage, id, &Token { owner: wallet.clone(), created_at: env.block.time.seconds() })?;
    WALLET.save(deps.storage, wallet, &id.to_string())?;
    Ok(())
}

#[entry_point]
pub fn instantiate(deps: DepsMut, env: Env, info: MessageInfo, msg: InstantiateMsg) -> Result<Response, Error> {
    no_funds(&info)?;
    if env.block.chain_id != "columbus-5" || msg.image_sha256 != IMAGE_SHA256 || msg.founders_contract.trim().is_empty() {
        return Err(Error::Invalid);
    }
    CONFIG.save(deps.storage, &Config {
        founders_contract: deps.api.addr_validate(&msg.founders_contract)?,
        image_sha256: msg.image_sha256,
        max_supply: MAX_SUPPLY,
        collection_name: crate::state::COLLECTION_NAME.into(),
    })?;
    NEXT_ID.save(deps.storage, &1)?;
    COUNT.save(deps.storage, &0)?;
    Ok(Response::new().add_attribute("action", "instantiate_base_only"))
}

#[entry_point]
pub fn execute(mut deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> Result<Response, Error> {
    no_funds(&info)?;
    match msg {
        ExecuteMsg::ClaimBaseForPass { pass_token_id } => {
            if !eligible_pass_id(&pass_token_id) { return Err(Error::Unauthorized); }
            if PASS_CLAIMS.has(deps.storage, &pass_token_id) { return Err(Error::Duplicate); }
            if WALLET.has(deps.storage, &info.sender) { return Err(Error::Occupied); }
            let config = CONFIG.load(deps.storage)?;
            let pass = pass_owner(deps.as_ref(), &config, &pass_token_id)?.ok_or(Error::Unauthorized)?;
            if pass.owner != info.sender.as_str() { return Err(Error::Unauthorized); }
            let count = COUNT.load(deps.storage)?;
            if count >= MAX_SUPPLY { return Err(Error::Exhausted); }
            let serial = NEXT_ID.load(deps.storage)?;
            let token_id = format!("AV-{serial:06}");
            mint(deps.branch(), &env, &info.sender, &token_id)?;
            PASS_CLAIMS.save(deps.storage, &pass_token_id, &token_id)?;
            COUNT.save(deps.storage, &(count + 1))?;
            NEXT_ID.save(deps.storage, &(serial + 1))?;
            Ok(Response::new().add_attribute("action", "claim_base")
                .add_attribute("pass_token_id", pass_token_id)
                .add_attribute("token_id", token_id)
                .add_attribute("owner", info.sender))
        }
    }
}
