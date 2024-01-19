use cosmwasm_std::{
    to_binary, from_binary, AllBalanceResponse, Api, CanonicalAddr, CosmosMsg, DepsMut, Env,
    MessageInfo, QueryRequest, Response, StdError, StdResult, Storage, WasmMsg, WasmQuery, to_vec, Uint128,
    BankQuery, Binary,
    log,
};
use cosmwasm_std::{
    AllBalanceResponse, Api, CanonicalAddr, CosmosMsg, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult, WasmMsg, WasmQuery, BankQuery, Binary, QueryRequest, QueryStakersMsg, QueryRewardsMsg, StakingModuleMsg
};
use cosmwasm_std::{AllBalanceResponse, to_binary, from_binary, Storage};
use cosmwasm_std::query::QueryResponse;


const STAKING_MODULE_ADDRESS: &str = "staking"; // Replace with the actual staking module address
const ADMIN_ADDRESS: &str = "admin"; // Replace with the contract admin address

// Define your contract struct with necessary state
#[derive(Default)]
struct MyContract;

pub struct Staker {
    pub address: CanonicalAddr,
    pub rewards: Uint128,
}

pub struct State {
    pub stakers: Vec<Staker>,
}

impl Default for State {
    fn default() -> Self {
        State { stakers: vec![] }
    }
}

// Implement the contract
impl MyContract {
    fn distribute_rewards(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
        // Check if the sender is the contract admin
        if info.sender != deps.api.addr_canonicalize(ADMIN_ADDRESS)? {
            return Err(StdError::Unauthorized { backtrace: None });
        }

        // Fetch the list of stakers
        let stakers = get_stakers(deps.api, STAKING_MODULE_ADDRESS)?;

        // Calculate rewards for stakers
        let total_rewards = calculate_total_rewards(deps.api, STAKING_MODULE_ADDRESS)?;
        let reward_per_staker = total_rewards / stakers.len() as u64;

        // Distribute rewards to stakers
        distribute_rewards_to_accounts(deps.storage, stakers, reward_per_staker)?;

        // Optionally, you may want to send a custom message or log events
        let msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: STAKING_MODULE_ADDRESS.into(),
            msg: to_binary(&StakingModuleMsg::CustomMessage { /* Your custom message data */ })?,
            funds: vec![],
        });

        // Return a response with custom message and events
        Ok(Response::new()
            .add_message(msg)
            .add_attribute("action", "distribute_rewards")
            .add_attribute("reward_per_staker", reward_per_staker.to_string()))
    }
}

// Helper function to distribute rewards to a list of accounts
fn distribute_rewards_to_accounts(
    storage: &mut dyn Storage,
    stakers: Vec<CanonicalAddr>,
    reward_per_staker: u64,
) -> StdResult<Response> {
    let mut total_rewards = 0u64;

    for staker in stakers.iter() {
        distribute_reward(storage, staker, reward_per_staker)?;

        // Accumulate rewards for total_rewards attribute
        total_rewards += reward_per_staker;
    }

    // Log the total rewards distributed
    log(&format!(
        "Distributed total rewards: {}",
        total_rewards
    ));

    Ok(Response::new()
        .add_attribute("action", "distribute_rewards")
        .add_attribute("reward_per_staker", reward_per_staker.to_string())
        .add_attribute("total_rewards_distributed", total_rewards.to_string())?)
}

fn get_stakers(api: &dyn Api, staking_module_address: &str) -> StdResult<Vec<CanonicalAddr>> {
    let stakers_query = QueryStakersMsg::AllDelegations {};
    let stakers_response: StakersResponse = api.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: staking_module_address.into(),
        msg: to_binary(&stakers_query)?,
    }))?;
    let stakers: Vec<CanonicalAddr> = stakers_response.delegations.into_iter()
        .map(|delegation| api.addr_canonicalize(&delegation.delegator).unwrap())
        .collect();
    Ok(stakers)
}

// Helper function to calculate the total rewards available for distribution
fn calculate_total_rewards(api: &dyn Api, staking_module_address: &str) -> StdResult<u64> {
    let rewards_query = QueryRewardsMsg::AllRewards {};
    let rewards_response: AllRewardsResponse = api.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: staking_module_address.into(),
        msg: to_binary(&rewards_query)?,
    }))?;
    let total_rewards: u64 = rewards_response.total.into_iter().map(|(_, amount)| amount).sum();
    Ok(total_rewards)
}

// Helper function to distribute rewards to an account
fn distribute_reward(storage: &mut dyn Storage, account: &CanonicalAddr, amount: u64) -> StdResult<()> {
    // Load or initialize the state
    let mut state: State = load_state(storage)?;

    // Find the staker in the state
    if let Some(staker) = state.stakers.iter_mut().find(|s| &s.address == account) {
        // Add reward to the staker's balance
        staker.rewards += Uint128::from(amount);
    } else {
        // If the staker is not found, add them to the state with the reward
        let new_staker = Staker {
            address: account.clone(),
            rewards: Uint128::from(amount),
        };
        state.stakers.push(new_staker);
    }

    // Save the updated state to storage
    save_state(storage, &state)?;

    Ok(())
}

// Helper function to load the contract state from storage
fn load_state(storage: &dyn Storage) -> StdResult<State> {
    let key = to_vec(&b"state"[..]);
    match storage.get(&key) {
        Some(data) => from_binary(&data),
        None => Ok(Default::default()),
    }
}

// Helper function to save the contract state to storage
fn save_state(storage: &mut dyn Storage, state: &State) -> StdResult<()> {
    let key = to_vec(&b"state"[..]);
    let state_data: Vec<u8> = to_binary(state)?;
    storage.set(&key, &state_data);
    Ok(())
}


// Your contract instantiate function
fn instantiate(deps: DepsMut, _env: Env, _info: MessageInfo) -> StdResult<Response> {
    // Perform any necessary setup during contract instantiation
    Ok(Response::new())
}


fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: CosmosMsg) -> StdResult<Response> {
    match msg {
        // Handle staking module messages
        CosmosMsg::Wasm(WasmMsg::Execute { contract_addr, msg, funds }) => {
            let staking_msg: StakingModuleMsg = from_binary(&msg)?;

            match staking_msg {
                StakingModuleMsg::CustomMessage { /* Your custom message data */ } => {
                    // Implement your custom message logic
                    // ...

                    Ok(Response::default())
                }
                // Handle other staking module messages if needed
                _ => Ok(Response::default()),
            }
        }
        // Handle bank transactions (e.g., sending tokens)
        CosmosMsg::Bank(BankMsg::Send { to_address, amount, .. }) => {
            // Validate the sender (optional)
            if info.sender != deps.api.addr_canonicalize(ADMIN_ADDRESS)? {
                return Err(StdError::Unauthorized { backtrace: None });
            }

            // Implement logic for handling the bank send transaction
            // For example, you might want to distribute the sent tokens as rewards
            distribute_reward(deps.storage, &to_address, amount.amount)?;

            // Optionally, you may want to send a custom message or log events
            let custom_msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: STAKING_MODULE_ADDRESS.into(),
                msg: to_json_binary_from_slice(&StakingModuleMsg::CustomMessage { /* Your custom message data */ })?,
                funds: vec![],
            });

            // Return a response with custom message and events
            Ok(Response::new()
                .add_message(custom_msg)
                .add_attribute("action", "handle_bank_transaction")
                .add_attribute("amount_sent", amount.to_string()))
        }
        // Add other types of messages if needed
        _ => Ok(Response::default()),
    }
}
