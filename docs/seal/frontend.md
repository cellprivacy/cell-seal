# Cell Seal: Frontend

> Repo `cell-seal-web`. pnpm workspace. Vite, React 19, TypeScript, Tailwind 4, TanStack Query,
> Freighter. Design: [`design/README.md`](design/README.md). Every cryptographic operation goes
> through [`sdk.md`](sdk.md). The frontend does no cryptography of its own. The two apps are the
> apps of the **sandbox**, the reference tokenization platform. They are not the product
> ([`integration.md`](integration.md) section 1). A platform integrates the SDK into its own apps.
> This document shows one way to do it. Section 4 to section 8 are the rules any such app must
> keep.

## 1. Workspace

```
cell-seal-web/
|-- packages/
|   |-- sdk/            @cell-seal/sdk
|   `-- ui/             @cell-seal/ui: color tokens, shared components
`-- apps/
    |-- fund-console/   11 screens plus the key holder page
    `-- investor-app/   7 screens
```

The two apps are built and deployed separately, on two different origins. The investor app is the
public surface. The fund console can sit behind the private network of whoever runs the sandbox.

## 2. Design system

Tokens come from the variables in `design/cell-seal.pen`:

| Token | Value | Used for |
|---|---|---|
| `--background` | `#0A0B0D` | Page background |
| `--card` | `#121418` | Cards, tables, rails |
| `--secondary` | `#1A1D23` | Inputs, selected rows, pills |
| `--border` / `--border-strong` | `#252932` / `#343943` | |
| `--foreground` / `--muted-foreground` / `--dim` | `#EDEFF2` / `#858C99` / `#5A606C` | |
| `--primary` / `--primary-subtle` | `#C8F94E` / `#2E3A16` | Primary action, selected nav item, private figures |
| `--success` / `--warning` / `--destructive` / `--info` | `#4ADE80` / `#FBBF24` / `#F87171` / `#60A5FA` | Statuses |
| Font | Inter. JetBrains Mono with `tabular-nums` for every amount, address and order reference | |

Text on a `--primary` background is always `#0A0B0D`, never white.

Components of `@cell-seal/ui`: `Shell`, `SetupShell`, `PageHead`, `Pill`, `Button`, `Field`,
`Segmented`, `DataTable`, `StepList`, `SummaryCard`, `Kpis`, `PrivateTag`, `ProofProgress`,
`SealedGate`.

## 3. Routes

### 3.1 Fund console

| Route | Screen | Reads | Writes |
|---|---|---|---|
| `/setup/fund` | 01 Fund details | none | `fund-registry.create_fund`, then `POST /funds` |
| `/setup/keys` | 02 Who can read amounts | `GET /funds/{id}/dkg` | `ops.dkg.start`, invite key holders. Each holder runs `holder.dkg.join` on `/keys` |
| `/investors` | 03 Investors | `GET /funds/{id}/investors` | `fund-policy.set_investor`, then `PATCH /investors/{a}` |
| `/` | 04 Dashboard | orders, calendar, current window | none |
| `/orders` | 05 Orders | `GET /funds/{id}/orders` | none |
| `/orders/:id` | 06 Subscription, 08 Redemption | `GET /orders/{id}` | `ops.orders.attest`, hold, release |
| `/nav` | 07 NAV | calendar, `awaiting_price` orders | `ops.nav.publish`, then `ops.nav.priceOrders` |
| `/settlement` | 09 Settlement | current window and history | `ops.window.plan`, then `ops.window.close` |
| `/register` | 10 Register | `ops.register.build()` | export file |
| `/audit` | 11 Audit access | disclosure requests, unlock log | `ops.disclosure.request`, remind, `ops.disclosure.cancel` |
| `/keys` | Key holder | requests awaiting my approval | `holder.requests.approve` |

### 3.2 Investor app

| Route | Screen | Reads | Writes |
|---|---|---|---|
| `/welcome` | 01 Set up account | none | connect wallet, `seal.unlock()`, `POST /me/box-key` |
| `/funds` | 02 Funds | `GET /funds` | `POST /funds/{id}/access-requests` |
| `/funds/:id` | 03 Fund detail | `GET /funds/{id}`, `token.balance()` | none |
| `/funds/:id/invest` | 04 Invest | calendar, cash balance | `orders.createSubscription` |
| `/funds/:id/redeem` | 07 Redeem | `token.balance()` | `orders.createRedemption` |
| `/orders/:id` | 05 Order | `GET /orders/{id}` | `orders.cancel` |
| `/portfolio` | 06 Portfolio | `token.balance()` per fund, `GET /orders` | none |

`/funds` and `/funds/:id` work without login and with the wallet locked: they show public data
only. The *Your position* card appears only after `unlock`.

## 4. Two kinds of data, two ways of managing them

| | Public data and metadata | Decrypted data |
|---|---|---|
| Examples | Fund list, NAV, order status, calendar | Amounts, balances, shares, payouts, register |
| Source | REST API | `@cell-seal/sdk`, after `unlock` |
| Cache | TanStack Query, `staleTime` per kind | Memory only. `gcTime: 0`. Never persisted |
| When the wallet is locked | Still shown | Replaced by `SealedGate`: a blurred box with an *Unlock* button |
| In URLs, logs, analytics, error messages | Allowed | Never |

`SealedGate` wraps every place that displays a private figure. It guarantees one thing: there is
no render path that shows an amount without passing through the SDK's `unlocked` state.

## 5. Amounts

| Rule | Content |
|---|---|
| Type | `bigint` at scale `10^7` throughout. Never `number` for money or shares |
| Input | Decimal string to `bigint` through the `@cell-seal/ui` parser. Reject more than 7 decimal places |
| Display | `Intl.NumberFormat` on the integer and fractional parts separately. Always monospace, `tabular-nums` |
| Computing `shares`, `payout` | In the SDK, in integers, with a single rounding rule: round down so the investor receives less, the remainder stays with the fund. This rule must match the future price-linking circuit |
| Estimates before strike | Labelled *estimated*, computed at the latest NAV |

## 6. Asynchronous state of an on-chain action

Every button that creates a transaction goes through the same state chain, shown by
`ProofProgress`:

```
idle -> syncing -> merging? -> proving -> awaiting_signature -> submitting -> confirming -> done | failed
```

| State | What the user sees | Notes |
|---|---|---|
| `syncing` | "Checking your balance" | The SDK syncs events and checks the openings |
| `merging` | "Preparing your balance" | Only when spendable is insufficient. A separate transaction that needs a signature |
| `proving` | "Preparing your order, a few seconds" with a progress bar | Duration estimated from `seal.capabilities.threads` |
| `awaiting_signature` | "Confirm in your wallet" | |
| `submitting`, `confirming` | "Sending", "Waiting for the network" | `done` only when the **indexer** has seen the event, not when the RPC accepted the transaction |
| `failed` | Message by SDK error code ([`sdk.md`](sdk.md) section 7) | Always with a clear next action |

Leaving the page between `proving` and `confirming` is blocked by a confirmation dialog. After a
reload, the SDK restores the wallet lock state and the UI shows the pending work again.

## 7. Per screen: behaviour to get right

### Fund console

| Screen | Behaviour |
|---|---|
| 01 Fund details | One click runs the chain: generate `ops_root`, derive the settlement account's privacy key, `seal.platform.deployFund`, `ops.accounts.register` for the share wrapper and the cash wrappers, create the envelope for the creator. A failure at any step can be resumed from that step |
| 02 Who can read amounts | Each key holder opens `/keys` on their own device to join the DKG. *Ready* means a valid share commitment has been submitted. Allows continuing with 2 of 2 and re-running the DKG when the third party is ready |
| 03 Investors | *Approve* calls `fund-policy.set_investor` on chain first, then writes to the backend. *Holding* comes from the register on the client |
| 04 Dashboard | *To do* is built from order states and the calendar: set price, confirm money, pay out, approve investor, close window. Each row links straight to the screen that handles it |
| 06 Subscription order | *Confirm money received* signs an attestation with the wallet. No on-chain transaction at this step |
| 07 NAV | *Publish price* calls `fund-registry.publish_nav`. After confirmation the SDK computes the `result` for each order and sends `result_ct`. The preview table is computed on the client |
| 08 Redemption order | *Mark as paid* signs a `cash_paid` attestation. The investor's bank details are decrypted from the `OrderSecret` |
| 09 Settlement | *Close window* is enabled only when `next_close` has been reached and no order in the period is still `awaiting_cash` or `held`. Shows the plan before running: `net`, order count, the *Privacy* level. Progress is shown per transaction in the chain |
| 10 Register | Built entirely on the client from decrypted legs and verified openings. Rows with an unverified secondary transfer are flagged |
| 11 Audit access | Create a request: reader, scope, period, expiry. Approval status per key holder. *History* is the unlock log |

### Investor app

| Screen | Behaviour |
|---|---|
| 01 Set up account | Three steps: connect wallet, KYC, *Sign with wallet*. If the SDK throws `E_NONDETERMINISTIC_SIGNER`, switch to the raw root flow with its mandatory backup step. The single-point-of-failure warning is mandatory under the primitive's spec |
| 02 Funds | The button on each card follows access status: *Invest*, *Request access*, *Access requested* |
| 03 Fund detail | Every figure outside the *Your position* card is public. The *Fund size by week* chart is built from `WindowClosed` |
| 04 Invest | Choosing *EURC* shows the sealed cash balance. If there is none, lead to the top-up flow, `seal.token(cashWrapper).wrap(amount)`, with the L9 advice: round amounts, ahead of time. If a DvP order is already pending, disable this choice and say why |
| 05 Order | Wire details are decrypted from the investor's copy of the ciphertext. The *Cancel* button appears only before cut-off |
| 06 Portfolio | Total value = sum of `balance x NAV` over funds, computed on the client |
| 07 Redeem | Quick buttons 25%, 50%, All are computed on `spendable + receiving`. The SDK merges automatically if needed |

## 8. Browser-side security

| Topic | Rule |
|---|---|
| Cross-origin isolation | `Cross-Origin-Opener-Policy: same-origin`, `Cross-Origin-Embedder-Policy: require-corp`. Needed for multithreaded proving. Also blocks embedding third-party resources |
| CSP | `default-src 'self'`. `connect-src` only to the API, RPC and indexer. `script-src 'self' 'wasm-unsafe-eval'`. No inline scripts |
| Third parties | No analytics, no session recorder, no fonts or scripts from a CDN. A third-party script in the page is an amount leak path |
| Storage | `localStorage` holds UI preferences only. Encrypted openings live in IndexedDB. No key is ever stored |
| Error reporting | Self-hosted error reporter. A filter strips every numeric field and every string matching an amount format before sending |
| Auto-lock | After 15 minutes of inactivity, or when the tab has been hidden for more than 5 minutes |
| Build integrity | The bundle hash is published with every release. Whoever runs the sandbox can build from source |

## 9. Testing

| Layer | How |
|---|---|
| Components | Vitest plus Testing Library. `SealedGate` has its own test asserting that no amount is rendered while the wallet is locked |
| End to end | Playwright on the `local` network, real verifier, test wallet signing automatically. Standard scenario: create fund, approve investor, subscribe, set price, close window, redeem, pay out, register matches |
| Amount leaks | The end-to-end test captures every network request and asserts that no body, URL or header contains any amount used in the scenario |
| Devices | Measure proving time on a desktop, an ordinary laptop and a 4 GB phone before every release, and compare with [`primitives.md`](primitives.md) section 8 |
| Against the design | Compare screenshots with `design/screens/` for all 18 screens |
