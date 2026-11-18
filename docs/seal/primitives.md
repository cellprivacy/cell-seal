# Cell Seal: Primitives

> A survey of OpenZeppelin's Confidential Token module for Stellar, read from the code on
> 2026-09-20. Nothing compiled, no proof generated. Everything marked **hypothesis** is inferred from
> the spec and must be checked with a prototype before it enters the design. Terms:
> [`glossary.md`](glossary.md).

## 1. Conclusion

Building Cell Seal on the Confidential Token is the right call: the hardest parts (commitment,
amount encryption for the recipient and for the auditor, six circuits, freeze, policy hook) exist
and can be used as they are. What Cell Seal needs on top falls into three groups:

| Group | Items |
|---|---|
| **Use as is** | 1:1 wrapper, confidential transfer, spender/allowance, freeze, auditor registry, the holder key model |
| **Configure and write thin contracts** | Share token contract, policy contract (whitelist + KYC), hook enforcing `auditor_id`, settlement account and settlement window |
| **Build** | DvP, k-of-n for the auditor key, position limit, sealed forced transfer, client SDK. A reference durable indexer in the sandbox |

Two risks must be faced, both outside Cell Seal's control:

1. **The verifier is unaudited and runs the non-zk flavour.** The proof recipe does not enable
   `--zk`, and the verification key README says the verifier "currently implements only the non-zk
   `ultra_flavor`". The consequence for confidentiality has not been answered upstream. This blocks
   mainnet.
2. **The module is not in any stable release**, and three branches expose different surfaces (section 2).

## 2. Where the code is

| | |
|---|---|
| Repo | `OpenZeppelin/stellar-contracts`, MIT, `packages/tokens/src/confidential/` |
| Verifier | `NethermindEth/rs-soroban-ultrahonk`, MIT, README says "has not been audited" |
| Network | Testnet. Needs protocol 26 or later. No mainnet deployment found |
| Audit | The module went through one audit round (issues L-01 to L-06, N-01 to N-11 closed July 2026). No published report found. Verifier: none |

| Branch | What it has | Use for Cell Seal |
|---|---|---|
| `main` | 6 circuits, `revoke_spender` needs a proof, **no** clawback, verifier is a trait with no body | No |
| `v0.9.0` | Adds clawback (circuit + trait), `revoke_spender` without proof, escrow blinding for the auditor, split docs | **Track this branch** |
| `feat/confidential-verifier-ultrahonk` (PR #820) | Wires Nethermind's real verifier. The SDF blog post and demo pin this branch. No clawback | Take the verifier wiring |

Proposal: pin one commit of `v0.9.0`, deploy the verifier ourselves, and treat rebasing on upstream
as a recurring work item rather than a one-off.

## 3. Use as is

| Need | What the primitive provides | Note for Cell Seal |
|---|---|---|
| Wrap the share token 1:1 | `deposit(from, to, amount)` and `withdraw(from, to, amount, data)` on any SEP-41 token | Both amounts are **public**. Exactly as [`flows.md`](flows.md) assumed |
| Sealed transfer | `confidential_transfer(from, to, data)`: public addresses, hidden amount and both balances | The proof shows conservation and range |
| Delegation | `set_spender`, `confidential_transfer_from`, `revoke_spender`: hidden allowance, escrowed out of the spendable balance | The basis of a redemption order awaiting NAV (section 5.1) |
| Freeze | `freeze` / `unfreeze` / `is_frozen`: blocks send, receive, deposit, withdraw | Access control to be added |
| Auditor registry | `register_key` / `rotate_key` / `get_key` by `auditor_id` | A single key, see section 5.2 |
| Encryption for the auditor | Every transfer encrypts the amount for the sender's auditor **and** the recipient's auditor | The auditor sees every amount and balance after each transaction |
| Selective disclosure | Specs of the `disclose_*` circuits (recipient, sender, auditor, balance <= / >=) | Spec only. The implementation lives in the demo repo, verified off-chain |

### The holder key model

This is where the survey changes the client design:

- A holder has a spending key `sk` on Grumpkin and a viewing key `vk` derived from `sk`. Both public
  keys are registered on chain once with `register`. **`sk` cannot be rotated**.
- `sk` is **derived deterministically from one signature of the Stellar wallet** (SEP-0053) over a
  message naming the contract and the account. No separate seed phrase: as long as the wallet
  exists, the privacy key exists.
- Consequence: the wallet's ed25519 key is the single point of failure, and wallets using MPC or
  threshold signing are **incompatible** because their signatures are not deterministic.
- The balance has two compartments: *receiving* (newly arrived) and *spendable*. `merge` folds
  receiving into spendable without a proof. The SDK must merge before building an order.
- The wallet keeps the opening `(v, r)` off-chain and decrypts from events. Stellar RPC keeps about
  7 days of events, so **a durable indexer is a required component**, not a convenience.
- Freighter does not support this key. Proofs are built in the browser with bb.js: about 1 second
  on desktop according to the demo. 2.7 to 6.7 seconds on a 4 GB phone. 10 to 16 seconds in an
  Android WebView.

## 4. Configure and thin contracts

| Item | What has to be done |
|---|---|
| **Share token contract** | The module is only a trait. `main` has no sample contract. Write a concrete contract: constructor, access control, wiring to verifier and auditor |
| **Policy contract** | `Policy::is_authorized(account, token) -> bool`, called for both `from` and `to` on every operation. Cell Seal writes the implementation: whitelist + KYC attestation. **Policy never sees an amount** |
| **Enforce `auditor_id`** | The holder picks `auditor_id` at `register` and cannot change it. Core only checks that the id exists. Must be blocked in the `on_register` hook, otherwise an investor picks an auditor outside the fund and the issuer is blind |
| **Settlement account** | An issuer account inside the wrapper: receives redemption shares, holds shares awaiting delivery. At the end of the window the net, and only the net, is wrapped into it or unwrapped out of it |
| **Settlement window** | No such concept in the primitive. It is Cell Seal state and schedule, not cryptography |

An operating rule follows from the signature `deposit(from, to, amount)`: the issuer **never**
deposits straight to an investor's address, because the amount would be public together with the
recipient. Always deposit into the issuer's account, then transfer sealed.

## 5. Build

### 5.1 DvP may not need a new circuit *(hypothesis)*

The primitive has no swap, no escrow, no cross-token circuit. But two of its properties open a much
shorter path than "a circuit linking two commitments":

1. Two `confidential_transfer` calls inside the same contract invocation are **atomic** on Soroban:
   if one fails, both revert.
2. The payload of every transfer already contains the amount encrypted for the **recipient**. The
   recipient can decrypt it off-chain *before* the transaction goes on chain, and the proof forces
   that ciphertext to match the commitment.

So a settlement contract is enough for DvP: each side builds the proof for its own leg, sends the
payload to the other side, the other side decrypts it to check the agreed number and only then
signs. The contract executes both legs in one call. Neither side has to trust the other, and no new
circuit is needed.

To be checked with a prototype:

- A transfer proof is bound to the sender's current balance commitment. If the balance changes
  between proof generation and execution (including through `merge`), the proof breaks. For orders
  that wait several days, the investor uses `set_spender` to escrow the needed part. The primitive's
  spec requires **the spender to be a registered account with keys**, so the spender is the fund's
  settlement account and cannot be a contract. The wrapper hook forces every allowance pull through
  the settlement contract, so the settlement account cannot pull while skipping the other leg.
- A circuit linking `cash = shares x NAV - fee` is only needed if the **chain** itself must enforce
  the price relation instead of the two sides checking it. That would need a separate verifier,
  because the core trait fixes `CircuitType`.

### 5.2 k-of-n for the auditor key

The auditor key is **a single Grumpkin scalar**, with no threshold hook. Decryption only needs
`k*R_e` per event, so off-chain threshold ECDH is feasible: each key holder computes its part on
`R_e`, the parts are combined by interpolation, and the scalar `k` never exists in one place.

What makes k-of-n workable in operations: **the issuer is a party to every subscription and
redemption**, so it reads amounts with its own viewing key, without touching the auditor key. The
auditor key is only needed for audit, for the regulator, and for secondary transfers between two
investors.

A secondary transfer changes the register without the issuer being a party. The design answers
this with the recipient sending the opening to the fund ([`cryptography.md`](cryptography.md)
section 7), so the register never needs `k` parties for routine updates. Clawback in section 5.4
needs `k_aud` *inside the circuit*, which means a controlled key-reconstruction ceremony
([`cryptography.md`](cryptography.md) section 6.4).

### 5.3 Position limit

Policy does not see amounts, so it cannot be enforced through the policy call. The primitive only
has specs for `disclose_balance_le / ge`, built by the holder, off-chain. Nothing for a ratio over
supply or for on-chain enforcement. The direction OpenZeppelin chose (issue #766) is
"auditor-assisted transfer", not yet implemented.

Proposal in two steps: the first release checks **at order intake**, off-chain, with the numbers
the issuer can already read, and rejects orders over the limit. An on-chain circuit comes later.
The order of these two steps must be stated clearly wherever a circuit was promised.

### 5.4 Clawback and forced transfer

Only on `v0.9.0`: `clawback(account, amount, destination, data, operator)`. **Public amount**,
returns the underlying token, requires the account to be frozen, and the proof needs the auditor's
secret. There is no sealed forced transfer to another account.

Proposal: the first release uses OpenZeppelin's clawback, a rare event under court order where a
public amount is acceptable. A sealed forced transfer is a new circuit, to be done later.

### 5.5 Indexer and client SDK

OpenZeppelin ships **a spec, not an SDK**. A maintainer's demo repo has a TypeScript SDK, disclosure
circuits, an indexer and a front end wired to Freighter. Use it as a reference, not as a dependency.
Cell Seal writes its own client SDK (key derivation, proof generation, decryption, merge). The SDK
needs an indexer that keeps every event permanently so wallets and the register can be recovered.
The platform runs one, and the sandbox ships a reference implementation.

## 6. Needs matrix

| Need | Primitive | Cell Seal does |
|---|---|---|
| Share token wrapped 1:1 | Yes | Write a concrete contract |
| Whitelist + KYC on every transfer | Partial, interface only | Write the policy contract. Enforce `auditor_id` |
| Atomic DvP | No | Settlement contract. A linking circuit is optional (section 5.1) |
| k-of-n auditor key | No | Off-chain threshold ECDH (section 5.2) |
| Net unwrap per window | No | Settlement account + schedule |
| Position limit | Partial, spec only, off-chain | Check at order intake. Circuit later |
| Clawback | Partial, `v0.9.0`, public amount | Use as is. Sealed forced transfer later |
| Freeze | Yes | Add access control |

## 7. What the earlier design had to change

| Place | It said | It must be |
|---|---|---|
| Investor app, Set up account screen | Save the 12 words of the privacy key | Sign a message with the wallet. The privacy key is derived from that signature |
| Glossary, *Privacy key* | A key created separately on the device | Derived deterministically from a Stellar wallet signature |
| Glossary, *Viewing key* | A separate key of the fund | It is the primitive's auditor key. The issuer's viewing key as a party is a different key |
| Flows, leak list | 6 paths | Add: deposit straight to an investor. Investor choosing an `auditor_id` outside the fund |
| Overview section 6 | "Position limit and clawback on sealed numbers" are circuits we write | The first release writes no circuit for either |

The full design of the items above: [`architecture.md`](architecture.md),
[`contracts.md`](contracts.md), [`cryptography.md`](cryptography.md).

## 8. Measured on testnet

A prototype of the wrapper, the verifier and a minimal `settlement` ran one full cycle on testnet
on 2026-10-09 with real proofs. The design above rests on these figures.

| Measurement | Result |
|---|---|
| Verifying one proof on chain | 90-97 M instructions, 0.10-0.17 XLM |
| Proving one transfer | About 1 second on a laptop |
| Two transfer proofs in one transaction | About 188 M instructions against the 400 M ceiling. They fit with room |
| `settlement.require_auth()` inside the wrapper hook | Works. A direct call to the wrapper is rejected even when the sender signed |
| Two-party authorization for `settle_pair` | Works. The settlement account signs its authorization entry with the SDK's `authorizeEntry`, the issuer signs the envelope, one transaction |
| Tool versions | nargo `1.0.0-beta.11`, bb `0.87.0`, upstream commit `5ffda91`. The six verification keys rebuild byte-identical |
| Auditor key generator | `H`, the same generator as `PVK` |
| First `close_window` | 0.40 XLM, mostly rent for new entries |
