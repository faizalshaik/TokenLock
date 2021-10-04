//! State transition types
use std::collections::HashMap;
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{
    account_info::{AccountInfo}};
use solana_program::pubkey::Pubkey;    

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, BorshSchema)]
pub struct ReleaseSchedule {
    pub release_count: u32,
    pub delay_until_first_release_in_seconds: u32,
    pub initial_release_portion_in_bips: u32,
    pub period_between_releases_in_seconds: u32,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, BorshSchema)]
pub struct Timelock {
    pub schedule_id: u32,
    pub commencement_timestamp: u32,
    pub tokens_transferred: u32,
    pub total_amount: u32,
    pub cancelable_by: Vec<Pubkey>,
}
impl Timelock{
    pub fn set_cancelable_by(&mut self, cancelable_by: &[Pubkey]) -> (){
        self.cancelable_by = cancelable_by.to_vec();
    }
    pub fn has_cancelable_by(&self, cancelable_by: &Pubkey) -> bool{
        for item in &self.cancelable_by{
            if item == cancelable_by {
                return true;
            }
        }
        return false;
    }
}


#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, BorshSchema)]
pub struct TokenLockAccount {
    pub counter: u32,    
    pub max_release_delay: u32,    
    pub min_timelock_amount: u32,    
    pub release_schedules: Vec<ReleaseSchedule>,
    pub timelocks: HashMap<Pubkey, Vec<Timelock>>,
    pub total_tokens_unlocked: HashMap<Pubkey, u32>,
    pub allowances: HashMap<Pubkey, HashMap<Pubkey, u32>>,
}

impl TokenLockAccount{
    pub fn add_timelock(&mut self, addr: &Pubkey, time_lock: Timelock)-> i32{
        if let Some(timelocks_for_addr) = self.timelocks.get_mut(addr){
            timelocks_for_addr.push(time_lock);
            return (timelocks_for_addr.len() - 1) as i32;
        }
        return -1;
    }

    pub fn get_timelock(&self, addr: &Pubkey, idx: usize)-> Option<&Timelock>{
        if let Some(timelocks_for_addr) = self.timelocks.get(addr){
            return timelocks_for_addr.get(idx);
        }
        return None;
    }

    pub fn get_timelock_mut(&mut self, addr: &Pubkey, idx: usize)-> Option<&mut Timelock>{
        if let Some(timelocks_for_addr) = self.timelocks.get_mut(addr){
            return timelocks_for_addr.get_mut(idx);
        }
        return None;
    }

    pub fn add_release_schedule(&mut self, release_schedule: ReleaseSchedule) ->i32{
        self.release_schedules.push(release_schedule);
        return (self.release_schedules.len()-1) as i32;
    }
   
}
