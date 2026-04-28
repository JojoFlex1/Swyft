#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec};

// ── Constants ────────────────────────────────────────────────────────────────
const KEY_MINTER: Symbol = symbol_short!("MINTER");
const KEY_NEXT_ID: Symbol = symbol_short!("NEXT_ID");
const KEY_POSITION: &[u8] = b"POSITION";

// ── Error Types ──────────────────────────────────────────────────────────────
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum PositionNftError {
    NotAuthorized = 1,
    PositionNotFound = 2,
    NotInitialized = 3,
    Overflow = 4,
}

impl From<PositionNftError> for soroban_sdk::Error {
    fn from(e: PositionNftError) -> Self {
        soroban_sdk::Error::from_contract_error(e as u32)
    }
}

// ── Types ────────────────────────────────────────────────────────────────────
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct PositionMetadata {
    pub owner: Address,
    pub pool: Address,
    pub tick_lower: i32,
    pub tick_upper: i32,
    pub liquidity: u128,
    pub created_at: u64, // Unix timestamp
}

// ── Contract ─────────────────────────────────────────────────────────────────
#[contract]
pub struct PositionNft;

#[contractimpl]
impl PositionNft {
    /// Returns the name of the contract.
    pub fn name(_env: Env) -> Symbol {
        Symbol::new(&_env, "position_nft")
    }

    /// Initialize the position NFT contract with a minter address.
    /// Only callable once.
    pub fn initialize(env: Env, minter: Address) -> Result<(), PositionNftError> {
        minter.require_auth();
        if env.storage().instance().has(&KEY_MINTER) {
            return Err(PositionNftError::NotAuthorized);
        }
        env.storage().instance().set(&KEY_MINTER, &minter);
        env.storage().instance().set(&KEY_NEXT_ID, &0u64);
        Ok(())
    }

    /// Mint a new position NFT and assign it to the owner.
    /// Only callable by the minter (pool contract).
    pub fn mint(
        env: Env,
        owner: Address,
        pool: Address,
        tick_lower: i32,
        tick_upper: i32,
        liquidity: u128,
    ) -> Result<u64, PositionNftError> {
        require_minter(&env)?;

        let id: u64 = env
            .storage()
            .instance()
            .get(&KEY_NEXT_ID)
            .unwrap_or(0u64);

        // Check for overflow
        if id == u64::MAX {
            return Err(PositionNftError::Overflow);
        }

        let created_at = env.ledger().timestamp();

        let meta = PositionMetadata {
            owner: owner.clone(),
            pool,
            tick_lower,
            tick_upper,
            liquidity,
            created_at,
        };

        let mut key = Vec::new(&env);
        key.push_back(symbol_short!("POS"));
        key.push_back(Symbol::new(&env, &id.to_string()));

        env.storage().persistent().set(&key, &meta);
        env.storage()
            .instance()
            .set(&KEY_NEXT_ID, &(id.checked_add(1).ok_or(PositionNftError::Overflow)?));

        // Emit Transfer event: (from=0, to=owner, token_id)
        env.events().publish(
            (symbol_short!("transfer"), symbol_short!("nft")),
            (Symbol::new(&env, "0x0"), owner, id),
        );

        Ok(id)
    }

    /// Burn a position NFT, removing it from existence.
    /// Only callable by the minter (pool contract).
    pub fn burn(env: Env, token_id: u64) -> Result<(), PositionNftError> {
        require_minter(&env)?;

        let mut key = Vec::new(&env);
        key.push_back(symbol_short!("POS"));
        key.push_back(Symbol::new(&env, &token_id.to_string()));

        // Get the position to emit event with the owner
        let position: Option<PositionMetadata> = env.storage().persistent().get(&key);
        if position.is_none() {
            return Err(PositionNftError::PositionNotFound);
        }

        let owner = position.unwrap().owner;
        env.storage().persistent().remove(&key);

        // Emit Transfer event: (from=owner, to=0, token_id)
        env.events().publish(
            (symbol_short!("transfer"), symbol_short!("nft")),
            (owner, Symbol::new(&env, "0x0"), token_id),
        );

        Ok(())
    }

    /// Transfer a position NFT from one address to another.
    /// The sender must be the current owner of the position.
    pub fn transfer(
        env: Env,
        token_id: u64,
        from: Address,
        to: Address,
    ) -> Result<(), PositionNftError> {
        from.require_auth();

        let mut key = Vec::new(&env);
        key.push_back(symbol_short!("POS"));
        key.push_back(Symbol::new(&env, &token_id.to_string()));

        let mut position: PositionMetadata = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(PositionNftError::PositionNotFound)?;

        // Verify the sender is the current owner
        if position.owner != from {
            return Err(PositionNftError::NotAuthorized);
        }

        // Update the owner
        position.owner = to.clone();
        env.storage().persistent().set(&key, &position);

        // Emit Transfer event
        env.events().publish(
            (symbol_short!("transfer"), symbol_short!("nft")),
            (from, to, token_id),
        );

        Ok(())
    }

    /// Get the owner of a position NFT.
    pub fn owner_of(env: Env, token_id: u64) -> Result<Address, PositionNftError> {
        let mut key = Vec::new(&env);
        key.push_back(symbol_short!("POS"));
        key.push_back(Symbol::new(&env, &token_id.to_string()));

        let position: PositionMetadata = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(PositionNftError::PositionNotFound)?;

        Ok(position.owner)
    }

    /// Get the complete metadata of a position NFT.
    pub fn get_position(
        env: Env,
        token_id: u64,
    ) -> Result<PositionMetadata, PositionNftError> {
        let mut key = Vec::new(&env);
        key.push_back(symbol_short!("POS"));
        key.push_back(Symbol::new(&env, &token_id.to_string()));

        env.storage()
            .persistent()
            .get(&key)
            .ok_or(PositionNftError::PositionNotFound)
    }

    /// Get the total number of positions ever minted (next ID to be issued).
    pub fn total_supply(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&KEY_NEXT_ID)
            .unwrap_or(0u64)
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Verify that the caller is the minter address.
fn require_minter(env: &Env) -> Result<(), PositionNftError> {
    let minter: Address = env
        .storage()
        .instance()
        .get(&KEY_MINTER)
        .ok_or(PositionNftError::NotInitialized)?;
    minter.require_auth();
    Ok(())
}

#[cfg(test)]
mod test;
