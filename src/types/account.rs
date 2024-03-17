use alloy_primitives::Address;

pub struct Account {
    pub address: Address,
    pub distributor_fee: Option<u32>,
}

impl Account {
    pub fn new(address: Address, distributor_fee: Option<u32>) -> Self {
        Self {
            address,
            distributor_fee,
        }
    }
}
