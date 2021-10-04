//! Instruction types

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    sysvar,
};


/// Instruction definition
#[allow(clippy::large_enum_variant)]
// can consider making it from primitive, read as input header and manually dispatch to borsh if needed (cause transfer as most often operation is empty input)
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum TokenLockInstruction {
    Greeting,
    CreateReleaseSchedule(u32, u32, u32, u32),
}

impl TokenLockInstruction {
    pub fn greeting() ->Instruction{
        let data = TokenLockInstruction::Greeting;
        let accounts = vec![];
        Instruction::new_with_borsh(crate::id(), &data, accounts)
    }

    pub fn createReleaseSchedule(token :&Pubkey, release_count: u32, delay_until_first_release_in_seconds:u32, initial_release_portion_in_bips:u32, period_between_releases_in_seconds:u32)-> Instruction{
        let data = TokenLockInstruction::CreateReleaseSchedule(release_count, delay_until_first_release_in_seconds, initial_release_portion_in_bips, period_between_releases_in_seconds);
        let accounts = vec![
            AccountMeta::new(*token, false),
        ];
        Instruction::new_with_borsh(crate::id(), &data, accounts)
    }
}


#[cfg(test)]
mod tests {
    use super::{TokenLockInstruction};
    use borsh::{BorshDeserialize, BorshSerialize};
    use crate::{
        types::{TokenLockAccount, ReleaseSchedule}
    };
    use std::collections::HashMap;

    #[test]
    fn pack_unpack() {
        let mint = TokenLockInstruction::Greeting;
        let packed = mint.try_to_vec().unwrap();
        assert_eq!(hex::encode(packed), "00");

        // let mint = TokenLockInstruction::CreateReleaseSchedule(2, 3, 4, 5);
        // let packed = mint.try_to_vec().unwrap();
        // assert_eq!(hex::encode(packed), "00");

        // let account = TokenLockAccount{
        //     release_schedules: vec![],
        //     timelocks: HashMap::new(),
        //     total_tokens_unlocked: HashMap::new(),
        //     allowances: HashMap::new(),
        //     counter: 0 
        // };
        // let packed = account.try_to_vec().unwrap();
        // assert_eq!(hex::encode(packed), "00");


        //println!("{}", hex::encode(packed));

        // for b in packed{
        //     println!("{}", b);    
        // }
        // let unpack = NftInstruction::deserialize(&mut &packed[..]).unwrap();

        // assert_eq!(mint, unpack);

        // let mint = TokenLockInstruction::TokenLockInstruction;
        // let packed = mint.try_to_vec().unwrap();
        // let unpack = NftInstruction::deserialize(&mut &packed[..]).unwrap();

        // assert_eq!(mint, unpack);
        // assert!(matches!(unpack, NftInstruction::InitializeMint(data) if !data.name.is_empty()));
    }
}