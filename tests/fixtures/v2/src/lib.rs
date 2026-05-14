#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Env, String};

#[contracttype]
#[derive(Clone)]
pub struct ConfigData {
    pub admin: String,
    // BREAK: threshold removed
}

#[contracttype]
#[derive(Clone, Copy)]
pub enum StatusEvent {
    Active = 1,
    Paused = 3, // BREAK: value changed from 2 to 3
    Archived = 4, // Added case
}

#[contract]
pub struct MockContract;

#[contractimpl]
impl MockContract {
    // BREAK: new argument added, return type removed
    pub fn initialize(_env: Env, _admin: String, _extra_param: u32) {
    }

    pub fn execute_action(_env: Env, _config: ConfigData, _status: StatusEvent) {
        // do nothing
    }
}
