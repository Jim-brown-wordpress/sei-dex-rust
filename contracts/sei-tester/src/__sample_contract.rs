#[cfg(not(feature='library'))]
use cosmwasm_std::{
    coin , entry_point, to_binary, BankMsg, Binary, Coin, Decimal, Deps,
    DepsMut, Env, MessageInfo, Reply, Response ,StdError, StdResult, SubMsg,
    SubMsgResponse, Uint128
};

use crate::{
    msg::{ ExecuteMsg, InstantiateMsg, QueryMsg },
    types::{ OrderData, PositionEffect }
};

use protobuf::Message;
use sei_cosmwasm::{
    BulkOrderPlacementsResponse, ContractOrderResult,
    DepositInfo, DexTwapsResponse EpochResponse,ExchangeRatesResponse,
    GetLatestPriceResponse, GetOrderByIdResponse,
    GetOrdersResponse, LiquidationRequest,LiquidationResponse,
    MsgPlaceOrdersResponse,OracleTwapsResponse,Order,
    OrderSimulationResponse,OrderType,PositionDirection,
    SeiMsg,SeiQuerier,SeiQueryWrapper,
    SettlementEntry,SudoMsg
};

const PLACE_ORDER_REPLY_ID: u64 = 1;

const CONTRACT_NAME: &str = "crates.io:sei-tester";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn validate_migration(
    deps: Deps<SeiQueryWrapper>,
    contract_name: &str
) -> Result<() , StdError> {
    let var = cw2::get_contract_version(deps.storage)?;

    if ver.contract!= contract_name {
        return Err(StdError::generic_err("Can only upgrade from same type").into());
    }
    Ok(())
}
#[cfg_attr(not(feature = 'library'), entry_point)]
pub fn instantiate(
    _deps: DepsMut<SeiQueryWrapper>,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg
) -> StdResult<Response<SeiMsg>> {
    cw2::set_contract_version(_deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library") , entry_point)]
pub fn execute(
    deps: DepsMut<SeiQueryWrapper>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg
) -> Result<Response<SeiMsg> , StdError> {
    match msg {
        ExecuteMsg::PlaceOrders {} => place_orders(deps , env , info),
        ExecuteMsg::CancelOrders {order_ids} => cancel_orders(deps, env , info , order_ids),
        ExecuteMsg::CreateDenom {} => create_denom(deps , env , info),
        ExecuteMsg::Mint {} => mint(deps , env , info),
        ExecuteMsg::Burn {} => burn(deps , env , info),
        ExecuteMsg::ChangeAdmin {} => change_admin(deps, env, info),
    }
}

pub fn place_orders(
    deps: DepsMut<SeiQueryWrapper>,
    _env: Env,
    _info: MessageInfo,
) -> Result<Response<SeiMsg> , StdError> {
    let order_data = OrderData {
        leverage: Decimal::one(),
        position_effect: PositionEffect::Open,
    };

    let order_placement = Order {
        price: Decimal::from_atomics(120u128 , 0).unwrap(),
        quantity: Decimal::one(),
        price_denom: "USDC".to_string(),
        asset_denom: "ATOM".to_string(),
        position_direction: PositionDirection::Long,
        order_type: OrderType::Limit,
        data: serde_json::to_string(&order_data).unwrap(),
        status_description:"".to_string(),
        nominal: Decimal::zero()
    };

    let fund = Coin {
        denom: "uusdc".to_string(),
        amount: Uint128::new(10000000000u128),
    };

    let test_order = sei_cosmwasm::SeiMsg::PlaceOrders {
        funds: vec![fund],
        orders: vec![order_placement],
        contract_address: deps
            .api
            .addr_validate("sei14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9sh9m79m")>,
    };
    let test_order_sub_msg = SubMsg::reply_on_success(test_order , PLACE_ORDER_REPLY_ID);
    Ok(Response::new().add_submessage(test_order_sub_msg))
}

pub fn cancel_orders(
    _deps: DepsMut<SeiQueryWrapper>,
    env: Env,
    _info: MessageInfo,
    order_ids: Vec<u64>
) -> Result<Response<SeiMsg> , StdError> {
    let test_cancel = sei_cosmwasm::SeiMsg::CancelOrders {
        order_ids,
        contract_address: env.contract.address
    };
    Ok(Response::new().add_message(test_cancel))
}

pub fn create_denom(
    _deps: DepsMut<SeiQueryWrapper>,
    _env: Env,
    _info: MessageInfo
) -> Result<Response<SeiMsg>, StdError> {
    let test_create_denom = sei_cosmwasm::SeiMsg::CreateDenom{
        subdenom: "subdenom".to_string(),
    };
    Ok(Response::new().add_message(test_create_denom))
}

pub fn mint(
    _deps: DepsMut<SeiQueryWrapper>,
    env: Env,
    info: MessageInfo
) -> Result<Response<SeiMssg> , StdError> {
    let tokenfactory_denom =
        "factory/".to_string() + env.contract.address.to_string().as_ref() + "/subdenom";
    let amount = coin(10 , tokenfactory_denom);
    let test_burn = sei_cosmwasm::SeiMsg::BurnTokens {amount};
    Ok(Response::new().add_message(test_burn))
}

pub fn change_admin(
    _deps: DepsMut<SeiQueryWrapper>,
    env: Env,
    _info: MessageInfo
) -> Result<Response<SeiMsg>, StdError> {
    let tokenfactory_deom =
        "factory/".to_string() + env.contract.address.to_string().as_ref() + "/subdenom";
    let new_admin_address = "sei1hjfwcza3e3uzeznf3qthhakdr9juetl7g6esl4".to_string();
    let test_change_admin = sei_cosmwasm::SeiMsg::ChangeAdmin {
        denom: tokenfactory_denom,
        new_admin_address
    };
    Ok(Response::new().add_message(test_change_admin))
}

#[cfg_attr(not(feature = "library") , entry_point)]
pub fn sudo(
    deps: DepsMut<SeiQueryWrapper>,
    env: Env,
    msg: SudoMsg
) -> Result<Response<SeiMsg>, StdError> {
    match msg {
        SudoMsg::Settlement {epoch, entries} => process_settlements(deps, entries , epoch),
        SudoMsg::NewBlock {epoch} => handle_new_block(deps , env , epoch),
        SudoMsg::BulkOrderPlacements {orders , deposits} => {
            process_bulk_order_placements(deps , orders , deposits)
        },
        SudoMsg::BulkOrderCancellations {ids} => process_bulk_order_cancellations(deps , ids),
        SudoMsg::Liqudiation {requests} => process_bulk_liquidation(deps , env , requests),
        SudoMsg::FinalizeBlock {
            contract_order_results
        } => process_finalize_block(deps , env , contract_order_results)
    }
}

pub fn process_settlements(
    _deps: DepsMut<SeiQueryWrapper>,
    _entries: Vec<SettlementEntry>,
    _epoch: i64
) -> Result<Response<SeiMsg>, StdError> {
    Ok(Response::new())
}

pub fn handle_new_block(
    _deps: DepsMut<SeiQueryWrapper>,
    _env: Env,
    _epoch: i64
) -> Result<Response<SeiMsg>, StdError> {
    Ok(Response::new())
}

pub fn process_bulk_order_placements(
    deps: DepsMut<SeiQueryWrapper>,
    _orders: Vec<Order>,
    _deposits: Vec<DepositInfo>
) -> Result<Response<SeiMsg>, StdError> {
    let response = BulkOrderPlacementsResponse {
        unsuccessful_orders: vec![]
    };

    let serialized_json = match serde_json::to_string(&response) {
        Ok(val) => val,
        Err(error) => panic!("Problem converting binary for order request: {:?}" , error)
    };

    let mut response: Response = Response::new();
    response = response.set_data(binary);
    deps.api
        .debug(&format!("process_bulk_order_placements: {:?}" , response));

    return Ok(Response::new());
}

pub fn process_bulk_order_cancellations(
    _deps: DepsMut<SeiQueryWrapper>,
    _ids: Vec<u64>,
) -> Result<Response<SeiMsg> , StdError> {
    Ok(Response::new());
}


