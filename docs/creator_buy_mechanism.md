# Creator Buy Mechanism for Custom Token Launch

## Overview

This document describes the technical requirements and logic for implementing a special buy mechanism for token creators in the N-Dollar DeFi ecosystem. The mechanism allows the creator of a new token to purchase 10% of the total supply at a fixed price immediately after token creation, with strict rules for timing, price, and token lockup.

---

## Requirements

1. **Creator Buy Opportunity:**

   - After a new token is created, the creator has a one-time opportunity to purchase exactly 10% of the total token supply at a fixed price of **0.00005 N-Dollar per token**.
   - This opportunity is available only once and only to the creator.

2. **No Second Chance:**

   - If the creator declines or ignores this opportunity, it is permanently forfeited. No second attempt is allowed.

3. **Price Change:**

   - After the creator's action (buy or skip), the price for all subsequent buyers increases to **0.0002 N-Dollar per token** (4x higher).

4. **Token Lockup:**

   - If the creator purchases the 10%, these tokens are locked (escrowed) for **1 year** and cannot be transferred or spent until the lockup period expires.

5. **Exclusivity:**
   - While the creator's opportunity is pending, no one else can buy tokens from the bonding curve.

---

## State Management

- **BondingCurve Account:**

  - `creator: Pubkey` — address of the token creator.
  - `creator_buy_status: enum { Pending, Claimed, Skipped }` — current state of the creator buy opportunity.
  - `creator_locked_until: i64` — UNIX timestamp until which the creator's tokens are locked (if purchased).
  - `initial_supply: u64` — total supply of the token.

- **Escrow Account:**
  - PDA (Program Derived Address) holding the creator's purchased tokens until the lockup expires.

---

## Instructions

### 1. `creator_buy_tokens`

- Callable only by the creator and only if `creator_buy_status == Pending`.
- Allows the creator to purchase exactly 10% of the total supply at 0.00005 N-Dollar per token.
- Transfers the required N-Dollar amount from the creator to the treasury.
- Mints or transfers 10% of the tokens to the escrow account (PDA).
- Sets `creator_buy_status = Claimed` and `creator_locked_until = now + 1 year`.
- Increases the bonding curve price to 0.0002 N-Dollar per token for all future buyers.

### 2. `skip_creator_buy`

- Callable only by the creator and only if `creator_buy_status == Pending`.
- Sets `creator_buy_status = Skipped`.
- Increases the bonding curve price to 0.0002 N-Dollar per token for all future buyers.

### 3. `unlock_creator_tokens`

- Callable by the creator after the lockup period (`now > creator_locked_until`).
- Transfers the locked tokens from the escrow account to the creator's wallet.

### 4. `buy_tokens` (for all users)

- If `creator_buy_status == Pending`, only the creator can buy (via `creator_buy_tokens`).
- If `creator_buy_status == Claimed` or `Skipped`, anyone can buy at the new price (0.0002 N-Dollar per token and up).

---

## Escrow Mechanism

- The creator's purchased tokens are held in a PDA escrow account with seeds `[b"creator_lock", mint, creator]`.
- The escrow account only allows withdrawal after the lockup period via the `unlock_creator_tokens` instruction.

---

## Security & Edge Cases

- The creator cannot buy more or less than 10% of the total supply.
- The creator cannot buy after skipping or after the opportunity expires.
- No one else can buy tokens while the creator's opportunity is pending.
- The lockup is enforced at the smart contract level; tokens cannot be transferred from escrow until the period ends.

---

## User Flow

1. User creates a new token (genesis program).
2. Bonding curve is initialized with `creator_buy_status = Pending`.
3. Creator is prompted to either buy 10% at the fixed price or skip.
4. If bought, tokens are locked for 1 year; if skipped, price increases immediately.
5. After 1 year, creator can unlock and transfer their tokens.
6. All other users can buy tokens at the higher price after the creator's decision.

---

## Implementation Notes

- All state and logic must be enforced on-chain.
- The frontend should prompt the creator to act immediately after token creation.
- Optionally, a timeout can be implemented off-chain to call `skip_creator_buy` if the creator does not act within a certain period.
