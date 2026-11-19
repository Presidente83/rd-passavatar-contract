use crate::{
    msg::{ExecuteMsg, InstantiateMsg},
    state::*,
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, to_json_vec, Addr, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo,
    Response, StdError, StdResult, Uint128,
};

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("unauthorized")]
    Unauthorized,
    #[error("invalid payment")]
    Payment,
    #[error("wallet already owns an avatar")]
    Occupied,
    #[error("already minted or claimed")]
    Duplicate,
    #[error("invalid configuration or input")]
    Invalid,
    #[error("unavailable or expired")]
    Unavailable,
    #[error("incompatible evolution")]
    Incompatible,
    #[error("untrusted or expired quote")]
    Quote,
    #[error("transfers and approvals outside Store settlement are disabled")]
    Restricted,
}
fn no_funds(info: &MessageInfo) -> Result<(), Error> {
    if !info.funds.is_empty() {
        return Err(Error::Payment);
    }
    Ok(())
}
fn payment(info: &MessageInfo, amount: Uint128) -> Result<(), Error> {
    if amount.is_zero() {
        return no_funds(info);
    }
    if info.funds != vec![Coin::new(amount.u128(), "uluna")] {
        return Err(Error::Payment);
    }
    Ok(())
}
fn payout(to: &Addr, amount: Uint128) -> BankMsg {
    BankMsg::Send {
        to_address: to.to_string(),
        amount: vec![Coin::new(amount.u128(), "uluna")],
    }
}
fn empty_wallet(deps: Deps, wallet: &Addr) -> Result<(), Error> {
    if WALLET.has(deps.storage, wallet) {
        return Err(Error::Occupied);
    }
    Ok(())
}
fn owner(deps: Deps, token_id: &str, sender: &Addr) -> Result<Token, Error> {
    let token = TOKENS.load(deps.storage, token_id)?;
    if &token.owner != sender {
        return Err(Error::Unauthorized);
    }
    Ok(token)
}
fn next_revision(deps: DepsMut) -> Result<u64, Error> {
    let n = REVISION
        .load(deps.storage)?
        .checked_add(1)
        .ok_or(Error::Invalid)?;
    REVISION.save(deps.storage, &n)?;
    Ok(n)
}
fn mint(deps: DepsMut, env: &Env, wallet: &Addr, id: &str) -> Result<Response, Error> {
    let serial = id
        .strip_prefix("AV-")
        .and_then(|n| n.parse::<u64>().ok())
        .ok_or(Error::Invalid)?;
    if !(1..=MAX_SUPPLY).contains(&serial)
        || id != format!("AV-{serial:06}")
        || COUNT.load(deps.storage)? >= MAX_SUPPLY
    {
        return Err(Error::Unavailable);
    }
    empty_wallet(deps.as_ref(), wallet)?;
    if TOKENS.has(deps.storage, id) {
        return Err(Error::Duplicate);
    }
    let count = COUNT
        .load(deps.storage)?
        .checked_add(1)
        .ok_or(Error::Invalid)?;
    TOKENS.save(
        deps.storage,
        id,
        &Token {
            owner: wallet.clone(),
            created_at: env.block.time.seconds(),
            evolutions: vec![],
        },
    )?;
    WALLET.save(deps.storage, wallet, &id.to_string())?;
    COUNT.save(deps.storage, &count)?;
    Ok(Response::new()
        .add_attribute("action", "mint")
        .add_attribute("token_id", id)
        .add_attribute("owner", wallet))
}
fn mint_next(mut deps: DepsMut, env: &Env, wallet: &Addr) -> Result<Response, Error> {
    let n = NEXT_ID.load(deps.storage)?;
    let next = n.checked_add(1).ok_or(Error::Invalid)?;
    let response = mint(deps.branch(), env, wallet, &format!("AV-{n:06}"))?;
    NEXT_ID.save(deps.storage, &next)?;
    Ok(response)
}
fn quote(
    deps: Deps,
    env: &Env,
    info: &MessageInfo,
    config: &Config,
    q: &FeeQuote,
    signature: &Binary,
) -> Result<(), Error> {
    let now = env.block.time.seconds();
    if q.chain_id != env.block.chain_id
        || q.contract != env.contract.address
        || q.buyer != info.sender
        || q.expires_at <= now
        || q.expires_at > now.saturating_add(120)
        || q.network_fee >= q.gross
    {
        return Err(Error::Quote);
    }
    let valid = deps
        .api
        .ed25519_verify(
            &to_json_vec(q)?,
            signature.as_slice(),
            config.quote_public_key.as_slice(),
        )
        .map_err(|_| Error::Quote)?;
    if !valid {
        return Err(Error::Quote);
    }
    Ok(())
}
#[cw_serde]
enum FounderQuery {
    OwnerOf { token_id: String },
}
#[cw_serde]
struct FounderOwner {
    owner: String,
}
fn pass_owner(deps: Deps, config: &Config, id: &str) -> StdResult<Option<FounderOwner>> {
    deps.querier.query_wasm_smart(
        &config.founders_contract,
        &FounderQuery::OwnerOf {
            token_id: id.into(),
        },
    )
}
fn eligible_pass_id(id: &str) -> bool {
    id.len() == 3
        && id.bytes().all(|b| b.is_ascii_digit())
        && id.parse::<u16>().is_ok_and(|n| (1..=100).contains(&n))
}
fn is_founder(deps: Deps, config: &Config, address: &Addr) -> StdResult<bool> {
    if address == &config.founder {
        return Ok(pass_owner(deps, config, "000")?.is_some_and(|p| p.owner == address.as_str()));
    }
    #[cw_serde]
    struct Membership {
        has_pass: bool,
    }
    #[cw_serde]
    enum Q {
        Membership { address: String },
    }
    let m: Membership = deps.querier.query_wasm_smart(
        &config.founders_contract,
        &Q::Membership {
            address: address.to_string(),
        },
    )?;
    Ok(m.has_pass)
}
#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, Error> {
    no_funds(&info)?;
    if env.block.chain_id != "columbus-5" {
        return Err(Error::Invalid);
    }
    if info.sender != FOUNDER {
        return Err(Error::Unauthorized);
    }
    if msg.image_sha256 != IMAGE_SHA256 || msg.quote_public_key.len() != 32 {
        return Err(Error::Invalid);
    }
    CONFIG.save(
        deps.storage,
        &Config {
            founder: deps.api.addr_validate(FOUNDER)?,
            founders_contract: deps.api.addr_validate(&msg.founders_contract)?,
            image_sha256: msg.image_sha256,
            quote_public_key: msg.quote_public_key,
            paused: false,
            secondary_enabled: false,
            mint_price: msg.mint_price,
            max_supply: MAX_SUPPLY,
            collection_name: COLLECTION_NAME.into(),
        },
    )?;
    NEXT_ID.save(deps.storage, &1)?;
    COUNT.save(deps.storage, &0)?;
    REVISION.save(deps.storage, &0)?;
    Ok(Response::new().add_attribute("action", "instantiate_passavatar"))
}
#[entry_point]
pub fn execute(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, Error> {
    let mut config = CONFIG.load(deps.storage)?;
    if let ExecuteMsg::SetSecondaryEnabled { enabled } = msg {
        no_funds(&info)?;
        if info.sender != config.founder {
            return Err(Error::Unauthorized);
        }
        config.secondary_enabled = enabled;
        CONFIG.save(deps.storage, &config)?;
        return Ok(Response::new()
            .add_attribute("action", "set_secondary_enabled")
            .add_attribute("enabled", enabled.to_string()));
    }
    if !config.secondary_enabled
        && matches!(msg, ExecuteMsg::List { .. } | ExecuteMsg::BuyListing { .. })
    {
        return Err(Error::Unavailable);
    }
    if let ExecuteMsg::SetPaused { paused } = msg {
        no_funds(&info)?;
        if info.sender != config.founder {
            return Err(Error::Unauthorized);
        }
        config.paused = paused;
        CONFIG.save(deps.storage, &config)?;
        return Ok(Response::new().add_attribute("action", "set_paused"));
    }
    if config.paused && !matches!(msg, ExecuteMsg::CancelListing { .. }) {
        return Err(Error::Unavailable);
    }
    match msg {
        ExecuteMsg::ClaimBaseForPass {
            pass_token_id,
            expected_owner,
        } => {
            no_funds(&info)?;
            if info.sender != config.founder {
                return Err(Error::Unauthorized);
            }
            if !eligible_pass_id(&pass_token_id) {
                return Err(Error::Unauthorized);
            }
            if PASS_CLAIMS.has(deps.storage, &pass_token_id) {
                return Err(Error::Duplicate);
            }
            let recipient = deps.api.addr_validate(&expected_owner)?;
            // NFT000/Founder is explicitly outside the Avatar Club 100 eligibility,
            // even if it ever owns a Pass in the eligible ID range.
            if recipient == config.founder {
                return Err(Error::Unauthorized);
            }
            let pass =
                pass_owner(deps.as_ref(), &config, &pass_token_id)?.ok_or(Error::Unauthorized)?;
            if pass.owner != recipient.as_str() {
                return Err(Error::Unauthorized);
            }
            empty_wallet(deps.as_ref(), &recipient)?;
            let token_id = format!("AV-{:06}", NEXT_ID.load(deps.storage)?);
            let response = mint_next(deps.branch(), &env, &recipient)?;
            PASS_CLAIMS.save(deps.storage, &pass_token_id, &token_id)?;
            Ok(response
                .add_attribute("pass_token_id", pass_token_id)
                .add_attribute("sponsored_by", config.founder))
        }
        ExecuteMsg::BuyBase { .. } => Err(Error::Unavailable),
        ExecuteMsg::List { token_id, price } => {
            no_funds(&info)?;
            owner(deps.as_ref(), &token_id, &info.sender)?;
            if price.is_zero() {
                return Err(Error::Payment);
            }
            let revision = next_revision(deps.branch())?;
            LISTINGS.save(
                deps.storage,
                &token_id,
                &Listing {
                    seller: info.sender,
                    price,
                    revision,
                    listed_at: env.block.time.seconds(),
                },
            )?;
            Ok(Response::new()
                .add_attribute("action", "list")
                .add_attribute("revision", revision.to_string()))
        }
        ExecuteMsg::CancelListing { token_id } => {
            no_funds(&info)?;
            owner(deps.as_ref(), &token_id, &info.sender)?;
            LISTINGS.remove(deps.storage, &token_id);
            Ok(Response::new().add_attribute("action", "cancel_listing"))
        }
        ExecuteMsg::BuyListing {
            token_id,
            quote: q,
            signature,
        } => {
            let listing = LISTINGS.load(deps.storage, &token_id)?;
            if q.action != "buy_listing"
                || q.token_id != token_id
                || q.revision != listing.revision
                || q.gross != listing.price
            {
                return Err(Error::Quote);
            }
            quote(deps.as_ref(), &env, &info, &config, &q, &signature)?;
            // The buyer pays the displayed gross price including the quoted
            // network fee. The seller bears that fee and the 5% commission.
            let attached = listing
                .price
                .checked_sub(q.network_fee)
                .map_err(|_| Error::Payment)?;
            payment(&info, attached)?;
            empty_wallet(deps.as_ref(), &info.sender)?;
            let mut token = owner(deps.as_ref(), &token_id, &listing.seller)?;
            let commission = listing.price.multiply_ratio(5u128, 100u128);
            let deductions = commission
                .checked_add(q.network_fee)
                .map_err(|_| Error::Payment)?;
            let seller_net = listing
                .price
                .checked_sub(deductions)
                .map_err(|_| Error::Payment)?;
            if seller_net.is_zero() {
                return Err(Error::Payment);
            }
            WALLET.remove(deps.storage, &token.owner);
            token.owner = info.sender.clone();
            TOKENS.save(deps.storage, &token_id, &token)?;
            WALLET.save(deps.storage, &info.sender, &token_id)?;
            LISTINGS.remove(deps.storage, &token_id);
            let mut result = Response::new()
                .add_attribute("action", "buy_listing")
                .add_attribute("token_id", token_id)
                .add_attribute("commission", commission)
                .add_attribute("network_fee_quote", q.network_fee)
                .add_message(payout(&listing.seller, seller_net));
            if !commission.is_zero() {
                result = result.add_message(payout(&config.founder, commission));
            }
            Ok(result)
        }
        ExecuteMsg::PublishCollection {
            collection,
            mut offer,
        } => {
            no_funds(&info)?;
            if info.sender != config.founder {
                return Err(Error::Unauthorized);
            }
            if collection.id.is_empty()
                || collection.id.len() > 64
                || !collection
                    .id
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-')
                || !collection.asset_uri.starts_with("https://")
                || collection.asset_uri.len() > 512
                || collection.asset_sha256.len() != 64
                || !collection
                    .asset_sha256
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit())
                || collection.prerequisites.len() > 16
                || collection.incompatible.len() > 16
                || offer.price.is_zero()
                || offer.ends_at.is_some_and(|end| end <= offer.starts_at)
            {
                return Err(Error::Invalid);
            }
            if let Some(old) = COLLECTIONS.may_load(deps.storage, &collection.id)? {
                if old != collection {
                    return Err(Error::Invalid);
                }
            } else {
                for id in collection
                    .prerequisites
                    .iter()
                    .chain(collection.incompatible.iter())
                {
                    if id == &collection.id || !COLLECTIONS.has(deps.storage, id) {
                        return Err(Error::Invalid);
                    }
                }
            }
            offer.revision = next_revision(deps.branch())?;
            COLLECTIONS.save(deps.storage, &collection.id, &collection)?;
            OFFERS.save(deps.storage, &collection.id, &offer)?;
            Ok(Response::new().add_attribute("action", "publish_collection"))
        }
        ExecuteMsg::BuyEvolutions {
            token_id,
            selections,
            expected_total,
            quote: q,
            signature,
        } => {
            let mut token = owner(deps.as_ref(), &token_id, &info.sender)?;
            if selections.is_empty() || selections.len() > 16 {
                return Err(Error::Invalid);
            }
            let now = env.block.time.seconds();
            let mut total = Uint128::zero();
            let mut updates = vec![];
            for selection in selections {
                let c = COLLECTIONS.load(deps.storage, &selection.collection_id)?;
                let mut offer = OFFERS.load(deps.storage, &selection.collection_id)?;
                if selection.revision != offer.revision
                    || now < offer.starts_at
                    || offer.ends_at.is_some_and(|end| now >= end)
                    || offer.remaining == Some(0)
                {
                    return Err(Error::Unavailable);
                }
                if token
                    .evolutions
                    .iter()
                    .any(|e| e.collection_id == c.id || (e.background && c.background))
                {
                    return Err(Error::Duplicate);
                }
                if c.founder_only && !is_founder(deps.as_ref(), &config, &info.sender)? {
                    return Err(Error::Unauthorized);
                }
                for id in &c.prerequisites {
                    if !token.evolutions.iter().any(|e| &e.collection_id == id) {
                        return Err(Error::Incompatible);
                    }
                }
                for e in &token.evolutions {
                    let old = COLLECTIONS.load(deps.storage, &e.collection_id)?;
                    if c.incompatible.contains(&old.id) || old.incompatible.contains(&c.id) {
                        return Err(Error::Incompatible);
                    }
                }
                total = total.checked_add(offer.price).map_err(|_| Error::Payment)?;
                if let Some(n) = offer.remaining {
                    offer.remaining = Some(n - 1);
                }
                token.evolutions.push(Evolution {
                    collection_id: c.id.clone(),
                    applied_at: now,
                    asset_uri: c.asset_uri,
                    asset_sha256: c.asset_sha256,
                    background: c.background,
                });
                updates.push((c.id, offer));
            }
            if total != expected_total {
                return Err(Error::Payment);
            }
            if q.action != "buy_evolutions"
                || q.token_id != token_id
                || q.revision != 0
                || q.gross != total
            {
                return Err(Error::Quote);
            }
            quote(deps.as_ref(), &env, &info, &config, &q, &signature)?;
            let net = total
                .checked_sub(q.network_fee)
                .map_err(|_| Error::Payment)?;
            payment(&info, net)?;
            for (id, offer) in updates {
                OFFERS.save(deps.storage, &id, &offer)?;
            }
            TOKENS.save(deps.storage, &token_id, &token)?;
            Ok(Response::new()
                .add_attribute("action", "buy_evolutions")
                .add_attribute("token_id", token_id)
                .add_message(payout(&config.founder, net)))
        }
        _ => Err(Error::Restricted),
    }
}
