# Cell Seal: Glossary

Every term has exactly one meaning across all documents, the contracts, the SDK and the events.
The **Where it appears** column names the contract function, event, type or SDK call that carries
the term, so an integrator can go from the word to the code. A dash means the term is a concept
with no identifier of its own.

Cell Seal ships no end-user interface. The wording a tokenization platform shows its own users is
the platform's decision. This document fixes the technical meaning only.

## 1. Product and parties

| Term | Meaning | Where it appears |
|---|---|---|
| **Cell Seal** | The settlement layer that makes the amount of a fund transaction unreadable on the public ledger. Sold as a contract set plus an SDK | - |
| **Rail** | The part of Cell Seal a platform runs: wrapper, settlement, policy, verifier, auditor registry, fund registry, and the SDK. Contains no fund business logic | Contracts `seal-token`, `settlement`, `fund-policy`, `seal-verifier`, `seal-auditor`, `fund-registry`. Package `@cell-seal/sdk` |
| **Tokenization platform** | A third-party fund management platform: holds the fund token, the mint and burn rights, KYC, the order lifecycle and its own apps. The customer of Cell Seal: integrates the contracts and the SDK into its own system | [`integration.md`](integration.md) |
| **Integration** | The work a platform does to put its funds on the rail: deploy the contract set, wrap the fund token, route sealed legs through `settlement`, rebuild the register from decrypted data | [`integration.md`](integration.md), [`onboarding.md`](onboarding.md) |
| **Sandbox** | A reference tokenization platform with Cell Seal already integrated: a backend, a fund console and an investor app on top of the SDK. It exists so a platform can run one full cycle on the rail before integrating, and so the reference integration has a running form. Not a product, not operated for real funds | [`design/README.md`](design/README.md), [`backend.md`](backend.md), [`frontend.md`](frontend.md) |
| **Sandbox backend** | The backend of the sandbox: order store, event archive and jobs, all amount-blind. A worked example of the host services, not the product | [`backend.md`](backend.md) |
| **Host services** | The three services the SDK needs from whoever runs it: Stellar RPC, a permanent event archive, an order store for ciphertext. A platform provides them in its own infrastructure | [`sdk.md`](sdk.md) section 2.1 |
| **Fund console** | The fund-side app of the sandbox: create a fund with a sample token, manage investors, process orders, publish NAV, close windows, rebuild the register, grant disclosure | Sandbox |
| **Investor app** | The investor-side app of the sandbox: browse funds, subscribe, redeem, portfolio | Sandbox |
| **Case 1 / Case 2** | The two ways a fund joins the rail: the fund has no token yet and the token is created together with the wrapper / the fund already has a token with public holders | [`onboarding.md`](onboarding.md) |
| **Issuer** | The legal entity that issues the fund's shares. Owns the token contract and the issuer account on the platform. A customer of the platform | `issuer_admin` in `fund-registry.create_fund`. Co-signer of `settlement.close_window` |
| **Fund ops** | The people who run the fund day to day on the issuer side, acting through roles granted by the issuer admin | `FundRole::{Ops, Compliance, NavPublisher}` in `fund-registry` |
| **Transfer agent (TA)** | The party that keeps the fund's register. One of the key holders | Key holder |
| **Investor** | A party that passed KYC and was approved by the issuer, allowed to hold shares. Same as *holder* when speaking about on-chain state | `InvestorEntry`, `fund-policy.set_investor` |
| **Auditor / regulator** | A party granted the right to read amounts within a scope and for a period | Disclosure request |

## 2. Fund

| Term | Meaning | Where it appears |
|---|---|---|
| **Fund** | An open-ended fund priced by NAV. One fund maps to exactly one share token and one share wrapper | `fund_id`, `FundInfo` in `fund-registry` |
| **Share** | The unit of ownership of the fund. One share = one unit of the share token | - |
| **NAV** | Net asset value per share, published by the issuer every dealing day. **Always public** | `fund-registry.publish_nav`, `nav(fund_id, dealing_day)` |
| **Dealing day** | The day the fund prices and processes the orders received before cut-off | `dealing_day`. `FundTerms.dealing` |
| **Cut-off** | The deadline for orders to be included in a dealing day. Orders arriving later fall into the next period | `FundTerms.cutoff_secs_before` |
| **NAV strike** | The moment the NAV of a dealing day is fixed and published. Every pending order is priced from it | `ops.nav.publish`, `ops.nav.priceOrders` |
| **Forward pricing** | An order is placed before the price is known. The price is the NAV of the next dealing day. The reason every order has two legs separated in time | - |
| **Subscription** | The investor pays money to receive shares | `seal.orders.createSubscription` |
| **Redemption** | The investor returns shares to receive money | `seal.orders.createRedemption` |
| **Secondary transfer** | Transfer of shares between two investors, without the issuer | `settlement.settle_pair`, `seal.pair.*` |
| **Fee** | Subscription or redemption fee, deducted when the payout is computed | `FundTerms.sub_fee_bps`, `red_fee_bps` |
| **Register** | The record of who holds how many shares. With Cell Seal the register is rebuilt from decrypted data, not read from public balances | `ops.register.build` |

## 3. Orders and settlement

| Term | Meaning | Where it appears |
|---|---|---|
| **Order** | A subscription, redemption or secondary transfer request. Always has two legs | Backend order record |
| **Order reference** | The human-readable code of an order (`SB-0410`, `RD-0412`), used in bank transfer descriptions and in the platform's apps. **Never carries an amount** | `reference` returned by `seal.orders.create*` |
| **`order_id`** | The 32-byte identifier of an order, `SHA-256(fund_id \|\| investor \|\| client_nonce)`, generated on the client. The only part of an order that appears on chain | `BytesN<32>` in every `settlement` function and event |
| **Leg** | One half of an order. Each leg has its own status | `Leg`, `LegKind::{Direct, Spender}` |
| **Share leg** | The leg that moves shares. Always on-chain, always sealed | `LegRole::Share` |
| **Cash leg** | The leg that moves money. Off-chain (fiat) or on-chain sealed (stablecoin) | `LegRole::Cash` |
| **Leg status** | `pending` then `delivered` then `confirmed`. Changes only when the chain has confirmed, or when fund ops attest for an off-chain leg | Backend state machine, [`backend.md`](backend.md) |
| **Attest** | Fund ops confirm an off-chain event: fiat has arrived, or money has been paid | `ops.orders.attest(orderId, "cash_received" \| "cash_paid")` |
| **DoP** | *Delivery on Payment.* The two legs settle separately. The second leg is triggered only when the first is `confirmed`. Default for a fiat cash leg | `settlement.submit_leg` |
| **DvP** | *Delivery versus Payment.* Both legs in one transaction, all-or-nothing. Each side decrypts and checks the other side's amount before signing. The cash leg must be a sealed stablecoin | `settlement.settle_dvp` |
| **Strike** | The moment the NAV of a dealing day is published and every pending order is *priced*. Nothing changes hands | `ops.nav.priceOrders` |
| **Settlement** | When the settlement window closes: the net is wrapped or unwrapped, subscription shares are delivered, DvP legs are executed. The issuer mints or burns the public part on the platform's rail, before or after | `settlement.close_window`, then `submit_leg` and `settle_dvp` |
| **Settlement window** | A fixed period (for example weekly) at the end of which the issuer brings the supply change to the public ledger. Runs even when there are no orders | `WindowState`, `FundTerms.window_period_secs`, `next_close` |
| **Net** | Total subscriptions minus total redemptions of one settlement window. The only number that becomes public | `net: i128` in `close_window`. `WindowClosed.net` |
| **Cover** | How well the net of a window hides the individual orders inside it. Low when the window has too few orders or one order makes up almost all of it | `cover` in the result of `ops.window.plan` |

## 4. Privacy

| Term | Meaning | Where it appears |
|---|---|---|
| **Sealed** | An amount that sits on chain in unreadable form. Opposite: *public* | - |
| **Public** | Readable by anyone from the chain or published by the fund: addresses, timestamps, transaction type, NAV, total supply, the net of a window | - |
| **Confidentiality** | Hiding *how much*, not *who*. This is what Cell Seal does | - |
| **Anonymity** | Hiding *who*. Cell Seal does **not** do this. Holder addresses are always public | - |
| **Commitment** | An on-chain value that stands for an amount without revealing it. It can be added and subtracted, so the chain can check conservation | `confidential_balance()` on the wrapper |
| **Proof** | A zero-knowledge proof attached to a sealed transaction. Proves the transaction is valid without revealing the amount | `seal-verifier`. SDK `prover/` |
| **Conservation** | A proven property: total in = total out, no share is created or destroyed | Transfer circuit |
| **Range** | A proven property: the amount and the balance after the transaction are not negative | Transfer circuit |
| **Amount-blind** | A property every host service and the sandbox backend must have: it only stores and forwards ciphertext and has no key to read any amount | [`backend.md`](backend.md) section 1 |
| **Order secret** | The secret part of an order (amount, bank details), sealed in a sealed box to the fund and to the investor | `OrderSecret`, SDK `box/` |
| **Opening** | The pair `(v, r)` that opens a commitment. Whoever has the opening can check `v*G + r*H` against the on-chain commitment | SDK `wallet/`. `ops.register.verifyOpening` |
| **Leak** | Any path by which a sealed amount can be inferred from public data | L-n, [`security.md`](security.md) section 3 |

## 5. Keys and disclosure

| Term | Meaning | Where it appears |
|---|---|---|
| **Stellar key** | The user's wallet key. Signs transactions | Wallet adapter passed to `SealClient.connect` |
| **Privacy key** | The investor's key pair inside a wrapper: a spending key to build proofs, a viewing key (derived from the spending key) to decrypt their own balance and amounts. **Derived deterministically from one signature of the Stellar wallet**: no separate seed phrase, never sent to Cell Seal, registered on chain once and not rotatable | `sk`, `vk` in SDK `keys/`. `seal.token(wrapper).register` |
| **Viewing key** | The fund's key to decrypt every amount in the fund. It is the primitive's *auditor key*: one key per `auditor_id`, and every investor of the fund is forced to use that `auditor_id`. Distinct from the key the issuer uses day to day as a party to an order | `auditor_key` in `create_fund`. `seal-auditor` |
| **Ops root** | 32 random bytes per fund, the root of the privacy key of the settlement account and of the fund's box key. Handed to fund ops through an envelope | `ops_root`, [`keys.md`](keys.md) |
| **Box key** | An X25519 key for encrypting off-chain data (order secret, envelope). Each user and each fund has one | SDK `box/`. `box_pk` |
| **Envelope** | A secret (ops root, viewing key share) sealed to the recipient's box key and stored in the backend | SDK `box/` |
| **Key share** | One part of the viewing key, held by one key holder | SDK `threshold/` |
| **Key holder** | A party holding a key share: typically issuer, transfer agent, regulator | `seal.keyHolder(fundId)` |
| **Key epoch** | One generation of the viewing key. `rotate_key` opens a new epoch. The old epoch is still needed to read its history | `seal-auditor.rotate_key` |
| **k-of-n** | `k` of the `n` key holders must take part to open the viewing key. Cell Seal is never one of the `n` | `k_t` in [`cryptography.md`](cryptography.md) |
| **Disclosure** | Opening amounts to a party, within a defined scope and period | `seal.disclosure.open` |
| **Disclosure request** | A request to open: who reads, what, which period, when it expires. Needs `k` approvals | `holder.requests.list`, `holder.requests.approve` |
| **Unlock log** | The log of every use of the viewing key | Backend, [`backend.md`](backend.md) |

## 6. On-chain

| Term | Meaning |
|---|---|
| **Share token** / **fund token** | The platform's SEP-41 token representing the fund's shares. The platform's issuer mints and burns it. Cell Seal has no rights on it. Its supply is public |
| **Wrapper** | The `seal-token` contract that holds the share token and issues sealed balances 1:1 against it. Every sealed transfer happens inside it |
| **Share wrapper / cash wrapper** | The wrapper of the share token / of a stablecoin. DvP needs both |
| **Wrap / unwrap** | Moving tokens into / out of the wrapper (`deposit` / `withdraw`). **The amount of a wrap or unwrap is public**, which is why a redemption never unwraps per order |
| **Confidential transfer** | A transfer inside the wrapper: public addresses, sealed amount |
| **Settlement account** | The issuer's account inside the wrapper: receives shares when investors redeem, holds shares awaiting delivery for subscriptions. At the end of a window the net is wrapped into or unwrapped out of this account. The issuer funds or retires the public part on the platform's rail. `fund-registry.is_settlement_account` |
| **Receiving / spendable balance** | The two compartments of an account's balance inside the wrapper. Newly received shares sit in *receiving*. Only *spendable* can be transferred out |
| **Merge** | Folding receiving into spendable. No proof needed, no amount revealed. The SDK does it automatically before building an order |
| **Spender / allowance** | Delegating to another address the right to move part of the sealed shares. The delegated part is split off from spendable. Used for orders awaiting NAV (`set_spender`, `revoke_spender`) |
| **`settlement`** | The Cell Seal contract that every sealed transfer must go through: attaches the `order_id`, executes DvP, closes the settlement window |
| **Hook** | The wrapper's extension point, run before every operation (`SealHooks`). Cell Seal uses it to enforce the anti-leak rules |
| **Policy** | The rules checked before every confidential transfer: whitelist, KYC, freeze. `fund-policy.is_authorized` |
| **Whitelist** | The list of addresses approved by the issuer. Addresses outside the list cannot receive shares |
| **Freeze** | The issuer blocks an address from sending and receiving |
| **Position limit** | The maximum share of total supply one investor may hold. The first version checks it at order intake from numbers the issuer can already read. A check on sealed numbers needs a proof |
| **Clawback** | The issuer takes shares back from a frozen address without the holder's signature, under a court order or sanction. **The amount of a clawback is public** |
| **Indexer** / **event archive** | The service that reads on-chain events, keeps them permanently and serves them to wallets. Never sees a plaintext amount. The platform runs one. The sandbox `indexer` is the reference implementation |

## 7. Pairs that are easy to confuse

| Do not confuse | With | Difference |
|---|---|---|
| Rail | Sandbox | The rail is the contracts, the SDK and the onboarding procedure, what a platform integrates. The sandbox is one reference platform built on the rail, for trying it out |
| Host services | Sandbox backend | The host services are an interface the SDK needs (RPC, event archive, order store). The sandbox backend is one implementation of it |
| Settlement window | Dealing day | The dealing day is when orders are *priced*. The settlement window is when *supply changes publicly*. One window can contain several dealing days |
| Privacy key | Viewing key | The privacy key reads the numbers of *one investor*. The viewing key reads the numbers of *the whole fund* and needs k-of-n |
| Viewing key | The issuer's day-to-day key | The issuer is a party to every subscription and redemption, so it reads amounts with its own privacy key. The viewing key is needed only for audit, for the regulator and for secondary transfers |
| Sealed | Off-chain | Sealed still sits on chain and is checked by the chain. Off-chain means the chain knows nothing |
| DoP | "Not safe" | DoP only carries counterparty risk between an investor and a licensed issuer, a risk the fund industry already accepts |
| Register | Share token balance | After wrapping, the public balance only shows that the wrapper holds everything. The register is what is rebuilt from decrypted data |
| Sealed supply | Total supply | `supply_after` is `balance(token, wrapper)`, the wrapped part. In Case 2 the total supply also contains the public part that is not yet wrapped |
