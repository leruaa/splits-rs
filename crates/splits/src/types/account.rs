use alloy::primitives::Address;

pub struct Account {
    pub address: Address,
    pub accounts: Vec<Address>,
    pub percents_allocation: Vec<u32>,
    pub distributor_fee: u32,
}

impl Account {
    pub fn new(
        address: Address,
        accounts: Vec<Address>,
        percents_allocation: Vec<u32>,
        distributor_fee: u32,
    ) -> Self {
        Self {
            address,
            accounts,
            percents_allocation,
            distributor_fee,
        }
    }
}
