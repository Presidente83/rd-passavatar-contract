use super::*;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    coin, from_json, to_json_binary, to_json_vec, Addr, Binary, Deps, DepsMut, Empty, Env,
    MessageInfo, Response, StdResult, Uint128,
};
use cw_multi_test::{App, ContractWrapper, Executor};
use msg::*;
use state::*;
const PRICE: u128 = 1_000_000_000;

fn key() -> ed25519_zebra::SigningKey {
    ed25519_zebra::SigningKey::from([7u8; 32])
}
fn pk() -> Binary {
    let k: [u8; 32] = ed25519_zebra::VerificationKey::from(&key()).into();
    k.to_vec().into()
}
fn sig(q: &FeeQuote) -> Binary {
    let s: [u8; 64] = key().sign(&to_json_vec(q).unwrap()).into();
    s.to_vec().into()
}
#[cw_serde]
struct PassInit {
    sold_out: bool,
    owners: Vec<(String, String)>,
}
#[cw_serde]
enum PassQuery {
    OwnerOf { token_id: String },
    Supply {},
    Membership { address: String },
}
fn pass_init(d: DepsMut, _: Env, _: MessageInfo, m: PassInit) -> StdResult<Response> {
    cw_storage_plus::Item::new("sold").save(d.storage, &m.sold_out)?;
    let owners: cw_storage_plus::Map<&str, String> = cw_storage_plus::Map::new("pass_owners");
    for (id, owner) in m.owners {
        owners.save(d.storage, &id, &owner)?;
    }
    Ok(Response::new())
}
fn pass_exec(_: DepsMut, _: Env, _: MessageInfo, _: Empty) -> StdResult<Response> {
    Ok(Response::new())
}
fn pass_query(d: Deps, _: Env, q: PassQuery) -> StdResult<Binary> {
    match q {
        PassQuery::OwnerOf { token_id } => {
            #[cw_serde]
            struct O {
                owner: String,
            }
            let owners: cw_storage_plus::Map<&str, String> =
                cw_storage_plus::Map::new("pass_owners");
            to_json_binary(
                &owners
                    .may_load(d.storage, &token_id)?
                    .map(|owner| O { owner }),
            )
        }
        PassQuery::Supply {} => {
            #[cw_serde]
            struct S {
                issued_pass_count: u16,
                sold_out: bool,
            }
            let s: bool = cw_storage_plus::Item::new("sold").load(d.storage)?;
            to_json_binary(&S {
                issued_pass_count: if s { 100 } else { 99 },
                sold_out: s,
            })
        }
        PassQuery::Membership { address } => {
            #[cw_serde]
            struct M {
                has_pass: bool,
            }
            to_json_binary(&M {
                has_pass: address == FOUNDER
                    || address.starts_with("cosmwasm")
                    || address.starts_with("terra1"),
            })
        }
    }
}
fn holder_address(n: u16) -> String {
    format!("holder{n:03}")
}
fn setup(sold: bool) -> (App, Addr) {
    let mut app = App::default();
    app.update_block(|block| block.chain_id = "columbus-5".into());
    let mut funded = vec![
        FOUNDER.to_string(),
        "holder001".into(),
        "holder002".into(),
        "buyer".into(),
        "other".into(),
    ];
    funded.push(holder_address(100));
    for who in funded {
        app.sudo(cw_multi_test::SudoMsg::Bank(
            cw_multi_test::BankSudo::Mint {
                to_address: who,
                amount: vec![coin(10 * PRICE, "uluna")],
            },
        ))
        .unwrap();
    }
    let pass_code = app.store_code(Box::new(ContractWrapper::new(
        pass_exec, pass_init, pass_query,
    )));
    let pass = app
        .instantiate_contract(
            pass_code,
            Addr::unchecked(FOUNDER),
            &PassInit {
                sold_out: sold,
                owners: (0u16..=100)
                    .map(|n| {
                        let id = format!("{n:03}");
                        let owner = if n == 0 {
                            FOUNDER.to_string()
                        } else {
                            holder_address(n)
                        };
                        (id, owner)
                    })
                    .collect(),
            },
            &[],
            "passes",
            None,
        )
        .unwrap();
    let code = app.store_code(Box::new(ContractWrapper::new(execute, instantiate, query)));
    let addr = app
        .instantiate_contract(
            code,
            Addr::unchecked(FOUNDER),
            &InstantiateMsg {
                founders_contract: pass.into_string(),
                image_sha256: IMAGE_SHA256.into(),
                quote_public_key: pk(),
                mint_price: Uint128::new(PRICE),
            },
            &[],
            "passavatar",
            None,
        )
        .unwrap();
    if sold {
        app.execute_contract(
            Addr::unchecked(FOUNDER),
            addr.clone(),
            &ExecuteMsg::SetSecondaryEnabled { enabled: true },
            &[],
        )
        .unwrap();
    }
    (app, addr)
}
fn q(
    app: &App,
    contract: &Addr,
    buyer: &str,
    action: &str,
    token: &str,
    revision: u64,
    gross: u128,
) -> FeeQuote {
    FeeQuote {
        chain_id: app.block_info().chain_id,
        contract: contract.to_string(),
        buyer: buyer.into(),
        action: action.into(),
        token_id: token.into(),
        revision,
        gross: Uint128::new(gross),
        network_fee: Uint128::new(1000),
        expires_at: app.block_info().time.seconds() + 90,
    }
}
fn buy_founder(app: &mut App, contract: &Addr) {
    app.execute_contract(
        Addr::unchecked(FOUNDER),
        contract.clone(),
        &ExecuteMsg::ClaimBaseForPass {
            pass_token_id: "100".into(),
            expected_owner: holder_address(100),
        },
        &[],
    )
    .unwrap();
}
#[cw_serde]
struct Num {
    count: u64,
}
fn base_msg(app: &App, c: &Addr, buyer: &str) -> ExecuteMsg {
    let count: Num = app
        .wrap()
        .query_wasm_smart(c, &QueryMsg::NumTokens {})
        .unwrap();
    let quote = q(
        app,
        c,
        buyer,
        "buy_base",
        &format!("AV-{:06}", count.count + 1),
        0,
        PRICE,
    );
    ExecuteMsg::BuyBase {
        signature: sig(&quote),
        quote,
    }
}
fn evolution_quote(app: &App, c: &Addr, total: u128) -> FeeQuote {
    let mut quote = q(
        app,
        c,
        &holder_address(100),
        "buy_evolutions",
        "AV-000001",
        0,
        total,
    );
    quote.network_fee = Uint128::new(1);
    quote
}
fn claim(app: &mut App, contract: &Addr, n: &str) {
    let pass_id = n.parse::<u16>().unwrap();
    let recipient = holder_address(pass_id);
    app.execute_contract(
        Addr::unchecked(FOUNDER),
        contract.clone(),
        &ExecuteMsg::ClaimBaseForPass {
            pass_token_id: format!("{pass_id:03}"),
            expected_owner: recipient,
        },
        &[],
    )
    .unwrap();
}
fn token(app: &App, c: &Addr, id: &str) -> Token {
    app.wrap()
        .query_wasm_smart(
            c,
            &QueryMsg::Token {
                token_id: id.into(),
            },
        )
        .unwrap()
}
fn collection(id: &str, background: bool, prerequisites: Vec<String>) -> Collection {
    Collection {
        id: id.into(),
        asset_uri: format!("https://rdstation.org/{id}.png"),
        asset_sha256: "a".repeat(64),
        prerequisites,
        incompatible: vec![],
        founder_only: false,
        background,
    }
}
fn publish(app: &mut App, c: &Addr, col: Collection, remaining: Option<u64>) {
    app.execute_contract(
        Addr::unchecked(FOUNDER),
        c.clone(),
        &ExecuteMsg::PublishCollection {
            collection: col,
            offer: Offer {
                price: Uint128::new(100),
                starts_at: 0,
                ends_at: None,
                remaining,
                revision: 0,
            },
        },
        &[],
    )
    .unwrap();
}
fn selection(app: &App, c: &Addr, id: &str) -> Selection {
    #[cw_serde]
    struct R {
        collection: Collection,
        offer: Offer,
    }
    let r: R = app
        .wrap()
        .query_wasm_smart(
            c,
            &QueryMsg::Collection {
                collection_id: id.into(),
            },
        )
        .unwrap();
    Selection {
        collection_id: id.into(),
        revision: r.offer.revision,
    }
}
#[test]
fn genesis_public_purchase_unique_and_cw721() {
    let (mut app, c) = setup(false);
    buy_founder(&mut app, &c);
    claim(&mut app, &c, "001");
    assert_eq!(token(&app, &c, "AV-000002").owner, holder_address(1));
    let quote = q(&app, &c, FOUNDER, "buy_base", "AV-000001", 0, PRICE);
    assert!(app
        .execute_contract(
            Addr::unchecked(FOUNDER),
            c.clone(),
            &ExecuteMsg::BuyBase {
                signature: sig(&quote),
                quote
            },
            &[coin(PRICE - 1000, "uluna")]
        )
        .is_err());
    let r: crate::queries::Access = app
        .wrap()
        .query_wasm_smart(
            &c,
            &QueryMsg::OwnerOf {
                token_id: "AV-000001".into(),
                include_expired: None,
            },
        )
        .unwrap();
    assert_eq!(r.owner, holder_address(100));
    assert!(r.approvals.is_empty());
    let info: crate::queries::NftInfo = app
        .wrap()
        .query_wasm_smart(
            &c,
            &QueryMsg::NftInfo {
                token_id: "AV-000001".into(),
            },
        )
        .unwrap();
    assert_eq!(info.extension.image_sha256, IMAGE_SHA256);
}
#[test]
fn rejects_forged_quotes_underpayment_wrong_buyer_and_replay_domain() {
    let (mut app, c) = setup(true);
    for mutate in 0..5 {
        let mut quote = q(&app, &c, FOUNDER, "buy_base", "AV-000001", 0, PRICE);
        let signature = sig(&quote);
        match mutate {
            0 => quote.network_fee = Uint128::new(PRICE - 1),
            1 => quote.buyer = "other".into(),
            2 => quote.contract = "othercontract".into(),
            3 => quote.chain_id = "otherchain".into(),
            _ => quote.expires_at = 1,
        };
        assert!(app
            .execute_contract(
                Addr::unchecked(FOUNDER),
                c.clone(),
                &ExecuteMsg::BuyBase { quote, signature },
                &[coin(PRICE - 1000, "uluna")]
            )
            .is_err());
    }
    let quote = q(&app, &c, FOUNDER, "buy_base", "AV-000001", 0, PRICE);
    for funds in [
        vec![],
        vec![coin(1, "uluna")],
        vec![coin(PRICE - 1000, "uusd")],
    ] {
        assert!(app
            .execute_contract(
                Addr::unchecked(FOUNDER),
                c.clone(),
                &ExecuteMsg::BuyBase {
                    signature: sig(&quote),
                    quote: quote.clone()
                },
                &funds
            )
            .is_err());
    }
    assert!(app
        .execute_contract(
            Addr::unchecked("buyer"),
            c.clone(),
            &ExecuteMsg::BuyBase {
                signature: sig(&quote),
                quote
            },
            &[coin(PRICE - 1000, "uluna")]
        )
        .is_err());
}
#[test]
fn genesis_100_pass_claims_are_sponsored_unique_and_capped() {
    let (mut app, c) = setup(false);
    for n in 1..=100 {
        let pass_id = format!("{n:03}");
        let recipient = holder_address(n);
        let msg = ExecuteMsg::ClaimBaseForPass {
            pass_token_id: pass_id.clone(),
            expected_owner: recipient.clone(),
        };
        app.execute_contract(Addr::unchecked(FOUNDER), c.clone(), &msg, &[])
            .unwrap();
        let token_id = format!("AV-{n:06}");
        assert_eq!(token(&app, &c, &token_id).owner, recipient);
        let claimed: Option<String> = app
            .wrap()
            .query_wasm_smart(
                &c,
                &QueryMsg::ClaimStatus {
                    pass_token_id: pass_id,
                },
            )
            .unwrap();
        assert_eq!(claimed.as_deref(), Some(token_id.as_str()));
        assert!(app
            .execute_contract(Addr::unchecked(FOUNDER), c.clone(), &msg, &[])
            .is_err());
    }
    let excluded_founder_claim = ExecuteMsg::ClaimBaseForPass {
        pass_token_id: "100".into(),
        expected_owner: FOUNDER.into(),
    };
    assert!(app
        .execute_contract(
            Addr::unchecked(FOUNDER),
            c.clone(),
            &excluded_founder_claim,
            &[]
        )
        .is_err());
    let invalid = ExecuteMsg::ClaimBaseForPass {
        pass_token_id: "101".into(),
        expected_owner: holder_address(101),
    };
    assert!(app
        .execute_contract(Addr::unchecked(FOUNDER), c.clone(), &invalid, &[])
        .is_err());
    let count: Num = app
        .wrap()
        .query_wasm_smart(&c, &QueryMsg::NumTokens {})
        .unwrap();
    assert_eq!(count.count, 100);
    assert!(app
        .wrap()
        .query_wasm_smart::<Token>(
            &c,
            &QueryMsg::Token {
                token_id: "AV-000000".into()
            }
        )
        .is_err());
    assert!(app
        .execute_contract(
            Addr::unchecked("buyer"),
            c.clone(),
            &base_msg(&app, &c, "buyer"),
            &[coin(PRICE, "uluna")],
        )
        .is_err());
}

#[test]
fn direct_transfers_operator_and_burn_disabled() {
    let (mut app, c) = setup(true);
    buy_founder(&mut app, &c);
    for msg in [
        ExecuteMsg::TransferNft {
            recipient: "buyer".into(),
            token_id: "AV-000001".into(),
        },
        ExecuteMsg::SendNft {
            contract: "buyer".into(),
            token_id: "AV-000001".into(),
            msg: Binary::default(),
        },
        ExecuteMsg::Approve {
            spender: "buyer".into(),
            token_id: "AV-000001".into(),
        },
        ExecuteMsg::ApproveAll {
            operator: "buyer".into(),
        },
        ExecuteMsg::Burn {
            token_id: "AV-000001".into(),
        },
    ] {
        assert!(app
            .execute_contract(Addr::unchecked(FOUNDER), c.clone(), &msg, &[])
            .is_err());
    }
}
#[test]
fn settlement_atomic_fee_owner_index_and_stale_listing() {
    let (mut app, c) = setup(true);
    buy_founder(&mut app, &c);
    claim(&mut app, &c, "001");
    app.execute_contract(
        Addr::unchecked(holder_address(100)),
        c.clone(),
        &ExecuteMsg::List {
            token_id: "AV-000001".into(),
            price: Uint128::new(100_000),
        },
        &[],
    )
    .unwrap();
    let listing: Option<Listing> = app
        .wrap()
        .query_wasm_smart(
            &c,
            &QueryMsg::Listing {
                token_id: "AV-000001".into(),
            },
        )
        .unwrap();
    let revision = listing.unwrap().revision;
    let occupied = q(
        &app,
        &c,
        &holder_address(1),
        "buy_listing",
        "AV-000001",
        revision,
        100_000,
    );
    assert!(app
        .execute_contract(
            Addr::unchecked(holder_address(1)),
            c.clone(),
            &ExecuteMsg::BuyListing {
                token_id: "AV-000001".into(),
                signature: sig(&occupied),
                quote: occupied
            },
            &[coin(99_000, "uluna")]
        )
        .is_err());
    let quote = q(
        &app,
        &c,
        "buyer",
        "buy_listing",
        "AV-000001",
        revision,
        100_000,
    );
    let balance = app.wrap().query_balance("buyer", "uluna").unwrap().amount;
    app.execute_contract(
        Addr::unchecked("buyer"),
        c.clone(),
        &ExecuteMsg::BuyListing {
            token_id: "AV-000001".into(),
            signature: sig(&quote),
            quote: quote.clone(),
        },
        &[coin(99_000, "uluna")],
    )
    .unwrap();
    assert_eq!(token(&app, &c, "AV-000001").owner, "buyer");
    assert_eq!(
        app.wrap().query_balance("buyer", "uluna").unwrap().amount,
        balance - Uint128::new(99_000)
    );
    assert!(app
        .execute_contract(
            Addr::unchecked("other"),
            c.clone(),
            &ExecuteMsg::BuyListing {
                token_id: "AV-000001".into(),
                signature: sig(&quote),
                quote
            },
            &[coin(99_000, "uluna")]
        )
        .is_err());
    let wallet: crate::queries::WalletResponse = app
        .wrap()
        .query_wasm_smart(
            &c,
            &QueryMsg::Wallet {
                address: holder_address(100),
            },
        )
        .unwrap();
    assert_eq!(wallet.token_id, None);
}
#[test]
fn evolution_bundle_atomic_and_republication_does_not_reset_ownership() {
    let (mut app, c) = setup(true);
    buy_founder(&mut app, &c);
    publish(&mut app, &c, collection("a", false, vec![]), Some(1));
    publish(
        &mut app,
        &c,
        collection("b", false, vec!["a".into()]),
        Some(0),
    );
    let a = selection(&app, &c, "a");
    let b = selection(&app, &c, "b");
    let before = app.wrap().query_balance(FOUNDER, "uluna").unwrap().amount;
    assert!(app
        .execute_contract(
            Addr::unchecked(holder_address(100)),
            c.clone(),
            &ExecuteMsg::BuyEvolutions {
                token_id: "AV-000001".into(),
                selections: vec![a.clone(), b],
                expected_total: Uint128::new(200),
                quote: evolution_quote(&app, &c, 200),
                signature: sig(&evolution_quote(&app, &c, 200)),
            },
            &[coin(199, "uluna")]
        )
        .is_err());
    assert!(token(&app, &c, "AV-000001").evolutions.is_empty());
    assert_eq!(
        before,
        app.wrap().query_balance(FOUNDER, "uluna").unwrap().amount
    );
    app.execute_contract(
        Addr::unchecked(holder_address(100)),
        c.clone(),
        &ExecuteMsg::BuyEvolutions {
            token_id: "AV-000001".into(),
            selections: vec![a],
            expected_total: Uint128::new(100),
            quote: evolution_quote(&app, &c, 100),
            signature: sig(&evolution_quote(&app, &c, 100)),
        },
        &[coin(99, "uluna")],
    )
    .unwrap();
    publish(&mut app, &c, collection("a", false, vec![]), Some(10));
    let a = selection(&app, &c, "a");
    assert!(app
        .execute_contract(
            Addr::unchecked(holder_address(100)),
            c.clone(),
            &ExecuteMsg::BuyEvolutions {
                token_id: "AV-000001".into(),
                selections: vec![a],
                expected_total: Uint128::new(100),
                quote: evolution_quote(&app, &c, 100),
                signature: sig(&evolution_quote(&app, &c, 100)),
            },
            &[coin(99, "uluna")]
        )
        .is_err());
    assert_eq!(token(&app, &c, "AV-000001").evolutions.len(), 1);
}
#[test]
fn only_one_permanent_background() {
    let (mut app, c) = setup(true);
    buy_founder(&mut app, &c);
    publish(&mut app, &c, collection("sky", true, vec![]), None);
    publish(&mut app, &c, collection("sea", true, vec![]), None);
    let sky = selection(&app, &c, "sky");
    let sea = selection(&app, &c, "sea");
    app.execute_contract(
        Addr::unchecked(holder_address(100)),
        c.clone(),
        &ExecuteMsg::BuyEvolutions {
            token_id: "AV-000001".into(),
            selections: vec![sky],
            expected_total: Uint128::new(100),
            quote: evolution_quote(&app, &c, 100),
            signature: sig(&evolution_quote(&app, &c, 100)),
        },
        &[coin(99, "uluna")],
    )
    .unwrap();
    assert!(app
        .execute_contract(
            Addr::unchecked(holder_address(100)),
            c.clone(),
            &ExecuteMsg::BuyEvolutions {
                token_id: "AV-000001".into(),
                selections: vec![sea],
                expected_total: Uint128::new(100),
                quote: evolution_quote(&app, &c, 100),
                signature: sig(&evolution_quote(&app, &c, 100)),
            },
            &[coin(99, "uluna")]
        )
        .is_err());
}
#[test]
fn malformed_legacy_buy_not_accepted() {
    assert!(from_json::<ExecuteMsg>(br#"{"buy_base":{"network_fee":"999999999"}}"#).is_err());
}

#[test]
fn marketplace_splits_five_percent_and_fee_from_distinct_seller() {
    let (mut app, c) = setup(true);
    claim(&mut app, &c, "001");
    app.execute_contract(
        Addr::unchecked(holder_address(1)),
        c.clone(),
        &ExecuteMsg::List {
            token_id: "AV-000001".into(),
            price: Uint128::new(100_000),
        },
        &[],
    )
    .unwrap();
    let listing: Option<Listing> = app
        .wrap()
        .query_wasm_smart(
            &c,
            &QueryMsg::Listing {
                token_id: "AV-000001".into(),
            },
        )
        .unwrap();
    let quote = q(
        &app,
        &c,
        "buyer",
        "buy_listing",
        "AV-000001",
        listing.unwrap().revision,
        100_000,
    );
    let before_seller = app
        .wrap()
        .query_balance(holder_address(1), "uluna")
        .unwrap()
        .amount;
    let before_treasury = app.wrap().query_balance(FOUNDER, "uluna").unwrap().amount;
    let msg = ExecuteMsg::BuyListing {
        token_id: "AV-000001".into(),
        signature: sig(&quote),
        quote,
    };
    assert!(app
        .execute_contract(
            Addr::unchecked("buyer"),
            c.clone(),
            &msg,
            &[coin(99_999, "uluna")]
        )
        .is_err());
    assert_eq!(token(&app, &c, "AV-000001").owner, holder_address(1));
    app.execute_contract(
        Addr::unchecked("buyer"),
        c.clone(),
        &msg,
        &[coin(99_000, "uluna")],
    )
    .unwrap();
    assert_eq!(
        app.wrap()
            .query_balance(holder_address(1), "uluna")
            .unwrap()
            .amount
            - before_seller,
        Uint128::new(94_000)
    );
    assert_eq!(
        app.wrap().query_balance(FOUNDER, "uluna").unwrap().amount - before_treasury,
        Uint128::new(5_000)
    );
    assert_eq!(
        app.wrap().query_balance(&c, "uluna").unwrap().amount,
        Uint128::zero()
    );
    // Paid public base minting is disabled; the original claim remains the only base mint path.
    assert!(app
        .execute_contract(
            Addr::unchecked(holder_address(1)),
            c.clone(),
            &base_msg(&app, &c, &holder_address(1)),
            &[coin(PRICE - 1000, "uluna")],
        )
        .is_err());
    assert_eq!(token(&app, &c, "AV-000001").owner, "buyer");
}

#[test]
fn stale_quote_cannot_buy_repriced_or_cancelled_listing() {
    let (mut app, c) = setup(true);
    claim(&mut app, &c, "001");
    let list = ExecuteMsg::List {
        token_id: "AV-000001".into(),
        price: Uint128::new(100_000),
    };
    app.execute_contract(Addr::unchecked(holder_address(1)), c.clone(), &list, &[])
        .unwrap();
    let listing: Option<Listing> = app
        .wrap()
        .query_wasm_smart(
            &c,
            &QueryMsg::Listing {
                token_id: "AV-000001".into(),
            },
        )
        .unwrap();
    let quote = q(
        &app,
        &c,
        "buyer",
        "buy_listing",
        "AV-000001",
        listing.unwrap().revision,
        100_000,
    );
    let buy = ExecuteMsg::BuyListing {
        token_id: "AV-000001".into(),
        signature: sig(&quote),
        quote,
    };
    app.execute_contract(Addr::unchecked(holder_address(1)), c.clone(), &list, &[])
        .unwrap();
    assert!(app
        .execute_contract(
            Addr::unchecked("buyer"),
            c.clone(),
            &buy,
            &[coin(99_000, "uluna")]
        )
        .is_err());
    app.execute_contract(
        Addr::unchecked(holder_address(1)),
        c.clone(),
        &ExecuteMsg::CancelListing {
            token_id: "AV-000001".into(),
        },
        &[],
    )
    .unwrap();
    assert!(app
        .execute_contract(
            Addr::unchecked("buyer"),
            c.clone(),
            &buy,
            &[coin(99_000, "uluna")]
        )
        .is_err());
    assert_eq!(token(&app, &c, "AV-000001").owner, holder_address(1));
}

#[test]
fn cannot_instantiate_on_another_chain() {
    let mut app = App::default();
    let code = app.store_code(Box::new(ContractWrapper::new(execute, instantiate, query)));
    assert!(app
        .instantiate_contract(
            code,
            Addr::unchecked(FOUNDER),
            &InstantiateMsg {
                founders_contract: "founders".into(),
                image_sha256: IMAGE_SHA256.into(),
                quote_public_key: pk(),
                mint_price: Uint128::new(PRICE),
            },
            &[],
            "wrong-chain",
            None
        )
        .is_err());
}

#[test]
fn secondary_market_requires_explicit_issuer_activation() {
    let (mut app, c) = setup(false);
    buy_founder(&mut app, &c);
    let config: Config = app
        .wrap()
        .query_wasm_smart(&c, &QueryMsg::Config {})
        .unwrap();
    assert!(!config.secondary_enabled);
    assert_eq!(config.max_supply, 100);
    assert_eq!(config.collection_name, COLLECTION_NAME);
    let list = ExecuteMsg::List {
        token_id: "AV-000001".into(),
        price: Uint128::new(100_000),
    };
    assert!(app
        .execute_contract(Addr::unchecked(FOUNDER), c.clone(), &list, &[])
        .is_err());
    assert!(app
        .execute_contract(
            Addr::unchecked("buyer"),
            c.clone(),
            &ExecuteMsg::SetSecondaryEnabled { enabled: true },
            &[]
        )
        .is_err());
    assert_eq!(token(&app, &c, "AV-000001").owner, holder_address(100));
}

// Documents the trust boundary: authentic quotes do not prove actual gas paid.
#[test]
fn compromised_quote_signer_can_reduce_seller_payment() {
    let (mut app, c) = setup(true);
    claim(&mut app, &c, "001");
    app.execute_contract(
        Addr::unchecked(holder_address(1)),
        c.clone(),
        &ExecuteMsg::List {
            token_id: "AV-000001".into(),
            price: Uint128::new(100_000),
        },
        &[],
    )
    .unwrap();
    let listing: Option<Listing> = app
        .wrap()
        .query_wasm_smart(
            &c,
            &QueryMsg::Listing {
                token_id: "AV-000001".into(),
            },
        )
        .unwrap();
    let mut quote = q(
        &app,
        &c,
        "buyer",
        "buy_listing",
        "AV-000001",
        listing.unwrap().revision,
        100_000,
    );
    quote.network_fee = Uint128::new(94_999);
    let before = app
        .wrap()
        .query_balance(holder_address(1), "uluna")
        .unwrap()
        .amount;
    app.execute_contract(
        Addr::unchecked("buyer"),
        c.clone(),
        &ExecuteMsg::BuyListing {
            token_id: "AV-000001".into(),
            signature: sig(&quote),
            quote,
        },
        &[coin(5_001, "uluna")],
    )
    .unwrap();
    assert_eq!(token(&app, &c, "AV-000001").owner, "buyer");
    assert_eq!(
        app.wrap()
            .query_balance(holder_address(1), "uluna")
            .unwrap()
            .amount
            - before,
        Uint128::new(1)
    );
}

#[test]
fn pause_requires_founder_and_preserves_listing_cancellation() {
    let (mut app, c) = setup(true);
    claim(&mut app, &c, "001");
    app.execute_contract(
        Addr::unchecked(holder_address(1)),
        c.clone(),
        &ExecuteMsg::List {
            token_id: "AV-000001".into(),
            price: Uint128::new(100_000),
        },
        &[],
    )
    .unwrap();
    assert!(app
        .execute_contract(
            Addr::unchecked("buyer"),
            c.clone(),
            &ExecuteMsg::SetPaused { paused: true },
            &[],
        )
        .is_err());
    app.execute_contract(
        Addr::unchecked(FOUNDER),
        c.clone(),
        &ExecuteMsg::SetPaused { paused: true },
        &[],
    )
    .unwrap();
    assert!(app
        .execute_contract(
            Addr::unchecked(FOUNDER),
            c.clone(),
            &ExecuteMsg::ClaimBaseForPass {
                pass_token_id: "002".into(),
                expected_owner: holder_address(2)
            },
            &[],
        )
        .is_err());
    app.execute_contract(
        Addr::unchecked(holder_address(1)),
        c.clone(),
        &ExecuteMsg::CancelListing {
            token_id: "AV-000001".into(),
        },
        &[],
    )
    .unwrap();
    let listing: Option<Listing> = app
        .wrap()
        .query_wasm_smart(
            &c,
            &QueryMsg::Listing {
                token_id: "AV-000001".into(),
            },
        )
        .unwrap();
    assert!(listing.is_none());
}
