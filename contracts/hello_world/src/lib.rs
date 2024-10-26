#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Vec, IntoVal, symbol_short};

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum FundState {
    OpenToInvestors,
    Trading,
    Closed,
}

#[derive(Clone, PartialEq, Eq)]
#[contracttype]
pub enum DataKey {
    FundState,
    Manager,
    Trader(Address),
    TradingAllocation(Address),
    InvestorDeposit(Address),
    TotalDeposited,
    PerformanceFeePercent,
    Token,
}

#[contract]
pub struct AlphaFund;

#[contractimpl]
impl AlphaFund {
    pub fn create(env: Env, manager: Address, performance_fee_percent: i128, token: Address) {
        assert!(performance_fee_percent < 100, "Performance fee must be less than 100");

        env.storage().persistent().set(&DataKey::FundState, &FundState::OpenToInvestors);
        env.storage().persistent().set(&DataKey::Manager, &manager);
        env.storage().persistent().set(&DataKey::TradingAllocation(manager.clone()), &0i128);
        env.storage().persistent().set(&DataKey::PerformanceFeePercent, &performance_fee_percent);
        env.storage().persistent().set(&DataKey::Token, &token);

        // Initialize the traders list with the manager
        let traders = Vec::from_slice(&env, &[manager.clone()]);
        env.storage().persistent().set(&DataKey::Trader(manager.clone()), &traders);
    }

    pub fn close_fund(env: Env, manager: Address) {
        manager.require_auth();
        let state: FundState = env.storage().persistent().get(&DataKey::FundState).unwrap_or(FundState::Closed);
        assert!(state != FundState::Closed, "Fund is already closed");

        let total_deposited: i128 = env.storage().persistent()
            .get(&DataKey::TotalDeposited)
            .unwrap_or(0);

        // Check if there's profit
        let contract_balance = Self::get_contract_balance(&env);
        if contract_balance > total_deposited {
            let profit = contract_balance - total_deposited;
            let performance_fee_percent: i128 = env.storage().persistent()
                .get(&DataKey::PerformanceFeePercent)
                .unwrap_or(0);
            let total_performance_fee = (profit * performance_fee_percent) / 100;

            // Distribute performance fee to traders based on allocation
            let traders: Vec<Address> = Self::get_traders(&env);
            for trader in traders.iter() {
                let alloc_percent = env.storage().persistent()
                    .get(&DataKey::TradingAllocation(trader.clone()))
                    .unwrap_or(0);
                if alloc_percent > 0 {
                    let trader_fee = (total_performance_fee * alloc_percent) / 100;
                    Self::transfer_tokens(&env, &trader, trader_fee);
                }
            }
        }

        // Distribute remaining balance to investors based on deposits
        let remaining_balance = Self::get_contract_balance(&env);
        let investors: Vec<Address> = Self::get_investors(&env);
        for investor in investors.iter() {
            let deposit_amt = env.storage().persistent()
                .get(&DataKey::InvestorDeposit(investor.clone()))
                .unwrap_or(0);
            if deposit_amt > 0 {
                let percentage = (deposit_amt * 100) / total_deposited;
                let fraction_to_pay = (remaining_balance * percentage) / 100;
                Self::transfer_tokens(&env, &investor, fraction_to_pay);
            }
        }

        env.storage().persistent().set(&DataKey::FundState, &FundState::Closed);
    }

    fn get_contract_balance(env: &Env) -> i128 {
        let token: Address = env.storage().persistent().get(&DataKey::Token).unwrap();
        let contract_id = env.current_contract_address();

        // Explicitly specify the expected return type when calling invoke_contract
        let balance: i128 = env
            .invoke_contract::<i128>(&token, &symbol_short!("balance"), (contract_id,).into_val(env));

        balance
    }


    fn transfer_tokens(env: &Env, recipient: &Address, amount: i128) {
        let token: Address = env.storage().persistent().get(&DataKey::Token).unwrap();
        let contract_id = env.current_contract_address();
        env.invoke_contract::<()>(
            &token,
            &symbol_short!("transfer"),
            (contract_id, recipient, amount).into_val(env)
        );
    }

    fn get_traders(env: &Env) -> Vec<Address> {
        let manager: Address = env.storage().persistent().get(&DataKey::Manager).unwrap();
        env.storage().persistent().get(&DataKey::Trader(manager)).unwrap_or(Vec::new(&env))
    }

    fn get_investors(env: &Env) -> Vec<Address> {
        // Assuming investor addresses are tracked similarly to traders; you might need a dedicated investor list
        Vec::new(&env)
    }
}