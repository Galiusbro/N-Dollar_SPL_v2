# N-Dollar DeFi Smart Contracts Suite

This repository contains a suite of Solana smart contracts (programs) for a modular DeFi ecosystem centered around the N-Dollar token, custom token creation, distribution, referral rewards, bonding curve mechanics, and a liquidity pool. All programs are written using the Anchor framework.

---

## Programs Overview

### 1. `n-dollar`

**Purpose:**
Implements the N-Dollar token and its initial liquidity pool.

**Key Features:**

- Creation of the N-Dollar token with metadata.
- Initialization of a liquidity pool for N-Dollar and SOL.
- Mints 108,000,000 N-Dollar tokens to the pool vault.

---

### 2. `liquidity-pool`

**Purpose:**
Provides a simple AMM (Automated Market Maker) for swapping between N-Dollar and SOL, and for adding/removing liquidity.

**Key Features:**

- Pool initialization with N-Dollar and SOL vaults.
- Add liquidity (N-Dollar + SOL).
- Swap SOL to N-Dollar and vice versa using a constant product formula.

---

### 3. `genesis`

**Purpose:**
Allows users to create their own tokens, with metadata, using N-Dollar as payment for rent and fees.

**Key Features:**

- Validates token parameters (name, symbol, supply, etc.).
- Swaps N-Dollar to SOL for rent via the liquidity pool.
- Creates token mint, metadata, and info accounts.
- Mints the total supply to a distributor account for further distribution.

---

### 4. `token-distributor`

**Purpose:**
Handles the distribution of newly created tokens according to a predefined allocation.

**Key Features:**

- Distributes tokens from the distributor account to:
  - Referral program treasury (10%)
  - Bonding curve (40%)
  - AI agent (50%)
- Ensures all associated token accounts are created as needed.

---

### 5. `referral-program`

**Purpose:**
Implements a simple referral reward mechanism.

**Key Features:**

- Distributes fixed rewards from the referral treasury to two referees.
- Ensures sufficient treasury balance and correct account ownership.

---

### 6. `bonding-curve`

**Purpose:**
Implements a linear bonding curve for token price discovery and liquidity.

**Key Features:**

- Initializes a bonding curve with a target supply and price range.
- Allows users to buy tokens from the curve (price increases linearly).
- Allows users to sell tokens back to the curve (price decreases linearly).
- All calculations are performed with high precision and overflow checks.

---

## Example Flow

1. **User creates a new token** via the `genesis` program, paying with N-Dollar.
2. **Token supply is minted** to a distributor account.
3. **`token-distributor` splits the supply** between the referral treasury, bonding curve, and AI agent.
4. **Users can buy/sell tokens** on the bonding curve, or swap N-Dollar/SOL in the liquidity pool.
5. **Referral rewards** are distributed via the referral program.

---

## Technologies

- [Solana](https://solana.com/)
- [Anchor](https://book.anchor-lang.com/)
- [Metaplex Token Metadata](https://docs.metaplex.com/programs/token-metadata/overview)

---

## Structure

```
programs/
  n-dollar/           # N-Dollar token and pool initialization
  liquidity-pool/     # AMM for N-Dollar/SOL swaps
  genesis/            # User token creation
  token-distributor/  # Token distribution logic
  referral-program/   # Referral rewards
  bonding-curve/      # Linear bonding curve for tokens
```

---

## Deployment

Each program is independent but designed to work together. Deploy using Anchor CLI:

```sh
anchor build
anchor deploy
```

смотри, надо поменять смарт контракт. Надо чтобы после создания пользовательского токена у создателя была возможность выкупить 10% токенов по цене 0.00005 n-dollar за токен.
Потом после, если пользователь купил токены, они фризиятся на один год.
И после этого токен становится доступным для торговли на бондинг кривой по цене 0.0002.

или пользователь отказывается от предложения и тогда токен тоже становится доступным для торгов и цена так же становится 0.0002

То есть надо дать пользователю возможность купить 10м токенов по спец цене. Возможно есть
