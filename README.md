# Token Lockup and SPL Smart Contracts

**WARNING: DO NOT DEPLOY THE `main` BRANCH TO A PRODUCTION ENVIRONMENT SUCH AS ETHEREUM MAINNET** 

The code in the main branch is under active development and there may be significant bugs or security issues introduced that have not been caught by code review or independent security auditors.  

It is recommended you use versioned releases where there is an attached audit report. The independent audits are typically conducted for specific git commit identifiers specified in the security audit reports. It is highly advisable to perform your own audit of the smart contracts to both understand what you are deploying and to independently assess the security of the code.

This Open Source software is provided "as is" with no warranty as specified in the [LICENSE](LICENSE) file.

## Overview

This is an Solana SPL standard compatible token and TokenLockup scheduled release "vesting" smart contract that:

* Does not have centralized controllers or admin roles to demonstrate strong decentralization and increased trust
* Can enforce a scheduled release of tokens (e.g. investment lockups)
* The maximum number of tokens is minted on deployment and it is not possible exceed this number
* Smart contract enforced lockup schedules are used to control the circulating supply instead of inflationary minting. 
* Allows for burning tokens to reduce supply (e.g. for permanent cross chain transfers to a new blockchain and burning excess reserve tokens to support token price)
* Optimized to decrease the use of gas for the costly transfer schedules

### At A Glance

| Feature               | Value                                                        |
| --------------------- | ------------------------------------------------------------ |
| Network               | Solana                                                       |
| Protocol              | SPL                                                          |
| `mint()`              | no tokens minted ever after deployment                       |
| `freeze()`            | never                                                        |
| `burn()`              | Only from transaction senders own wallet address. No one can burn from someone else's address. |
| Admin Roles           | None                                                         |
| Upgradeable           | No                                                           |
| Transfer Restrictions | None                                                         |
| Additional Functions  | Unlock Schedule related functions                            |
| Griefer Protection    | Minimum locked scheduled token amount slashing               |

# Dev Environment

Clone this repo and `cd` into root. Then:
The following dependencies are required to build and run this example, depending
on your OS, they may already be installed:

- Install node (v14 recommended)
- Install npm
- Install the latest Rust stable from https://rustup.rs/
- Install Solana v1.7.11 or later from
  https://docs.solana.com/cli/install-solana-cli-tools

If this is your first time using Rust, these [Installation
Notes](README-installation-notes.md) might be helpful.

### Configure CLI

> If you're on Windows, it is recommended to use [WSL](https://docs.microsoft.com/en-us/windows/wsl/install-win10) to run these commands

1. Set CLI config url to localhost cluster

```bash
solana config set --url localhost
```

2. Create CLI Keypair

If this is your first time using the Solana CLI, you will need to generate a new keypair:

```bash
solana-keygen new
```

### Start local Solana cluster

This example connects to a local Solana cluster by default.

Start a local Solana cluster:
```bash
solana-test-validator
```
> **Note**: You may need to do some [system tuning](https://docs.solana.com/running-validator/validator-start#system-tuning) (and restart your computer) to get the validator to run

Listen to transaction logs:
```bash
solana logs
```

### Install npm dependencies

```bash
npm install
```

### Build the on-chain program

There is both a Rust and C version of the on-chain program, whichever is built
last will be the one used when running the example.

```bash
npm run build:program-rust
```

```bash
npm run build:program-c
```

### Deploy the on-chain program

```bash
solana program deploy dist/program/helloworld.so
```

### Run the JavaScript client

```bash
npm run start
```
