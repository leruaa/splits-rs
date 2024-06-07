use alloy::primitives::{address, Address};

pub struct Config {
    pub main_address: Address,
}

impl Config {
    pub fn new(chain_id: u8) -> Self {
        match chain_id {
            1 => Self {
                main_address: address!("2ed6c4B5dA6378c7897AC67Ba9e43102Feb694EE"),
            },
            other => unimplemented!("The chain {other} is not supported"),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new(1)
    }
}
