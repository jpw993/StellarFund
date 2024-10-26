
#[cfg(test)]
mod tests {
    use super::*;

    extern crate std;
    
    use soroban_sdk::{testutils::Address as TestAddress, Address, Env, Vec};
    use crate::{AlphaFund, AlphaFundClient, DataKey, FundState};

    #[test]
    fn test_create_fund() {
        let env = Env::default();
        env.mock_all_auths();
        
        let manager = Address::generate(&env);
        let token = Address::generate(&env);

        let contract_id = env.register_contract(None, AlphaFund);
        let client = AlphaFundClient::new(&env, &contract_id);

        client.create(&manager.clone(), &10, &token.clone());

        let fund_state: FundState = env.storage().persistent().get(&DataKey::FundState).unwrap();
        let stored_manager: Address = env.storage().persistent().get(&DataKey::Manager).unwrap();

        assert_eq!(fund_state, FundState::OpenToInvestors);
        assert_eq!(stored_manager, manager);
    }

    #[test]
    fn test_add_investor() {
        let env = Env::default();
        env.mock_all_auths();
        
        let manager = Address::generate(&env);
        let token = Address::generate(&env);
        let investor = Address::generate(&env);

        let contract_id = env.register_contract(None, AlphaFund);
        let client = AlphaFundClient::new(&env, &contract_id);

        client.create(&manager.clone(), &10, &token.clone());

        // Add an investor with a deposit
        client.add_investor(&investor.clone(), &100);

        let investors: Vec<Address> = AlphaFund::get_investors(&env);
        assert_eq!(investors.len(), 1);
        assert_eq!(investors.get(0).unwrap(), investor);

        // Verify the deposit amount
        let deposit_amount: i128 = env.storage().persistent().get(&DataKey::InvestorDeposit(investor.clone())).unwrap();
        assert_eq!(deposit_amount, 100);
    }

    #[test]
    fn test_add_multiple_investors() {
        let env = Env::default();
        env.mock_all_auths();
        
        let manager = Address::generate(&env);
        let token = Address::generate(&env);
        let investor1 = Address::generate(&env);
        let investor2 = Address::generate(&env);

        let contract_id = env.register_contract(None, AlphaFund);
        let client = AlphaFundClient::new(&env, &contract_id);

        client.create(&manager.clone(), &10, &token.clone());

        // Add first investor
        client.add_investor(&investor1.clone(), &200);
        // Add second investor
        client.add_investor(&investor2.clone(), &300);

        let investors: Vec<Address> = AlphaFund::get_investors(&env);
        assert_eq!(investors.len(), 2);
        assert_eq!(investors.get(0).unwrap(), investor1);
        assert_eq!(investors.get(1).unwrap(), investor2);

        // Verify the deposit amounts
        let deposit1: i128 = env.storage().persistent().get(&DataKey::InvestorDeposit(investor1.clone())).unwrap();
        let deposit2: i128 = env.storage().persistent().get(&DataKey::InvestorDeposit(investor2.clone())).unwrap();
        assert_eq!(deposit1, 200);
        assert_eq!(deposit2, 300);
    }

    #[test]
    fn test_close_fund() {
        let env = Env::default();
        env.mock_all_auths();
        
        let manager = Address::generate(&env);
        let token =Address::generate(&env);
        let investor = Address::generate(&env);

        let contract_id = env.register_contract(None, AlphaFund);
        let client = AlphaFundClient::new(&env, &contract_id);

        client.create(&manager.clone(), &10, &token.clone());

        client.add_investor(&investor.clone(), &100);

        // Close the fund
        client.close_fund(&manager.clone());

        // Check that the fund state is closed
        let fund_state: FundState = env.storage().persistent().get(&DataKey::FundState).unwrap();
        assert_eq!(fund_state, FundState::Closed);

        // Verify that the investors are paid out their deposits
        let remaining_balance = 50; // Assume some balance is left
        // Mock the transfer tokens function (you might need to set this up based on your testing framework)
        // This should simulate the balance and check if the investor received their portion
        // (additional setup may be needed to mock the actual transfer and check balances)
    }
}
