# Cell Seal: Security

> Threat model, leak catalogue, invariants, dependencies that are not yet assured, and the
> assurance plan. This document does not replace an audit. It is the input to one.

## 1. What must be protected

| # | Asset | If lost |
|---|---|---|
| A1 | **Confidentiality of amounts**: the size of every order, the balance of every investor | Exactly what the product exists to prevent: front-running, repriced raises, copycat runs |
| A2 | **Integrity of shares**: no share is created, destroyed or moved to the wrong owner | Direct financial loss, legal liability for the issuer |
| A3 | **The fund's and the regulator's ability to read** | The fund breaches its record-keeping and reporting obligations |
| A4 | **The investor's ability to spend**: openings, keys, event history | The assets still exist but cannot be used |
| A5 | **Metadata**: who invested in which fund, and when | Commercially sensitive information, even though it is not an amount |

## 2. Adversaries

| Adversary | Capability | Wants |
|---|---|---|
| **Chain observer** | Reads everything public, forever, with analysis tooling | A1: infer sizes from public data |
| **Malicious investor** | One valid account, can build arbitrary proofs | A2: spend more than the balance. A1: read other people's numbers |
| **Backend attacker** | Full read and write on the backend that hosts the order store and the event archive: the sandbox backend, or the platform's | A1, A5. Tamper with order state |
| **Cell Seal insider** | As above on the sandbox, plus release rights on the SDK package and the contract wasm | A1. Plant malicious code in a build |
| **Malicious or compromised fund ops** | The `signer` role of one fund | A2: deliver the wrong shares. A1 for that fund |
| **Malicious key holder** | One key share | A1, when colluding up to the threshold |
| **Upstream third party** | A bug in the circuits, the verifier, or the primitive | A1, A2 |

Out of scope: an issuer that deliberately defrauds its own investors through off-chain channels
(a bank transfer that never arrives, a wrong NAV). Cell Seal provides evidence for a dispute. It
cannot prevent that behaviour.

## 3. Leak catalogue: where an amount can leak

Each leak is a way to infer a sealed amount from public data or from data the backend can see.

| # | Leak | Mechanism | Blocked where |
|---|---|---|---|
| L1 | An order carries a plaintext amount | The platform's event or API has an amount field | An order carries only `order_id`. The amount is a sealed box. `settlement` has no amount parameter other than `net` |
| L2 | Cash leg paid in a public stablecoin | `payout / NAV = shares` | The on-chain cash leg only goes through the cash wrapper |
| L3 | An investor unwraps shares directly | An unwrap has a public amount | `on_withdraw` of the share wrapper allows only the settlement account |
| L4 | An unwrap right after a large order | The timing matches the order | `close_window` refuses to run before `next_close` |
| L5 | A fee paid as a separate transfer | `fee / rate = size` | The fee is deducted inside `result`. There is no separate fee transfer |
| L6 | A window with too few orders | `net` is about one order | The *Privacy* indicator before closing. Longer windows per fund configuration. Windows run even when empty |
| L7 | A deposit straight to an investor | `deposit(from, to, amount)` has a public amount and a public recipient | `on_deposit`: `from == to` |
| L8 | An `auditor_id` outside the fund | The fund is blind to that investor, and a stranger can read | `on_register` enforces the fund's `auditor_id` |
| L9 | A stablecoin top-up that can be tied to an order | A deposit into the cash wrapper has a public amount. Topping up exactly the amount to invest and investing immediately gives it away | The cash wrapper is shared across the platform, so a deposit does not point at a fund. The SDK warns on a top-up equal to a pending order (`E_*` codes do not block it) and the platform's app recommends round amounts, topped up ahead of time, away from the moment the order is placed. This is a mitigation, not an elimination |
| L10 | The number of legs in a period | The number of transfers to an address is public | Accepted. Addresses and transaction counts are outside the confidentiality scope by design |
| L11 | Shares delivered at settlement equal `net` exactly when there is a single subscription | A special case of L6 | As L6 |
| L12 | Amounts in URLs, logs, analytics, error reports | Leak through web infrastructure | [`frontend.md`](frontend.md) section 8. End-to-end tests scan the network |
| L13 | Ciphertext size | A sealed box is longer for a larger number | `OrderSecret` pads `amount` to a fixed length before encryption |
| L14 | Proving time | In theory depends on the witness | The circuits have a fixed constraint count. No measurable difference by amount. Recorded, not addressed |
| L15 | Case 2: new shares minted publicly to an investor who then self-wraps | Amount and address are public | Rules for the platform in [`integration.md`](integration.md) section 5. The indexer warns when a self-wrap `Deposit` follows right after a public `transfer` from the issuer to the same address |
| L16 | The issuer funds the settlement account with more than `net` | The public surplus reveals that the next period has subscriptions | The SDK refuses to close when the public balance of the settlement account differs from `net` (`E_NET_NOT_FUNDED` below, `E_NET_OVERFUNDED` above). The indexer reconciles T1c |
| L17 | Bank transfer reference | The bank sees the amount, which is inherent | Out of scope. The reference contains nothing but the order code |
| L18 | A stablecoin unwrap that can be tied to a payout | A `withdraw` from the cash wrapper has a public amount. Unwrapping exactly the payout right after a redemption settles gives `payout / NAV = shares` | The mirror of L9. The cash wrapper is shared across the platform. The SDK warns when the unwrap amount equals a payout received in the last window. The platform's app recommends keeping stablecoin sealed and unwrapping round amounts later. Mitigation, not elimination |

## 4. Invariants

On-chain invariants (T1-T2, C1-C6) are in [`contracts.md`](contracts.md) section 9. System-wide
invariants:

| # | Invariant | Checked by |
|---|---|---|
| S1 | No backend process, sandbox or platform host service, ever holds a plaintext amount, a private key, or a root | Schema and API review. Log-scanning test. No numeric column with monetary meaning other than public data |
| S2 | On-chain leg state changes only on final events recorded by the event archive | No endpoint writes on-chain leg state. Integration tests |
| S3 | Off-chain leg state changes only on an attestation signed by a `fund_ops:signer` | The backend checks the signature and the role at signing time |
| S4 | The browser main thread never receives `sk`, `vk`, a root, or a key share | SDK module boundary. Lint blocks the import |
| S5 | The SDK builds no proof when an opening diverges from the on-chain commitment, or when the verification key diverges from the pinned one | Unit and integration tests |
| S6 | Every use of the viewing key has a disclosure request approved up to the threshold and a line in the unlock log | The key-holder client checks the request itself. The `audit_log` hash chain |
| S7 | The viewing key `k` never exists in one place, except in the clawback ceremony | DKG design. Operating procedure |

## 5. Analysis by adversary

| Adversary | Scenario | Outcome |
|---|---|---|
| Observer | Watches every event of a fund for a year | Sees: who the investors are, how often each address transacts, NAV, weekly supply. Does not see: the size of any order, anyone's balance. Residual risk: L6, L9, L10 |
| Malicious investor | A proof that overspends, a negative amount, a replayed proof | The circuits enforce range and conservation. `C_spend` changes after every spend so an old proof is void. `(order_id, role)` settles only once |
| Malicious investor | Does not send the opening after a secondary transfer | The register drifts until the next reconciliation with the viewing key. The fund's client flags the transfer. A fund may freeze the receiving account until the opening arrives ([`cryptography.md`](cryptography.md) section 7) |
| Backend attacker | Reads the whole database | Gets metadata (A5) and ciphertext. Does not get A1 |
| Backend attacker | Sets an order's `status` to `settled` | The UI shows the wrong state, but no asset moves. Reconciliation with the chain detects it. Cannot forge an attestation without a fund ops wallet key |
| Backend attacker | Swaps the fund's `box_pk` for their own key | The client checks the wallet signature on `box_pk` before encrypting. For the fund, `box_pk` is also anchored in `FundInfo` in `fund-registry` |
| Backend attacker | Serves a forged or incomplete event stream to a wallet | The SDK checks the rebuilt opening against the on-chain commitment. On divergence it refuses. Worst case is denial of service |
| Insider | Plants key-stealing code in the web application build | **The largest remaining risk of the web model.** Mitigations: reproducible builds, published hashes, the platform building the SDK from source, strict CSP. Not fully eliminated without a hardware wallet that supports Grumpkin keys |
| Malicious fund ops | Delivers shares to the wrong person or in the wrong amount | A transaction needs both `ops_root` and the settlement account's signature (multisig). The recipient must be whitelisted. Everything is in `audit_log` and on chain. Detectable, not fully preventable |
| Malicious fund ops | Pulls an investor's allowance without paying | `on_spender_transfer` forces the path through `settle_dvp`, which requires both legs. Paying the *wrong amount* is still possible: the investor checks afterwards and has evidence |
| Key holder | One party reads on its own | Below threshold |
| Key holder | Enough parties collude to reach the threshold | They read everything in that key epoch. This is the chosen trust model. Shares must sit with independent legal entities |

## 6. Dependencies not yet assured

| Dependency | Status (2026-09-20) | Impact | Action |
|---|---|---|---|
| Verifier `rs-soroban-ultrahonk` | README states it is not audited | A soundness bug allows forged proofs: A2 is lost | Condition for lifting the pilot cap |
| Non-zk proof flavour | The recipe does not enable `--zk`. The verifier implements only the non-zk flavour | A proof may leak information about the witness: affects A1 | Raised with upstream. Condition for lifting the pilot cap |
| Confidential Token module | Developer preview. One audit round done, report not published. Not in any stable release | The surface may change. Unknown bugs | Pin the commit. Track the `v0.9.0` branch. Rerun the compatibility tests on every upgrade |
| Freighter and SEP-0053 | Message signing support is uneven across wallets | The signer root cannot be derived | The SDK checks determinism at enrolment and falls back to a raw root ([`keys.md`](keys.md) section 2.2) |
| Stellar RPC | Limited event retention window | If the indexer falls behind, events are lost for good | Alerting, backfill path |
| Stablecoin as a SAC | Its issuer can freeze or claw back the cash wrapper address | Affects every holder in the wrapper | State it clearly to the fund. Choose a stablecoin with a clear policy |

## 7. Assurance plan

| Stage | Activity |
|---|---|
| Foundation | The upstream test vectors pass. Real proofs on the `local` network in CI. Schema and log scanning tests for S1 |
| Settlement complete | Fuzz `settlement`: leg shapes, repeated `order_id`, call order, early `close_window`. Property tests for T1, T2, C1-C6. End-to-end network scan for L12 |
| Before mainnet | Independent cryptographic review of R1-R6 ([`cryptography.md`](cryptography.md) section 10). External audit of `settlement`, `fund-policy`, `fund-registry` and `SealHooks`, for example through the Soroban Audit Bank. Rehearsal of the clawback ceremony and of an `ops_root` compromise on testnet |
| Mainnet pilot | Per-fund supply cap in `settlement`. The cap is lifted when: the verifier is audited, the zk-flavour question has an answer, and the Cell Seal contracts are audited |

## 8. Incident response

| Incident | First hour | Afterwards |
|---|---|---|
| Suspected soundness bug in the verifier or a circuit | Issuer admin freezes the affected accounts. Stop `close_window` | Reconcile T1, T2. Coordinate with upstream. Notify the funds |
| A fund's `ops_root` is compromised | Rotate the settlement account's signers if they are also suspected | Create a new settlement account, move the balance, update the registry, notify investors |
| Backend compromised (the sandbox's, or a platform's host services) | Shut down `gateway`, keep `indexer` running. Rotate `JWT_SECRET` | Forensics. Restore from backup. Reconcile order state against the chain and attestations. Notify about leaked metadata |
| Web build tampered with | Pull the build, restore the hash-verified one | Treat every key used during that window as compromised. Handle per [`keys.md`](keys.md) section 7 |
| Indexer lost events | Stop accepting new orders with an on-chain leg | Backfill from an archive node. Check event-chain continuity per contract |
| On-chain verification key changed outside the plan | The SDK blocks proving by itself (`E_VK_MISMATCH`) | Investigate the platform admin key. The timelock buys reaction time |

## 9. Test strategy

| Layer | Tooling | Runs when |
|---|---|---|
| Contracts: unit, property, fuzz | `cargo test`, `proptest`, `cargo-fuzz` | Every commit |
| Contracts: integration with real proofs | `local` network in a container | Every pull request |
| SDK: vectors, property, compatibility | Vitest | Every commit |
| Sandbox backend: API, state machine | `cargo test` with a real Postgres | Every commit |
| End to end, on the sandbox | Playwright on the `local` network | Every pull request into the main branch |
| Amount leaks | Schema, log and network scans | Every pull request |
| Load | Window-close script with 50 orders | Before every release |
| Rehearsals | Clawback, `ops_root` compromise, indexer losing events | Before mainnet |

Mock verifiers are forbidden at every layer from integration upwards: a green test with a fake
verifier says nothing about the real system.
