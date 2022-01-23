#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Addr};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{OwnerResponse, ScoreResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE, SCORES};

const CONTRACT_NAME: &str = "crates.io:passport_assessment";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        owner: msg.owner,
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetScore {address, new_score} => try_set_score(deps, info, address, new_score),
    }
}

pub fn try_set_score(deps: DepsMut, info: MessageInfo, address: Addr, new_score: i32) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;
    if info.sender != state.owner {
        return Err(ContractError::Unauthorized {});
    }
    SCORES.update(deps.storage, address,
                  |_: Option<i32>| -> StdResult<i32> {
                      return Ok(new_score);
                  })?;
    Ok(Response::new().add_attribute("method", "set_score"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetOwner {} => to_binary(&query_owner(deps)?),
        QueryMsg::GetScore { address } => to_binary(&query_score(deps, address)?),
    }
}

fn query_score(deps: Deps, address: Addr) -> StdResult<ScoreResponse> {
    // Return 0 as a default if the user is not in the stored state.
    match SCORES.may_load(deps.storage, address)? {
        Some(number) => Ok(ScoreResponse { score: number }),
        None => Ok(ScoreResponse { score: 0 }),
    }
}

fn query_owner(deps: Deps) -> StdResult<OwnerResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(OwnerResponse { owner: state.owner })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn initialization_test() {
        // Setup mock data
        let mut deps = mock_dependencies(&[]);
        let msg = InstantiateMsg { owner: Addr::unchecked("owner") };
        let info = mock_info("creator", &coins(1000, "earth"));

        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    }

    #[test]
    fn query_owner_test() {
        // Setup mock data, note that sender and owner addresses are different
        let mut deps = mock_dependencies(&[]);
        let msg = InstantiateMsg { owner: Addr::unchecked("owner") };
        let info = mock_info("anyone", &coins(1000, "earth"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetOwner {}).unwrap();
        let value: OwnerResponse = from_binary(&res).unwrap();
        assert_eq!("owner", value.owner);
    }

    #[test]
    fn set_score_test() {
        // Setup mock
        let mut deps = mock_dependencies(&[]);
        let inst_msg = InstantiateMsg { owner: Addr::unchecked("owner") };
        let owner = mock_info("owner", &coins(500, "earth"));
        let _res = instantiate(deps.as_mut(), mock_env(), owner, inst_msg).unwrap();

        // Only the owner should be able to set scores
        let anyone = mock_info("anyone", &coins(1000, "earth"));
        let exec_msg = ExecuteMsg::SetScore { address: Addr::unchecked("someone"), new_score: 50};
        let res =  execute(deps.as_mut(), mock_env(), anyone, exec_msg);
        assert!(res.is_err());

        // The owner should be able to set scores
        let owner = mock_info("owner", &coins(500, "earth"));
        let exec_msg = ExecuteMsg::SetScore { address: Addr::unchecked("someone"), new_score: 50};
        let _res =  execute(deps.as_mut(), mock_env(), owner, exec_msg).unwrap();
    }

    #[test]
    fn query_score_test() {
        // Setup mock
        let mut deps = mock_dependencies(&[]);
        let inst_msg = InstantiateMsg { owner: Addr::unchecked("owner") };
        let owner = mock_info("owner", &coins(500, "earth"));
        let _res = instantiate(deps.as_mut(), mock_env(), owner, inst_msg).unwrap();

        // Set someone's score to 2022
        let owner = mock_info("owner", &coins(500, "earth"));
        let exec_msg = ExecuteMsg::SetScore { address: Addr::unchecked("someone"), new_score: 2022};
        let _res =  execute(deps.as_mut(), mock_env(), owner, exec_msg).unwrap();

        // Check that someone's score updated to 2022
        let score_msg = QueryMsg::GetScore { address: Addr::unchecked("someone") };
        let res = query(deps.as_ref(), mock_env(), score_msg).unwrap();
        let value: ScoreResponse = from_binary(&res).unwrap();
        assert_eq!(2022, value.score);
    }
}
