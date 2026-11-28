use super::*;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{coin, to_json_binary, Addr, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult};
use cw_multi_test::{App, ContractWrapper, Executor};
use msg::*;
use state::*;

#[cw_serde]
struct PassInit { owner: String }
#[cw_serde]
enum PassQuery { OwnerOf { token_id: String } }

fn pass_init(d: DepsMut, _: Env, _: MessageInfo, m: PassInit) -> StdResult<Response> {
    cw_storage_plus::Item::new("owner").save(d.storage, &m.owner)?;
    Ok(Response::new())
}
fn pass_exec(_: DepsMut, _: Env, _: MessageInfo, _: Empty) -> StdResult<Response> { Ok(Response::new()) }
fn pass_query(d: Deps, _: Env, q: PassQuery) -> StdResult<Binary> {
    match q {
        PassQuery::OwnerOf { token_id } if token_id == "001" => {
            #[cw_serde]
            struct O {
                token_id: String,
                owner: String,
                locked_listing_id: Option<String>,
            }
            to_json_binary(&Some(O {
                token_id,
                owner: cw_storage_plus::Item::<String>::new("owner").load(d.storage)?,
                locked_listing_id: None,
            }))
        }
        PassQuery::OwnerOf { .. } => to_json_binary(&Option::<String>::None),
    }
}

fn setup() -> (App, Addr, Addr) {
    let mut app = App::default();
    app.update_block(|b| b.chain_id = "columbus-5".into());
    let pass_code = app.store_code(Box::new(ContractWrapper::new(pass_exec, pass_init, pass_query)));
    let pass = app.instantiate_contract(pass_code, Addr::unchecked("creator"), &PassInit { owner: "holder".into() }, &[], "passes", None).unwrap();
    let code = app.store_code(Box::new(ContractWrapper::new(execute, instantiate, query)));
    let avatar = app.instantiate_contract(code, Addr::unchecked("creator"), &InstantiateMsg { founders_contract: pass.to_string(), image_sha256: IMAGE_SHA256.into() }, &[], "avatar", None).unwrap();
    (app, avatar, pass)
}

#[test]
fn eligible_holder_can_claim_once() {
    let (mut app, avatar, _) = setup();
    let result = app.execute_contract(Addr::unchecked("holder"), avatar.clone(), &ExecuteMsg::ClaimBaseForPass { pass_token_id: "001".into() }, &[]);
    assert!(result.is_ok());
    let result = app.execute_contract(Addr::unchecked("holder"), avatar, &ExecuteMsg::ClaimBaseForPass { pass_token_id: "001".into() }, &[]);
    assert!(result.is_err());
}

#[test]
fn non_holder_and_funded_claim_are_rejected() {
    let (mut app, avatar, _) = setup();
    assert!(app.execute_contract(Addr::unchecked("other"), avatar.clone(), &ExecuteMsg::ClaimBaseForPass { pass_token_id: "001".into() }, &[]).is_err());
    assert!(app.execute_contract(Addr::unchecked("holder"), avatar, &ExecuteMsg::ClaimBaseForPass { pass_token_id: "001".into() }, &[coin(1, "uluna")]).is_err());
}
