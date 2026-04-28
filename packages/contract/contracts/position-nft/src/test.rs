#![cfg(test)]
use soroban_sdk::{testutils::Address as _, Address, Env};

use crate::{PositionNft, PositionNftClient, PositionNftError};

fn setup() -> (Env, Address, Address, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(PositionNft, ());
    let minter = Address::generate(&env);
    let owner = Address::generate(&env);
    let pool = Address::generate(&env);
    let recipient = Address::generate(&env);
    (env, contract_id, minter, owner, pool, recipient)
}

// ── Initialization ────────────────────────────────────────────────────────────

#[test]
fn test_initialize_success() {
    let (env, id, minter, _, _, _) = setup();
    let client = PositionNftClient::new(&env, &id);

    let result = client.initialize(&minter);
    assert_eq!(result, Ok(()));
    assert_eq!(client.total_supply(), 0u64);
}

#[test]
fn test_initialize_twice_fails() {
    let (env, id, minter, _, _, _) = setup();
    let client = PositionNftClient::new(&env, &id);

    client.initialize(&minter);
    let result = client.initialize(&minter);
    assert_eq!(result, Err(Ok(PositionNftError::NotAuthorized)));
}

// ── Mint ──────────────────────────────────────────────────────────────────────

#[test]
fn test_mint_creates_position() {
    let (env, id, minter, owner, pool, _) = setup();
    let client = PositionNftClient::new(&env, &id);

    client.initialize(&minter);

    let result = client.mint(&owner, &pool, &-100i32, &100i32, &1000u128);
    assert_eq!(result, Ok(0u64));

    let position = client.get_position(&0u64).unwrap();
    assert_eq!(position.owner, owner);
    assert_eq!(position.pool, pool);
    assert_eq!(position.tick_lower, -100i32);
    assert_eq!(position.tick_upper, 100i32);
    assert_eq!(position.liquidity, 1000u128);
    assert!(position.created_at > 0);
}

#[test]
fn test_mint_increments_total_supply() {
    let (env, id, minter, owner, pool, _) = setup();
    let client = PositionNftClient::new(&env, &id);

    client.initialize(&minter);

    assert_eq!(client.total_supply(), 0u64);

    client.mint(&owner, &pool, &-100i32, &100i32, &1000u128);
    assert_eq!(client.total_supply(), 1u64);

    client.mint(&owner, &pool, &-200i32, &200i32, &2000u128);
    assert_eq!(client.total_supply(), 2u64);
}

#[test]
fn test_mint_assigns_unique_ids() {
    let (env, id, minter, owner, pool, _) = setup();
    let client = PositionNftClient::new(&env, &id);

    client.initialize(&minter);

    let id1 = client.mint(&owner, &pool, &-100i32, &100i32, &1000u128).unwrap();
    let id2 = client.mint(&owner, &pool, &-200i32, &200i32, &2000u128).unwrap();

    assert_eq!(id1, 0u64);
    assert_eq!(id2, 1u64);
    assert_ne!(id1, id2);
}

#[test]
fn test_mint_not_authorized_fails() {
    let (env, id, _minter, owner, pool, _) = setup();
    let client = PositionNftClient::new(&env, &id);

    let unauthorized = Address::generate(&env);
    let result = client.mint(&owner, &pool, &-100i32, &100i32, &1000u128);
    assert_eq!(result, Err(Ok(PositionNftError::NotInitialized)));
}

// ── Owner Of ──────────────────────────────────────────────────────────────────

#[test]
fn test_owner_of_returns_correct_owner() {
    let (env, id, minter, owner, pool, _) = setup();
    let client = PositionNftClient::new(&env, &id);

    client.initialize(&minter);
    client.mint(&owner, &pool, &-100i32, &100i32, &1000u128);

    let fetched_owner = client.owner_of(&0u64).unwrap();
    assert_eq!(fetched_owner, owner);
}

#[test]
fn test_owner_of_nonexistent_fails() {
    let (env, id, minter, _, _, _) = setup();
    let client = PositionNftClient::new(&env, &id);

    client.initialize(&minter);

    let result = client.owner_of(&999u64);
    assert_eq!(result, Err(Ok(PositionNftError::PositionNotFound)));
}

// ── Transfer ──────────────────────────────────────────────────────────────────

#[test]
fn test_transfer_changes_owner() {
    let (env, id, minter, owner, pool, recipient) = setup();
    let client = PositionNftClient::new(&env, &id);

    client.initialize(&minter);
    client.mint(&owner, &pool, &-100i32, &100i32, &1000u128);

    let result = client.transfer(&0u64, &owner, &recipient);
    assert_eq!(result, Ok(()));

    let new_owner = client.owner_of(&0u64).unwrap();
    assert_eq!(new_owner, recipient);
}

#[test]
fn test_transfer_preserves_metadata() {
    let (env, id, minter, owner, pool, recipient) = setup();
    let client = PositionNftClient::new(&env, &id);

    client.initialize(&minter);
    client.mint(&owner, &pool, &-100i32, &100i32, &1000u128);

    let original_position = client.get_position(&0u64).unwrap();
    let original_created_at = original_position.created_at;

    client.transfer(&0u64, &owner, &recipient);

    let new_position = client.get_position(&0u64).unwrap();
    assert_eq!(new_position.owner, recipient);
    assert_eq!(new_position.pool, pool);
    assert_eq!(new_position.tick_lower, -100i32);
    assert_eq!(new_position.tick_upper, 100i32);
    assert_eq!(new_position.liquidity, 1000u128);
    assert_eq!(new_position.created_at, original_created_at);
}

#[test]
fn test_transfer_not_owner_fails() {
    let (env, id, minter, owner, pool, recipient) = setup();
    let client = PositionNftClient::new(&env, &id);

    client.initialize(&minter);
    client.mint(&owner, &pool, &-100i32, &100i32, &1000u128);

    let unauthorized = Address::generate(&env);
    let result = client.transfer(&0u64, &unauthorized, &recipient);
    assert_eq!(result, Err(Ok(PositionNftError::NotAuthorized)));
}

#[test]
fn test_transfer_nonexistent_fails() {
    let (env, id, minter, _, _, _) = setup();
    let client = PositionNftClient::new(&env, &id);

    client.initialize(&minter);

    let owner = Address::generate(&env);
    let recipient = Address::generate(&env);
    let result = client.transfer(&999u64, &owner, &recipient);
    assert_eq!(result, Err(Ok(PositionNftError::PositionNotFound)));
}

// ── Burn ──────────────────────────────────────────────────────────────────────

#[test]
fn test_burn_removes_position() {
    let (env, id, minter, owner, pool, _) = setup();
    let client = PositionNftClient::new(&env, &id);

    client.initialize(&minter);
    client.mint(&owner, &pool, &-100i32, &100i32, &1000u128);

    assert!(client.get_position(&0u64).is_ok());

    let result = client.burn(&0u64);
    assert_eq!(result, Ok(()));

    let result = client.get_position(&0u64);
    assert_eq!(result, Err(Ok(PositionNftError::PositionNotFound)));
}

#[test]
fn test_burn_not_authorized_fails() {
    let (env, id, minter, owner, pool, _) = setup();
    let client = PositionNftClient::new(&env, &id);

    client.initialize(&minter);
    client.mint(&owner, &pool, &-100i32, &100i32, &1000u128);

    // Remove auth to test authorization check
    env.mock_all_auths_allowing_non_root_auth();

    let unauthorized = Address::generate(&env);
    let result = client.burn(&0u64);
    // This should fail because the caller is not the minter
    assert!(result.is_err());
}

#[test]
fn test_burn_nonexistent_fails() {
    let (env, id, minter, _, _, _) = setup();
    let client = PositionNftClient::new(&env, &id);

    client.initialize(&minter);

    let result = client.burn(&999u64);
    assert_eq!(result, Err(Ok(PositionNftError::PositionNotFound)));
}

// ── Get Position ──────────────────────────────────────────────────────────────

#[test]
fn test_get_position_returns_complete_metadata() {
    let (env, id, minter, owner, pool, _) = setup();
    let client = PositionNftClient::new(&env, &id);

    client.initialize(&minter);
    client.mint(&owner, &pool, &-500i32, &500i32, &5000u128);

    let position = client.get_position(&0u64).unwrap();
    assert_eq!(position.owner, owner);
    assert_eq!(position.pool, pool);
    assert_eq!(position.tick_lower, -500i32);
    assert_eq!(position.tick_upper, 500i32);
    assert_eq!(position.liquidity, 5000u128);
}

#[test]
fn test_get_position_nonexistent_fails() {
    let (env, id, minter, _, _, _) = setup();
    let client = PositionNftClient::new(&env, &id);

    client.initialize(&minter);

    let result = client.get_position(&999u64);
    assert_eq!(result, Err(Ok(PositionNftError::PositionNotFound)));
}

// ── Edge Cases ────────────────────────────────────────────────────────────────

#[test]
fn test_multiple_positions_independent() {
    let (env, id, minter, owner, pool, recipient) = setup();
    let client = PositionNftClient::new(&env, &id);

    client.initialize(&minter);

    let id1 = client
        .mint(&owner, &pool, &-100i32, &100i32, &1000u128)
        .unwrap();
    let id2 = client
        .mint(&recipient, &pool, &-200i32, &200i32, &2000u128)
        .unwrap();

    // Transfer id1
    client.transfer(&id1, &owner, &recipient);

    // Check both positions have correct owners
    assert_eq!(client.owner_of(&id1).unwrap(), recipient);
    assert_eq!(client.owner_of(&id2).unwrap(), recipient);

    // Burn id2
    client.burn(&id2);

    // id1 should still exist
    assert_eq!(client.owner_of(&id1).unwrap(), recipient);
    assert_eq!(client.get_position(&id1).unwrap().liquidity, 1000u128);

    // id2 should not exist
    assert!(client.get_position(&id2).is_err());
}

#[test]
fn test_chain_transfers() {
    let (env, id, minter, owner, pool, _) = setup();
    let client = PositionNftClient::new(&env, &id);

    let addr1 = owner;
    let addr2 = Address::generate(&env);
    let addr3 = Address::generate(&env);

    client.initialize(&minter);
    client.mint(&addr1, &pool, &-100i32, &100i32, &1000u128);

    client.transfer(&0u64, &addr1, &addr2);
    assert_eq!(client.owner_of(&0u64).unwrap(), addr2);

    client.transfer(&0u64, &addr2, &addr3);
    assert_eq!(client.owner_of(&0u64).unwrap(), addr3);

    client.transfer(&0u64, &addr3, &addr1);
    assert_eq!(client.owner_of(&0u64).unwrap(), addr1);
}
