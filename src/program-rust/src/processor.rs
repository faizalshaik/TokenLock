use crate::{
    instruction::{TokenLockInstruction},
    types::{TokenLockAccount, ReleaseSchedule, Timelock},
};

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{    
    account_info::{next_account_info, AccountInfo}, 
    entrypoint::ProgramResult, program_error::ProgramError,
    msg,
    pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
};


/// Program state handler.
pub struct Processor<'a>{
    pub account_key: &'a Pubkey,
    pub account_info: &'a mut TokenLockAccount,
    pub modified: bool
}

impl<'a> Processor<'a> {
    const BIPS_PRECISION: u32 = 10000;
    pub fn process_greeting(
        &mut self,
        program_id: &Pubkey
    ) -> ProgramResult    {       
     
        self.account_info.counter += 1;
        self.modified = true;
        msg!("Greeted {} time(s)!", self.account_info.counter);
        Ok(())
    }



    pub fn process_create_release_schedule(
        &mut self,
        program_id: &Pubkey, 
        account: &AccountInfo,
        release_count: u32,
        delay_until_first_release_in_seconds: u32,
        initial_release_portion_in_bips: u32,
        period_between_releases_in_seconds: u32
    ) -> ProgramResult {

        //checking params
        if delay_until_first_release_in_seconds > self.account_info.max_release_delay {
            msg!("first release > max delay");
            return Err(ProgramError::InvalidArgument);
        }
        if release_count < 1 {
            msg!("< 1 release");
            return Err(ProgramError::InvalidArgument);
        }
        if initial_release_portion_in_bips > Self::BIPS_PRECISION {
            msg!("release > 100%");
            return Err(ProgramError::InvalidArgument);
        }
        if release_count > 1 && period_between_releases_in_seconds ==0 {
            msg!("period = 0");
            return Err(ProgramError::InvalidArgument);
        }
        if release_count == 1 && initial_release_portion_in_bips < Self::BIPS_PRECISION {
            msg!("released < 100%");
            return Err(ProgramError::InvalidArgument);
        }

        let schedule = ReleaseSchedule{
            release_count: release_count,
            delay_until_first_release_in_seconds: delay_until_first_release_in_seconds,
            initial_release_portion_in_bips: initial_release_portion_in_bips,
            period_between_releases_in_seconds: period_between_releases_in_seconds
        };
        self.account_info.add_release_schedule(schedule);
        self.modified = true;
        msg!("process_create_release_schedule is called {}!", self.account_info.release_schedules.len());
        Ok(())
    }    

    fn fund(&mut self, to :&Pubkey, amount: u32, commencement_timestamp: u32, schedule_id: u32) -> i32 {

        //check params        
        if amount < self.account_info.min_timelock_amount {
            msg!("amount < min funding");
            return -1;
        }
        //require(to != address(0), "to 0 address");
        if schedule_id < self.account_info.release_schedules.len() as u32 {
            msg!("bad scheduleId");
            return -1;
        }
        if amount < self.account_info.release_schedules[schedule_id as usize].release_count {
            msg!("< 1 token per release");
            return -1;
        }

        // It will revert via ERC20 implementation if there's no allowance
        //token.safeTransferFrom(msg.sender, address(this), amount);
        // require(
        //     commencementTimestamp <= block.timestamp + maxReleaseDelay
        // , "commencement time out of range");

        // require(
        //     commencementTimestamp + releaseSchedules[scheduleId].delayUntilFirstReleaseInSeconds <=
        //     block.timestamp + maxReleaseDelay
        // , "initial release out of range");
        let timelock = Timelock {
            schedule_id: schedule_id,
            commencement_timestamp: commencement_timestamp,
            tokens_transferred: 0,
            total_amount: amount,
            cancelable_by: vec![],
        };
        return self.account_info.add_timelock(to, timelock);
    }    

    /**
        @notice Fund the programmatic release of tokens to a recipient.
            WARNING: this function IS CANCELABLE by cancelableBy.
            If canceled the tokens that are locked at the time of the cancellation will be returned to the funder
            and unlocked tokens will be transferred to the recipient.
        @param to recipient address that will have tokens unlocked on a release schedule
        @param amount of tokens to transfer in base units (the smallest unit without the decimal point)
        @param commencementTimestamp the time the release schedule will start
        @param scheduleId the id of the release schedule that will be used to release the tokens
        @param cancelableBy array of canceler addresses
        @return success Always returns true on completion so that a function calling it can revert if the required call did not succeed
    */
    pub fn fund_release_schedule(&mut self, to: &Pubkey, amount: u32, commencement_timestamp: u32, schedule_id: u32, cancelable_by: &[Pubkey]) -> bool {
        if cancelable_by.len() > 10 {
            msg!("max 10 cancelableBy addressees");
            return false;
        }

        let timelock_id = self.fund(to, amount, commencement_timestamp, schedule_id);
        if cancelable_by.len() > 0 {
            self.account_info.get_timelock_mut(to, timelock_id as usize).unwrap().set_cancelable_by(cancelable_by);
        }
        //emit ScheduleFunded(msg.sender, to, scheduleId, amount, commencementTimestamp, timelockId, cancelableBy);
        return true;
    }


    /**
        @notice Cancel a cancelable timelock created by the fundReleaseSchedule function.
            WARNING: this function cannot cancel a release schedule created by fundReleaseSchedule
            If canceled the tokens that are locked at the time of the cancellation will be returned to the funder
            and unlocked tokens will be transferred to the recipient.
        @param target The address that would receive the tokens when released from the timelock.
        @return success Always returns true on completion so that a function calling it can revert if the required call did not succeed
    */
    pub fn cancel_timelock(&mut self, target: &Pubkey, timelock_index: u32, reclaim_token_to: &Pubkey) -> bool {

        let timelock :&mut Timelock;
        let account_info = &self.account_info;
        if let Some(tmlock) = TokenLockAccount::get_timelock_mut(&account_info, target, timelock_index as usize){
            timelock = tmlock;
        }else
        {
            msg!("invalid timelock");
            return false;
        }

        //require(reclaimTokenTo != address(0), "Invalid reclaimTokenTo");
        let can_cancelable = timelock.has_cancelable_by(self.account_key);
        if can_cancelable == false{
            msg!("You are not allowed to cancel this timelock");
            return false;
        }

        let canceled_amount = Self::locked_balance_of_timelock(self, target, timelock_index);
        if canceled_amount == 0{
            msg!("Timelock has no value left");
            return false;
        }

        let paid_amount = self.unlocked_balance_of_timelock(target, timelock_index);

        // token.safeTransfer(reclaim_token_to, canceled_amount);
        // token.safeTransfer(target, paid_amount);
        // emit TimelockCanceled(msg.sender, target, timelockIndex, reclaimTokenTo, canceledAmount, paidAmount);

        timelock.tokens_transferred = timelock.total_amount;
        return true;
    }    


    /**
        @notice Get the unlocked balance for a specific address and specific timelock
        @param who the address to check
        @param timelockIndex for a specific timelock belonging to the who address
        @return unlocked balance of the timelock
    */
    fn unlocked_balance_of_timelock(&self, who: &Pubkey, timelock_index: u32) ->u32 {
        if let Some(timelock) = self.account_info.get_timelock(who, timelock_index as usize){
            if timelock.total_amount <= timelock.tokens_transferred {
                return 0;
            } else {
                return self.total_unlocked_to_date_of_timelock(who, timelock_index) - timelock.tokens_transferred;
            }
        }
        return 0;
    }


    /**
        @notice Get The locked balance for a specific address and specific timelock
        @param who The address to check
        @param timelockIndex Specific timelock belonging to the who address
        @return locked Balance of the timelock
    */
    pub fn locked_balance_of_timelock(&mut self, who: &Pubkey, timelock_index: u32) ->u32 {
        if let Some(timelock) = self.account_info.get_timelock_mut(who, timelock_index as usize) {
            if timelock.total_amount <= timelock.tokens_transferred {
                return 0;
            } else {
                return timelock.total_amount - self.total_unlocked_to_date_of_timelock(who, timelock_index);
            }   
        }
        return 0;
    }
    
    /**
        @notice Gets the total locked and unlocked balance of a specific address's timelocks
        @param who The address to check
        @param timelockIndex The index of the timelock for the who address
        @return total Locked and unlocked amount for the specified timelock
    */
    fn total_unlocked_to_date_of_timelock(&self, who: &Pubkey, timelock_index: u32) ->u32 {
        //let timelock = Self::timelock_of(account, who, timelock_index);
        // return calculate_unlocked(
        //     timelock.commencement_timestamp,
        //     block.timestamp,
        //     timelock.total_amount,
        //     timelock.schedule_id
        // );
        return 0;
    }


    /**
        @notice calculates how many tokens would be released at a specified time for a ReleaseSchedule struct.
            This is independent of any specific address or address's timelock.

        @param commencedTimestamp the commencement time to use in the calculation for the scheduled
        @param currentTimestamp the timestamp to calculate unlocked tokens for
        @param amount the amount of tokens
        @param releaseSchedule a ReleaseSchedule struct used to calculate the unlocked amount
        @return unlocked the total amount unlocked for the schedule given the other parameters
    */
    fn calculate_unlocked(&mut self, commenced_timestamp: u32, current_timestamp: u32, amount: u32, release_schedule: &ReleaseSchedule) -> u32{
        return self.calculate_unlocked_0(
            commenced_timestamp,
            current_timestamp,
            amount,
            release_schedule.release_count,
            release_schedule.delay_until_first_release_in_seconds,
            release_schedule.initial_release_portion_in_bips,
            release_schedule.period_between_releases_in_seconds
        );
    }    

        /**
        @notice The same functionality as above function with spread format of `releaseSchedule` arg
        @param commencedTimestamp the commencement time to use in the calculation for the scheduled
        @param currentTimestamp the timestamp to calculate unlocked tokens for
        @param amount the amount of tokens
        @param releaseCount Total number of releases including any initial "cliff'
        @param delayUntilFirstReleaseInSeconds "cliff" or 0 for immediate release
        @param initialReleasePortionInBips Portion to release in 100ths of 1% (10000 BIPS per 100%)
        @param periodBetweenReleasesInSeconds After the delay and initial release
        @return unlocked the total amount unlocked for the schedule given the other parameters
    */
    fn calculate_unlocked_0(
        &mut self, 
        commenced_timestamp: u32, 
        current_timestamp: u32, 
        amount: u32,
        release_count: u32, 
        delay_until_first_release_in_seconds: u32,
        initial_release_portion_in_bips: u32, 
        period_between_releases_in_seconds: u32) -> u32 {

        if commenced_timestamp > current_timestamp {
            return 0;
        }
        let seconds_elapsed = current_timestamp - commenced_timestamp;

        // return the full amount if the total lockup period has expired
        // unlocked amounts in each period are truncated and round down remainders smaller than the smallest unit
        // unlocking the full amount unlocks any remainder amounts in the final unlock period
        // this is done first to reduce computation
        if seconds_elapsed >= delay_until_first_release_in_seconds + (period_between_releases_in_seconds * (release_count - 1)){
            return amount;
        }

        let mut unlocked = 0;
        // unlock the initial release if the delay has elapsed
        if seconds_elapsed >= delay_until_first_release_in_seconds {
            unlocked = (amount * initial_release_portion_in_bips) / Self::BIPS_PRECISION;

            // if at least one period after the delay has passed
            if seconds_elapsed - delay_until_first_release_in_seconds >= period_between_releases_in_seconds {

                // calculate the number of additional periods that have passed (not including the initial release)
                // this discards any remainders (ie it truncates / rounds down)
                let additional_unlocked_periods = (seconds_elapsed - delay_until_first_release_in_seconds) / period_between_releases_in_seconds;

                // calculate the amount of unlocked tokens for the additionalUnlockedPeriods
                // multiplication is applied before division to delay truncating to the smallest unit
                // this distributes unlocked tokens more evenly across unlock periods
                // than truncated division followed by multiplication
                unlocked += ((amount - unlocked) * additional_unlocked_periods) / (release_count - 1);
            }
        }
        return unlocked;
    }



    pub fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        input: &[u8],
    ) -> ProgramResult{
        let accounts_iter = &mut accounts.iter();
        let account = next_account_info(accounts_iter)?;    

        let instruction =  TokenLockInstruction::try_from_slice(input).or(Err(ProgramError::InvalidInstructionData))?;
        match instruction {
            TokenLockInstruction::Greeting => {
                let mut account_info = TokenLockAccount::try_from_slice(&account.data.borrow()).unwrap();
                let processor = Processor{
                    //account: &account,
                    account_key: account.key,
                    account_info: &mut account_info,
                    modified: false
                };                    
                let res = processor.process_greeting(program_id);
                if processor.modified == true {
                    account_info.serialize(&mut &mut account.data.borrow_mut()[..])?
                }
                return res;
            }
            TokenLockInstruction::CreateReleaseSchedule(release_count, delay_until_first_release_in_seconds, initial_release_portion_in_bips, period_between_releases_in_seconds) => {
                // if let [account, ..] = accounts {
                //     let mut account_info = TokenLockAccount::try_from_slice(&account.data.borrow()).unwrap();
                //     let mut processor = Processor{
                //         account: &account,
                //         account_info: &account_info,
                //         modified: false
                //     };                    
                //     processor.process_create_release_schedule(program_id, account, release_count, delay_until_first_release_in_seconds, initial_release_portion_in_bips, period_between_releases_in_seconds);
                //     if processor.modified==true {
                //         account_info.serialize(&mut &mut account.data.borrow_mut()[..])?;
                //     }
                // } else {
                //     Err(ProgramError::NotEnoughAccountKeys)
                // }
                Ok(())
            }            
       }       

    }     
}