# esports_wager

## Project Title
esports_wager

## Project Description
esports_wager is a peer-to-peer esports betting protocol built on Stellar's
Soroban smart-contract platform. A market creator opens a betting market that
is pinned to a specific esports match id and a set of possible outcomes (for
example `team_a`, `team_b`, `draw`). Any user can then place a wager on the
outcome they believe will win. When the match is over, the market creator (or
oracle front-end) calls `settle_market` with the winning outcome, and winning
bettors claim a proportional share of the entire pool via `claim_payout`.
Unlike a generic prediction market, the contract is esports-specific and
carries a `match_id` field so that off-chain score feeds and tournament UIs
can correlate on-chain markets with real fixtures.

## Project Vision
To make competitive gaming wagering transparent, trust-minimized, and globally
accessible. By settling the bet book on a public ledger, esports_wager
removes the need for a centralized bookmaker, makes odds and pool sizes
auditable, and lets any wallet participate directly. The long-term vision
is a permissionless layer where tournament organizers, community casters, and
players themselves can spin up provably-fair markets for any match in any
game.

## Key Features
- **Match-linked markets** — every market is bound to a `match_id` and a
  explicit list of outcomes, so off-chain score oracles can drive settlement.
- **Creator-controlled settlement** — only the market creator (acting as a
  trusted oracle, or fronted by an off-chain result service) can declare the
  winning outcome.
- **Cumulative wagers** — bettors can top up their position on the same
  outcome; their wager totals are aggregated in storage.
- **Proportional payouts** — winners split the full pool (not just the
  winning side) in proportion to their contribution to the winning side, so
  the contract behaves like a parimutuel book.
- **One-claim safety** — each bettor can claim a settled market at most once,
  preventing replay or double-pay.
- **On-chain auditability** — `wager_total` exposes the live amount staked on
  any outcome, allowing any front-end to show real-time odds.

## Contract

- **Network:** Stellar Testnet (Public)
- **Scope:** gaming dApp — see `contracts/esports_wager/src/lib.rs` for the full esports_wager business logic.
- **Functions exposed:** see `Key Features` above and the `pub fn` list in `lib.rs`.
- **Contract ID:** `<CBF2T7QY46RJUTWRKHJHXJUK6SMHUTSJTSODR26B53EIE4QA4MRFYAGE>`
- **Explorer template:** `https://stellar.expert/explorer/testnet/tx/47770ab4d5f403744a2a81cc878909cc0ed5c68498afe3e176e22d130886b632`
- **Screenshot of deployed contract on Stellar Expert:**
  `_(Screenshot of the contract page on Stellar Expert will appear here after deploy.)_`


## Future Scope
- **Real asset settlement** — wire `place_wager` and `claim_payout` to a
  Stellar asset (USDC or a custom game token) via the SAC, replacing the
  current off-chain bookkeeping amounts with on-ledger transfers.
- **Oracle integration** — accept a signed result from a decentralized oracle
  (or a quorum of community oracles) instead of trusting a single creator
  address.
- **Multi-game support** — extend the market struct with `game`, `tournament`
  and `start_time` fields and add helper getters to filter by tournament.
- **Frontend dApp** — a React/Next.js UI that lists open markets, shows live
  `wager_total` bars, and lets a Freighter wallet place bets and claim
  payouts.
- **Refund on cancellation** — a `cancel_market` path that refunds all
  bettors pro-rata if the underlying match is postponed or voided.
- **Fee model** — a small creator fee (in basis points) deducted from the
  pool at settlement to incentivize market makers on Mainnet.

## Profile

- **Name:** <!-- Fill github name -->
- **Project:** `esports_wager` (gaming)
- **Built with:** Soroban SDK 25, Rust, Stellar Testnet
