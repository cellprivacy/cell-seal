# Cell Seal: Onboarding a Fund

> How a fund on a tokenization platform is brought onto the Cell Seal rail. Two cases: the fund
> has **no token yet** (Case 1) and the fund **already has a token** with holders (Case 2). The
> contracts, the SDK and the window procedure are the same in both. What differs is the token, where
> the public supply comes from, and whether the fund ends up fully sealed or in a hybrid state.
> Ownership boundary, token conditions and supply rules: [`integration.md`](integration.md).
> Business flows after onboarding: [`flows.md`](flows.md). Contract signatures:
> [`contracts.md`](contracts.md).

## 1. Which case applies

| The fund today | Case | Note |
|---|---|---|
| No token on Stellar | **Case 1** | The platform creates the token during onboarding |
| A token exists but was never distributed (supply 0, or all supply still in the issuing account) | **Case 1**, on the existing contract | Run the token checklist of Case 2 (section 4.2) first. Then the procedure of Case 1 applies, because no holder balance was ever public |
| A token exists and holders have balances | **Case 2** | Hybrid state until every holder has wrapped, possibly forever |
| The token is a SEP-8 regulated asset | Blocked | A contract-invoked `transfer` does not pass the SEP-8 approval server. The platform moves transfer control for the sealed part to `fund-policy` first. The public part keeps SEP-8 |
| The token lives on another chain | Out of scope | Cell Seal is not multi-chain. The platform issues on Stellar first, then Case 1 |

Case 1 is Case 2 with an initial supply of 0. There is no contract difference. The difference is
operational.

## 2. Prerequisites for both cases

| Item | Who | Detail |
|---|---|---|
| Platform contract set | Platform, once | `settlement`, `fund-policy`, `seal-verifier` (verification keys registered), `seal-auditor`, `fund-registry`. Deployed with the platform admin key (K6). One set serves every fund on the platform |
| Event archive (indexer) | Platform | Running and caught up before the first wrapper is deployed. Events older than the RPC retention window are otherwise lost. Interface: [`sdk.md`](sdk.md) section 2.1. Reference: [`backend.md`](backend.md) section 6 |
| Cash wrappers | Platform, once per stablecoin | One `seal-token` wrapper per stablecoin the platform accepts for DvP. Listed per fund in `FundTerms.cash_wrappers`. Not needed for a fiat-only fund |
| Issuer admin account | Fund | A multisig Stellar account. Issuer of the fund token on the platform's rail. Admin of the wrapper. Co-signer of `close_window` |
| Settlement account | Fund ops | A Stellar account of the fund whose signers are the fund ops. Its privacy key is derived from an `ops_root` generated in the browser of the person who creates the fund ([`keys.md`](keys.md) section 3) |
| Key holders | Issuer, transfer agent, regulator | Each with a wallet and a box key. Default threshold 2 of 3. A fund may start at 2 of 2 and rerun the ceremony when the regulator joins |
| Investor list | Platform compliance | Account, status, KYC expiry, ready to be written into `fund-policy` |
| Fund terms | Fund | Dealing frequency, cut-off, settlement window period, fees, minimum investment, payment methods, cash wrappers, document hash ([`contracts.md`](contracts.md) section 7 `FundTerms`) |
| Window period | Fund, with the platform | Long enough that the net of a window does not equal one order. A fund with few orders needs a longer window. See section 6 |
| Testnet dry run | Platform | The whole procedure below, with the same parameters, on testnet before mainnet. See section 7 |

## 3. Case 1: the fund has no token yet

### 3.1 What is specific to Case 1

- Token and wrapper are created in the same onboarding session. Supply starts at 0.
- No holder balance is ever public. From the first transaction, the public cap table shows one
  holder: the wrapper.
- Every new share enters the wrapper through the settlement account, as the `net` of a window.
  The issuer mints only into the settlement account, only before `close_window`, only `net`.
- There is no migration and no second rail. Every investor uses the sealed rail from day one.

### 3.2 Choosing the token

The platform chooses the token contract. Cell Seal takes its address. Either form works:

| | Soroban token (for example OpenZeppelin fungible) | Classic asset through its SAC |
|---|---|---|
| Mint and burn | Admin or minter role held by the issuer admin | Payment from the issuing account mints. Payment back to it burns |
| Decimals | Set to 7 at deployment | Always 7 |
| Transfer control | None needed. `fund-policy` controls the sealed part | `AUTH_REQUIRED` is possible but adds an authorization step for the wrapper and the settlement account. Recommended: leave it off and let `fund-policy` control the sealed part |
| Clawback | Do not enable | Leave `AUTH_CLAWBACK_ENABLED` off. The wrapper's pool must never be clawed back |
| SEP-8 | Not applicable | Do not use |
| Public holders | Never, by procedure | Never, by procedure |

In the sandbox, `fund-registry.create_fund` deploys a sample token together with the wrapper so a
platform can try the rail without preparing a token. In production the platform deploys its own
token and passes its address to `seal.platform.deployFund`.

### 3.3 Procedure

```mermaid
sequenceDiagram
    participant P as Platform
    participant T as Fund token
    participant W as Share wrapper
    participant R as fund-registry
    participant K as Key holders
    participant O as Fund ops (browser)
    P->>T: 1. Create the token, supply 0, issuer admin holds mint
    O->>O: 2. Generate ops_root, derive the settlement account's privacy key
    P->>K: 3. Viewing key ceremony (DKG), public key K
    P->>R: 4. deployFund(token, terms, issuer admin, settlement account, K)
    R->>W: deploy the wrapper (token, verifier, auditor, policy)
    R->>R: register the auditor key, open window 1
    P->>T: 5. If AUTH_REQUIRED: authorize wrapper and settlement account
    O->>W: 6. register(settlement account, auditor_id) in share wrapper and cash wrappers
    P->>P: 7. Write investors into fund-policy
    Note over P,W: 8. Investors register in the wrapper; auditor_id enforced by on_register
```

| Step | Who signs | Call | Result |
|---|---|---|---|
| 1 | Issuer admin | Token deployment, or classic asset creation | A SEP-41 token with supply 0 and 7 decimals |
| 2 | Fund ops | In the browser, no transaction | `ops_root` (32 random bytes), sealed into an envelope for each `signer`. Never sent in clear |
| 3 | Each key holder | `holder.dkg.join(ceremonyId)` | Public key `K` of the viewing key. Each holder keeps a share `k_j`. Nobody ever holds `k` |
| 4 | Issuer admin | `seal.platform.deployFund(params)`, which calls `fund-registry.create_fund` | `fund_id`, wrapper address, `auditor_id`, window 1 opened in `settlement` |
| 5 | Issuer admin | Classic: `set_trust_line_flags` on the wrapper and the settlement account. Only when `AUTH_REQUIRED` is set | The wrapper can hold the pool. The settlement account can receive the net |
| 6 | Settlement account | `ops.accounts.register({ wrappers })` | The settlement account exists inside the share wrapper and each cash wrapper, with the fund's `auditor_id` |
| 7 | Fund ops with the `compliance` role | `ops.investors.set(account, Approved, kycExpiry)` per investor | `fund-policy` returns `true` for those accounts on this fund's wrapper |
| 8 | Each investor | `seal.token(wrapper).register({ auditorId })` from the platform's app | A sealed account with zero balance. The hook rejects any other `auditor_id` (error `4000`) |

The wrapper address is known after step 4, so authorization in step 5 comes after deployment. The
wrapper holds nothing until the first `close_window`, so the order is safe.

### 3.4 The first window

Nothing has been wrapped yet. The first window is almost always a subscription-only window:

1. Investors place subscriptions. Bank transfers are attested by fund ops (`cash_received`).
   Stablecoin subscriptions set an allowance on the cash wrapper.
2. Strike: NAV is published. Each order is priced. `shares = cash / NAV - fee`, in the browser.
3. `ops.window.plan()` returns `net = sum of shares subscribed`, which is positive.
4. The issuer mints `net` public tokens into the settlement account.
5. `ops.window.close(plan)`: `close_window(net)` deposits `net` into the wrapper, then one
   `submit_leg` per subscription delivers sealed shares from the settlement account to each
   investor.
6. The platform's indexer reconciles: `balance(token, wrapper) = supply_after = net` (T1a), and one mint of
   exactly `net` preceded the close (T1c).

What the market sees in the first window:

| Public | Not public |
|---|---|
| One mint of `net` to the settlement account | Which investor subscribed how much |
| One `deposit` of `net` from the settlement account into the wrapper | Any individual balance |
| N sealed transfers from the settlement account to N investor addresses | The amount of any of them |

If the first window holds one subscription, `net` equals that order (L11). Funds usually announce
their launch size anyway, so this is accepted for the launch window. For every later window the
cover rules in section 6 apply.

### 3.5 Steady state

After the first window, Case 1 is the steady state described in [`flows.md`](flows.md): every
subscription and redemption is a sealed transfer inside the wrapper, and the only public figure per
window is `net`. The public cap table permanently shows one holder, the wrapper, whose balance
changes once per window.

## 4. Case 2: the fund already has a token

### 4.1 What is specific to Case 2

- The token's history is public and stays public. Every balance a holder had before wrapping is
  known. Cell Seal seals what happens **after** the wrap, not before.
- Holders are not forced to wrap. The fund runs a public rail and a sealed rail side by side.
- The public supply that funds a window with `net > 0` comes from the issuing account or a
  treasury by `transfer`, not necessarily by `mint`. The reconciliation reads either.
- The token was not designed for the wrapper. The checklist in section 4.2 decides whether it can be
  wrapped as is.

### 4.2 Token checklist

Go/no-go before anything is deployed. The full reasoning is in [`integration.md`](integration.md)
section 3.

| Check | Pass | Fail, and what to do |
|---|---|---|
| Implements the SEP-41 interface (`transfer`, `balance`) | SAC of a classic asset, or a Soroban token | Non-standard token: the platform issues a new token and swaps 1:1, which turns this into Case 1 |
| 7 decimals | Classic assets always. Soroban tokens usually | The SDK converts at the boundary. The conversion is recorded in the fund configuration |
| `AUTH_REQUIRED` | Off, or the issuer can authorize two more addresses | Issuer must authorize the wrapper and the settlement account before the first `deposit` |
| `AUTH_CLAWBACK_ENABLED` | Off, or the issuer commits in writing not to claw back the wrapper | Without a commitment the pool backing every sealed balance can be destroyed |
| SEP-8 approval server | Not used | **Blocker.** See section 1 |
| Mint and burn | Held by the issuer on the platform | Cell Seal never takes them |
| Who holds balances today | Known list, with sizes | Needed for the migration plan in section 4.3 and for the cover of the first windows |

### 4.3 Procedure

Steps 2 to 4 and 6 to 8 are identical to Case 1 (section 3.3). The additions are the token
authorization, the treasury and the holder migration.

```mermaid
sequenceDiagram
    participant P as Platform
    participant T as Fund token (existing)
    participant W as Share wrapper
    participant H as Existing holder
    participant S as Settlement account
    P->>T: 1. Checklist (4.2): flags, decimals, SEP-8, mint rights
    Note over P: Steps 2-4 as in Case 1: ops_root, DKG, deployFund
    P->>T: 5. If AUTH_REQUIRED: authorize wrapper and settlement account
    P->>T: 5b. If AUTH_CLAWBACK_ENABLED: commitment, or clear the flag
    Note over P: Steps 6-8 as in Case 1: register settlement account, sync policy, investors register
    P->>H: 9. Announce the sealed rail; "Move your shares to sealed" in the platform's app
    H->>W: 9a. register(auditor_id)
    H->>W: 9b. deposit(from = to = self, whole balance)
    Note over H,W: public amount, but it was already public
    P->>S: 10. Treasury: a public balance or the mint right, to fund net > 0 windows
```

| Step | Who signs | Call | Result |
|---|---|---|---|
| 1 | Platform | Off-chain review plus a read of the asset flags | Go / no-go, and the list of current holders |
| 5 | Issuer admin | `set_trust_line_flags` for the wrapper and the settlement account, classic assets only | Both can hold the token |
| 5b | Issuer admin | Clear `AUTH_CLAWBACK_ENABLED` if no holder needs it, else a written commitment | The wrapper's pool is safe from clawback |
| 9 | Platform | A screen in the platform's own app, not in Cell Seal | Holders learn that the sealed rail exists and what wrapping means |
| 9a | Holder | `seal.token(wrapper).register({ auditorId })` | A sealed account with the fund's `auditor_id`. Requires the holder to be `Approved` in `fund-policy` first, because the policy is checked on `to` |
| 9b | Holder | `seal.token(wrapper).wrap(amount)`, which calls `deposit(from = to = holder, amount)` | The holder's public balance moves into the wrapper's pool. The holder receives the same amount sealed, in the receiving balance. `merge` before spending |
| 10 | Issuer admin | None, or a transfer into a treasury account | A source of public tokens for `net > 0` windows: either the mint right (as in Case 1) or an existing balance in the issuing account |

The SDK checks after the first `deposit` that `balance(wrapper)` moved by exactly the amount. A
mismatch means the token blocks transfers to the wrapper (flags, SEP-8) and onboarding stops
there.

### 4.4 Running two rails

| | Public rail (platform) | Sealed rail (Cell Seal) |
|---|---|---|
| Holders | Those who have not wrapped | Those who called `deposit` |
| Subscription | Issuer transfers or mints public tokens to the holder | `settlement account to investor` inside the wrapper, at `close_window` |
| Redemption | Holder transfers public tokens to the issuer | `investor to settlement account` inside the wrapper, any time. Unwrapped as part of `net` |
| Register | Public balances of the token | Rebuilt from decrypted data |
| NAV | The same NAV applies to both | |
| Fund size | Public supply outside the wrapper, plus `balance(wrapper)` | Inside the wrapper the split per holder is sealed |

Rules the platform must follow so the public rail does not leak the sealed one:

| Do | Do not | Why |
|---|---|---|
| Deliver new shares to a sealed investor from the settlement account, inside the wrapper | Mint or transfer public tokens to an investor and let them wrap | A public amount tied to an address (L15). The platform's indexer flags a self-wrap that follows a public transfer from the issuer to the same address |
| Let a holder wrap an existing balance, ideally the whole balance in one `deposit` | Wrap in several pieces sized like upcoming orders | Each `deposit` is public. A pattern of deposits is a pattern of orders |
| Ask a public holder who wants to subscribe more to wrap first, then subscribe sealed | Run a public subscription and a sealed subscription for the same holder in the same window | The public one is visible and dates the sealed one |
| Keep `net` for the sealed part only | Add public-rail figures into `net` | The contract reconciles `net` against the wrapper only (T1a) |
| Count public subscriptions and redemptions on the platform side | Try to move a sealed holder back to the public rail | Investors cannot `withdraw` (L3). Leaving the sealed rail is a redemption |

A holder who wrapped cannot be moved back to the public rail. The only exit is a redemption. This
is deliberate: a `withdraw` by an investor would publish an amount.

### 4.5 The first windows in Case 2

The first windows mix three kinds of movement on the wrapper:

| Movement | Public? | Counted in `net`? |
|---|---|---|
| Self-wrap `deposit` by an existing holder | Yes, per holder, per amount. The amount was already public as that holder's balance | No. It changes `balance(wrapper)` outside `close_window`. T1b allows exactly this |
| Sealed subscription or redemption | Only the fact of a transfer | Yes |
| `close_window` deposit or withdraw of `net` | Yes, one figure | It is `net` |

Example with a negative net. Three holders wrap 1,000,000 shares each in week 1 (three public
deposits). In week 2 one of them redeems 400,000 shares sealed, inside the wrapper. At the close,
`net = -400,000`: `close_window` withdraws 400,000 public tokens to the settlement account, and the
issuer recalls them to the issuing account or burns them. The market sees three deposits of
1,000,000 and one withdrawal of 400,000. It does not see which of the three redeemed.

With a positive net the issuer funds the settlement account by `transfer` from the issuing account
or treasury (or by mint, where the token allows), then `close_window` deposits it. The platform's indexer
accepts either a `mint` or a `transfer` of exactly `net` as the T1c funding event.

### 4.6 Ending the hybrid state

The platform cannot force a wrap: `deposit` requires the holder's own authorization. Options, from
soft to hard:

| Option | Effect |
|---|---|
| Keep both rails indefinitely | Default. Public holders keep public balances. The register is the union of public balances and the sealed register |
| Sealed-only for new subscriptions after a date | Public holders may keep what they have. Any new shares go through the sealed rail. The share of sealed supply rises over time |
| Public holders redeem and re-subscribe sealed | Both legs are public amounts. Equivalent, for that holder, to a self-wrap. No privacy gain, only administrative cost |

Clawback of a public holder into the wrapper is not a migration path: it is a compliance action
and publishes the amount.

## 5. Side by side

| | Case 1: no token yet | Case 2: existing token |
|---|---|---|
| Token | Created with the wrapper. 7 decimals, no clawback, no SEP-8 | Checked against section 4.2. May need authorization and a clawback commitment |
| Public history | None | Everything before the wrap date |
| Holder action | Register only | Register, then self-wrap the balance |
| First window | Subscriptions only. `net > 0` funded by mint | Self-wrap deposits plus sealed orders. `net` may be negative. Funded by transfer or mint |
| Supply funding | Mint into the settlement account | Transfer from the issuing account or treasury, or mint |
| End state | Fully sealed | Hybrid, possibly permanent |
| Platform app changes | Orders without amounts, register from decrypted data, window procedure | The same, plus a "Move your shares to sealed" screen and rules for two rails |
| Contracts | Identical | Identical |
| Reconciliation | T1a, T1c | T1a, T1b (self-wraps), T1c |

## 6. Leaks specific to onboarding

The full catalogue is in [`security.md`](security.md) section 3. The ones that appear during onboarding:

| Leak | Case | Mitigation |
|---|---|---|
| L7: `deposit` straight to an investor | Both | `on_deposit` requires `from == to`. Only self-wraps and the settlement account's net deposit exist |
| L8: an investor registers with a foreign `auditor_id` | Both | `on_register` enforces the fund's `auditor_id` |
| L11: a window with one order | Case 1 launch window, any thin window | Accepted for the launch. Afterwards the window period is set so that several orders share a window, and the plan shows the cover before closing |
| L15: public mint to an investor who then wraps | Case 2 | Platform rule (section 4.4). Alert from the platform's indexer |
| L16: the settlement account is funded with more than `net` | Both | The SDK refuses to close when the settlement account's public balance differs from `net`. T1c |
| Self-wrap sized and timed like an order | Case 2 | Guidance in section 4.4: wrap the whole balance once, not in order-sized pieces |

## 7. Verification before go-live

| Check | How | Pass condition |
|---|---|---|
| Token accepts the wrapper | First `deposit` on testnet with the real token parameters | `balance(wrapper)` changes by exactly the amount |
| Settlement account registered | `confidential_balance(settlement account)` on the share wrapper and each cash wrapper | Returns an account with the fund's `auditor_id` |
| Policy sync | `fund-policy.investor(fund_id, account)` for a sample of investors | Matches the platform's KYC records |
| Viewing key | A disclosure request on the testnet fund, opened by `k_t` holders | Decrypted amounts match the commitments on chain. One line in the unlock log |
| Full cycle | One window on testnet with at least three subscriptions and two redemptions | Only `net` appears as a public figure in `WindowClosed`. The register rebuilt in the browser equals `balance(wrapper)` |
| Early close rejected | `close_window` before `next_close` | Reverts with `4104 WindowNotDue` |
| Unfunded close rejected | `close_window(net > 0)` without funding | Reverts with `4108 NetNotFunded` |
| Case 2 only: self-wrap | One holder wraps on testnet | T1b: the wrapper balance change matches the `Deposit` event. The holder's sealed balance equals the deposit after `merge` |
| Indexer | `GET /status` | Caught up. Cursor above the RPC retention floor |
