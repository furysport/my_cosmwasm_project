use cosmwasm_std::{
    to_binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult, WasmMsg, QueryRequest,
    BankMsg, Uint128, Coin, QuerierWrapper, QueryWrapper, StakingQuery, StakingResponse, to_vec,
    log, Binary, BankQuery,Cw4ExecuteMsg,
};

use cw4::{Cw4ExecuteMsg, Cw4QueryMsg};

#[derive(Debug, Clone)]
struct StakerInfo {
    address: String,
    share: Uint128,
}

#[derive(Debug, Clone)]
struct StakersQueryResponse {
    stakers: Vec<StakerInfo>,
    total_share: Uint128,
}

#[derive(Debug, Clone)]
struct InstantiateMsg {
    cw4_contract_address: String,
}

#[derive(Debug, Clone)]
enum ExecuteMsg {
    UpdateGroup(Vec<StakerInfo>), // Add a new variant to store staker information
}

#[derive(Debug, Clone)]
enum QueryMsg {
    Stakers {},
}

impl StakersQueryResponse {
    fn new() -> Self {
        StakersQueryResponse {
            stakers: Vec::new(),
            total_share: Uint128::zero(),
        }
    }

    fn add_staker(&mut self, address: String, share: Uint128) {
        self.stakers.push(StakerInfo { address, share });
        self.total_share += share;
    }
}

impl StakerInfo {
    fn to_binary(&self) -> StdResult<Binary> {
        to_binary(&self)
    }
}

impl From<&StakerInfo> for Binary {
    fn from(staker_info: &StakerInfo) -> Self {
        to_vec(staker_info).unwrap().into()
    }
}

impl Into<StakerInfo> for Binary {
    fn into(self) -> StakerInfo {
        from_slice(&self).unwrap()
    }
}

// Implement the contract execution logic
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::UpdateGroup(stakers) => {
            // Vector to store CW4 execute messages
            let mut cw4_messages: Vec<CosmosMsg> = Vec::new();

            // Iterate through stakers and accumulate CW4 messages
            for staker_info in stakers.iter() {
                let staker_address = staker_info.address.clone();
                let staker_share = staker_info.share.clone();
                let percentage = (staker_share / stakers_query_response.total_share) * Uint128::new(100);

                // Create a Cw4ExecuteMsg::Mint message
                let mint_msg = Cw4ExecuteMsg::Mint {
                    address: staker_address.clone(),
                    amount: percentage,
                };

                // Create a WasmMsg::Execute message for CW4 contract
                let cw4_msg = CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: stakers_query_response.cw4_contract_address.clone().into(),
                    funds: vec![],
                    msg: to_binary(&mint_msg)?,
                });

                // Push the CW4 message to the vector
                cw4_messages.push(cw4_msg);
            }

            // Execute all accumulated CW4 messages in a single transaction
            let response = Response::new().add_messages(cw4_messages);
            return Ok(response);
        }
    }
}

// Implement the contract query logic
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Stakers {} => {
            // Query native token stakers and their shares
            let stakers_query_response = query_native_stakers(deps)?;

            // Convert the response to binary
            to_binary(&stakers_query_response)
        }
    }
}

fn query_native_stakers(deps: Deps) -> StdResult<StakersQueryResponse> {
    // Use deps.querier.query to query the native token staking module
    let staking_response: StakingResponse = deps.querier.query(&QueryRequest::Staking(StakingQuery::Validators {}))?;

    let mut stakers_query_response = StakersQueryResponse::new();

    for validator in staking_response.validators {
        stakers_query_response.add_staker(validator.delegator_address, validator.delegator_shares);
    }

    Ok(stakers_query_response)
}
