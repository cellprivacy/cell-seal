# Cell Seal: Architecture

> The root document of the technical design. Every contract, service, table and package name in
> the other documents comes from here. Read [`overview.md`](overview.md), [`glossary.md`](glossary.md)
> and [`primitives.md`](primitives.md) first.

## 1. Four design principles

| # | Principle | Technical consequence |
|---|---|---|
| P1 | **Cell Seal cannot read any amount** | Every amount leaves the browser as ciphertext. Encryption, decryption and proofs happen only on the client. Any backend the SDK talks to is *amount-blind*: it stores and relays ciphertext and holds no key |
| P2 | **The chain is the source of truth** | The status of an on-chain leg changes only when the event archive sees a final event. Any database, the sandbox's or the platform's, is a read model, rebuildable from the chain and from ciphertext |
| P3 | **The rail knows nothing about fund business** | Rail-layer contracts have no notion of NAV, cut-off, dealing day. Those live in the tokenization platform's system, and in the sandbox in its backend and fund console |
| P4 | **Compose, do not rewrite** | Commitments, circuits, verifier, freeze come from OpenZeppelin's Confidential Token. Cell Seal writes only what [`primitives.md`](primitives.md) section 4 and section 5 list |

The product is the on-chain layer (section 3.1), the SDK (section 3.3, `@cell-seal/sdk`) and the
onboarding procedure ([`onboarding.md`](onboarding.md)). The backend (section 3.2) and the two apps
(section 3.4) are the **sandbox**: a reference tokenization platform with the rail integrated, run
for evaluation and read as a worked integration. Section 3 opens with the table that assigns every
component to one of the two. Ownership boundary, token requirements and who mints and burns:
[`integration.md`](integration.md).

## 2. System diagram

```mermaid
flowchart TB
    subgraph Client["Browser: the only place that sees plaintext"]
        FC["Fund console (sandbox)"]
        IA["Investor app (sandbox)"]
        KH["Key-holder client"]
        SDK["@cell-seal/sdk<br/>keys, proofs, decryption, sealed box"]
        FC --- SDK
        IA --- SDK
        KH --- SDK
    end
    subgraph Backend["Sandbox backend: amount-blind"]
        GW["gateway: REST + auth"]
        ORD["orders: state machine"]
        IDX["indexer: event archive"]
        DB[("Postgres")]
        GW --- ORD
        ORD --- DB
        IDX --- DB
    end
    subgraph Chain["Stellar, Soroban"]
        ST["fund token: the platform's"]
        SW["seal-token: share wrapper"]
        CW["seal-token: cash wrapper"]
        SET["settlement"]
        POL["fund-policy"]
        AUD["seal-auditor"]
        VER["seal-verifier"]
        REG["fund-registry"]
        SET --> SW
        SET --> CW
        SW --> POL
        SW --> AUD
        SW --> VER
        SW --> ST
    end
    SDK -- "signed transaction, with proof" --> RPC["Stellar RPC"]
    RPC --> Chain
    SDK -- "ciphertext, order metadata" --> GW
    IDX -- "getEvents" --> RPC
    SDK -- "events older than the RPC window" --> IDX
```

In an integration, the platform's own apps take the place of the two sandbox apps and the
platform's own backend takes the place of the sandbox backend. Both talk to the same SDK, and the
chain layer is identical. What the SDK needs from that backend is listed in
[`sdk.md`](sdk.md) section 2.1.

## 3. Components

| Component | Scope | Who runs it in production |
|---|---|---|
| Contracts, section 3.1 | Product | The platform, one contract set, from the audited wasm hashes |
| `@cell-seal/sdk`, section 3.3 | Product | The platform, inside its own apps and backend |
| Onboarding procedure, [`onboarding.md`](onboarding.md) | Product | The platform, once per fund |
| Event archive and order store | Host services the SDK needs | The platform, in its own infrastructure |
| Backend, section 3.2 | Sandbox, reference | Nobody for real funds. Its `indexer` is a reference event archive a platform may run as is |
| Fund console, investor app, `@cell-seal/ui`, section 3.4 | Sandbox | Nobody for real funds |

### 3.1 On-chain: repo `cell-seal-contracts` (Rust, Soroban)

| Contract | Instances | Role | Source |
|---|---|---|---|
| fund token | 1 per fund | The platform's SEP-41 token: SAC or Soroban token. The platform's issuer mints and burns. Cell Seal has no rights on it. `share-token` in the prototype is a sample token for tests and Case 1 | Platform. Sample: OpenZeppelin fungible |
| `seal-token` | 1 per fund (share wrapper) and 1 per stablecoin (cash wrapper) | Sealed balances and transfers. Cell Seal's hooks | OpenZeppelin `ConfidentialToken` trait, concrete contract written by Cell Seal |
| `seal-verifier` | 1 per platform | Verifies UltraHonk proofs, holds the verification key per `CircuitType` | Nethermind `rs-soroban-ultrahonk` + OpenZeppelin trait |
| `seal-auditor` | 1 per platform | Registry of viewing keys by `auditor_id`. One `auditor_id` per fund | `ConfidentialAuditor` trait |
| `fund-policy` | 1 per platform | `is_authorized(account, token)`: per-fund whitelist + KYC attestation | Cell Seal |
| `settlement` | 1 per platform | The single gate for every sealed transfer. Attaches the order reference. DvP. Closes the settlement window | Cell Seal |
| `fund-registry` | 1 per platform | Catalogue of funds and the public data of each | Cell Seal |

Details: [`contracts.md`](contracts.md). Requirements the fund token must meet:
[`integration.md`](integration.md) section 3.

### 3.2 Sandbox backend: repo `cell-seal` (Rust workspace: axum, sqlx, tokio, Postgres)

Part of the sandbox, not of the product. It is the backend of the reference platform and shows one
way to host the SDK: an amount-blind order store, an event archive and the jobs around them. A
platform has its own backend and provides the host services there. The one crate a platform may
take as is is `indexer`, the reference implementation of the event archive.

| Crate | Role |
|---|---|
| `gateway` | REST API, wallet-signature authentication, role-based authorization, rate limit |
| `orders` | State machine of orders and legs, dealing calendar, NAV, settlement window, attestations |
| `indexer` | Permanent archive of every event of the Cell Seal contracts. API for wallets to recover state |
| `jobs` | Supply reconciliation after every `WindowClosed` (T1a to T1c), storage TTL extension, reminders |
| `core` | Types, configuration, Stellar RPC client, error codes |
| `auth` | Challenge, session, roles |

Details: [`backend.md`](backend.md).

### 3.3 SDK: `@cell-seal/sdk` (TypeScript)

Product. Key derivation, core cryptography, proof generation in a Web Worker, wallet state,
contract clients, sealed box, threshold decryption, register reconstruction. The only code that
touches private keys and plaintext amounts. Lives in the `cell-seal-web` workspace with the sandbox
apps but is published and versioned on its own. Details: [`sdk.md`](sdk.md).

### 3.4 Sandbox apps: repo `cell-seal-web` (pnpm workspace: Vite, React, TypeScript, Tailwind)

| Package | Role |
|---|---|
| `@cell-seal/ui` | Design system of the sandbox, colour tokens from `design/` |
| `apps/fund-console` | The fund's 11 screens, plus the key-holder page |
| `apps/investor-app` | The investor's 7 screens |

Details: [`frontend.md`](frontend.md), [`design/README.md`](design/README.md).

## 4. Trust boundaries

| Zone | Sees | Never sees |
|---|---|---|
| **Investor's browser** | That investor's own balance and orders | Other investors' numbers |
| **Fund ops' browser** | Every order of the fund (through the fund's box key and the settlement account's key) | The k-of-n viewing key. Investors' keys |
| **Key-holder client** | That party's own key share. The plaintext of the approved scope | Other parties' key shares |
| **Backend** (the sandbox's, or the platform's host services) | Who placed an order, order type, time, status, NAV, addresses | **Every amount**, every private key, the settlement account's root |
| **Chain, observers** | Addresses, timestamps, order references, NAV, total supply, the window net | The amount of any order, the balance of any holder |

If the backend is compromised, the attacker gets metadata and ciphertext, not amounts. This is why
P1 is enforced by cryptography and not by access policy.

## 5. Data classification

| Class | Examples | Stored in | Form |
|---|---|---|---|
| **Public** | NAV, dealing calendar, total supply, net of closed windows, addresses, order references | Chain, `fund-registry`, DB | Plaintext |
| **Metadata** | Which investor an order belongs to, type, status, time, payment method | DB | Plaintext, role-scoped access |
| **Sealed on-chain** | Transfer amounts, balances | `seal-token` | Commitment + ciphertext for the recipient and for the auditor |
| **Sealed off-chain** | Amount of a fiat order, payout, net of the open window, bank details | DB | Sealed box to the box key of the fund and of the investor |
| **Secret** | Grumpkin `sk`, root, key share, private box key | Browser memory. Encrypted envelope in the DB | Never leaves the client in the clear |

## 6. The accounts of a fund

| Account | Type | Used for |
|---|---|---|
| **Issuer admin** | The issuer's Stellar multisig account | Issuer of the fund token on the platform: mint, burn on the platform's rail. Admin of the share wrapper: freeze, clawback, configuration. Co-signs `close_window` |
| **Settlement account** | A Stellar account of the fund, registered in the share wrapper and the cash wrappers | Receives redemption shares, delivers subscription shares, receives and pays stablecoin, acts as spender for DvP orders |
| **Fund ops** | Personal account of each staff member | Login, signing attestations. Signer of the settlement account |
| **Key holder** | Accounts of issuer, TA, regulator | Hold a share of the viewing key. Approve disclosure requests |

The settlement account has several operators, so it does not use a *signer root* (a key derived
from one specific signer's signature) but a **raw root** of 32 bytes, generated when the fund is
created and handed to each fund ops member as an encrypted envelope. See [`keys.md`](keys.md) section 3.

## 7. The two timelines of an order

The primitive forces the two moments apart, and every later document uses exactly these two words:

| Moment | Happens when | What takes place |
|---|---|---|
| **Strike** | Dealing day, after cut-off | NAV is published. Every pending order is *priced*. Nothing changes hands |
| **Settlement** | When the settlement window closes | The issuer has funded the public **net** into the settlement account. `close_window` wraps it, then the share legs of subscriptions are delivered and the DvP legs are executed |

Reason: before the window closes, the net shares of the period are not yet wrapped, so the
settlement account does not hold enough sealed shares to deliver every subscription. `settlement`
neither mints nor burns. The issuer does both on the platform's rail, before and after
`close_window` ([`integration.md`](integration.md) section 4):

```
net = sum of subscription shares - sum of redemption shares

net > 0 :  issuer funds net public into the settlement account (mint in Case 1, transfer in Case 2)
           then close_window: deposit(net) into the wrapper                      -+
net < 0 :  close_window: withdraw(|net|) to the settlement account                -+-> sealed balance = sum of subscription shares
           then issuer burns or retires |net| on the platform's rail              |
net = 0 :  no public operation                                                    -+
                                                                                  -> deliver everything to the subscriptions -> 0
```

Redemption shares reach the settlement account at any time during the period and are the source of
the netting.

## 8. Data flow of an order

Shown with the sandbox's components. In an integration the platform's app and backend play the
same two roles and the SDK calls are the same.

### 8.1 Subscription paid by bank transfer (DoP)

```mermaid
sequenceDiagram
    participant I as Investor app
    participant B as Backend
    participant F as Fund console
    participant C as Chain
    I->>I: Encrypt amount into a sealed box to the fund's box key and to its own
    I->>B: POST /orders {fund, kind=sub, pay=bank, amount_ct, order_id}
    B-->>I: Bank transfer details (encrypted for the investor), reference code
    Note over I: Investor transfers money outside the system
    F->>B: GET /orders, decrypt the amount in the browser
    F->>B: POST /orders/{id}/attest {cash_received, signature}
    Note over F,B: Strike: fund ops publish NAV
    F->>F: shares = amount / NAV - fee, computed in the browser
    Note over F,C: Settlement: issuer funds the net, closing the window wraps it
    F->>C: settlement.submit_leg(order_id, share wrapper, settlement account to investor, data)
    C-->>B: Event LegSettled(order_id) through the indexer
    B-->>I: Order = done
    I->>C: merge() before the next spend
```

### 8.2 Redemption paid out by bank transfer (DoP)

The investor builds the proof and calls `settlement.submit_leg(order_id, share wrapper, investor to
settlement account, data)` as soon as the order is placed. The settlement account is the recipient,
so the fund console decrypts the amount with its key. After strike, fund ops make the bank transfer
and attest `cash_paid`. The shares sitting in the settlement account are netted when the window
closes.

### 8.3 DvP with stablecoin

When placing the order, the side that must pay first delegates exactly the needed part to the
settlement account with `set_spender` (cash for a subscription, shares for a redemption). At
settlement, fund ops, signing as the settlement account, call `settlement.settle_dvp(order_id, legs)`: the pull leg uses
`confidential_transfer_from`, the pay leg uses `confidential_transfer`, in the same invocation,
hence atomic. Only the settlement account signs. The investor need not be online. The `seal-token`
hook forces every transfer through `settlement`, so the settlement account cannot pull funds while
skipping the other leg. Details and limits: [`cryptography.md`](cryptography.md) section 5.

### 8.4 Secondary transfer

Both investors are online. Each builds the proof for its own leg and signs one authorization entry.
one transaction calls `settlement.settle_pair(order_id, leg_a, leg_b)`. The receiving side sends
the *opening* of the leg it received to the fund (encrypted to the fund's box key) so the register
updates without the k-of-n viewing key. See [`cryptography.md`](cryptography.md) section 7.

## 9. Deployment

| Environment | Stellar network | Purpose |
|---|---|---|
| `local` | Quickstart container | Development. Real verifier, real proofs |
| `testnet` | Testnet | Integration and sandbox. Pilot with a sample fund |
| `mainnet-pilot` | Mainnet | Capped pilot. Value cap configured in `settlement` |

The product is deployed per platform: the platform deploys the contract set from the audited wasm
hashes with its own admin key, and installs the SDK from the package registry. The sandbox runs
under Docker Compose (`gateway`, `indexer`, `postgres`, the two apps). Cell Seal hosts one instance
on `testnet` for evaluation and a platform can run its own copy from source.

Circuit artifacts (ACIR, verification key) are pinned by hash in the SDK and compared with the key
registered in `seal-verifier` at startup. On a hash mismatch the SDK refuses to build proofs.

## 10. Reading order

| # | Document | Answers |
|---|---|---|
| 0 | [`integration.md`](integration.md), [`onboarding.md`](onboarding.md) | What the platform owns, what Cell Seal provides. The two onboarding cases step by step. Fund token requirements. Who mints, who burns |
| 1 | [`contracts.md`](contracts.md) | Each contract: storage, functions, events, errors, authorization, invariants |
| 2 | [`cryptography.md`](cryptography.md) | What is taken from the primitive, the DvP protocol, threshold disclosure, openings |
| 3 | [`keys.md`](keys.md) | Every key: generation, storage, recovery, rotation, loss |
| 4 | [`sdk.md`](sdk.md) | The `@cell-seal/sdk` API |
| 5 | [`backend.md`](backend.md) | Sandbox backend: services, API, data model, state machine, reference indexer |
| 6 | [`frontend.md`](frontend.md) | Sandbox apps: routes, state, amount handling |
| 7 | [`security.md`](security.md) | Threat model, invariants, leak catalogue, assurance plan, test strategy |
