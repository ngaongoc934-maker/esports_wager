#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Map, Symbol, Vec};

#[contract]
pub struct EsportsWager;

#[contractimpl]
impl EsportsWager {
    /// Open a new peer-to-peer esports betting market that is tied to a
    /// specific on-chain match id. The creator authenticates the call, the
    /// `match_id` is recorded for downstream consumers, and `outcomes` is the
    /// list of mutually-exclusive results that bettors can wager on (e.g.
    /// `["team_a", "team_b", "draw"]`). The market starts in the `open` state
    /// and can be settled later by the same creator.
    pub fn open_market(
        env: Env,
        creator: Address,
        market_id: Symbol,
        match_id: u64,
        outcomes: Vec<Symbol>,
    ) {
        creator.require_auth();

        if outcomes.len() < 2 {
            panic!("market needs at least 2 outcomes");
        }

        // Reject duplicate outcomes to keep settlement unambiguous.
        let len = outcomes.len();
        for i in 0..len {
            for j in (i + 1)..len {
                if outcomes.get(i).unwrap() == outcomes.get(j).unwrap() {
                    panic!("duplicate outcome");
                }
            }
        }

        let mkt_key = (symbol_short!("mkt"), market_id.clone());
        if env.storage().instance().has(&mkt_key) {
            panic!("market already exists");
        }

        env.storage().instance().set(&mkt_key, &creator);
        env.storage()
            .instance()
            .set(&(symbol_short!("match"), market_id.clone()), &match_id);
        env.storage()
            .instance()
            .set(&(symbol_short!("outcomes"), market_id.clone()), &outcomes);
        env.storage()
            .instance()
            .set(&(symbol_short!("status"), market_id.clone()), &symbol_short!("open"));
    }

    /// Place a wager of `amount` contract units on `outcome` inside `market_id`.
    /// The bettor must authenticate the call. Repeated wagers by the same
    /// bettor on the same outcome are accumulated. Wagers on a market that
    /// has already been settled are rejected.
    pub fn place_wager(
        env: Env,
        bettor: Address,
        market_id: Symbol,
        outcome: Symbol,
        amount: u64,
    ) {
        bettor.require_auth();

        if amount == 0 {
            panic!("amount must be positive");
        }

        let status: Symbol = env
            .storage()
            .instance()
            .get(&(symbol_short!("status"), market_id.clone()))
            .expect("market not found");
        if status != symbol_short!("open") {
            panic!("market is not open");
        }

        let outcomes: Vec<Symbol> = env
            .storage()
            .instance()
            .get(&(symbol_short!("outcomes"), market_id.clone()))
            .expect("market not found");
        if !outcome_exists(&outcomes, &outcome) {
            panic!("outcome not offered by market");
        }

        // Record / extend the bettor's wager for this outcome.
        let wager_key = (symbol_short!("wager"), market_id.clone(), bettor.clone());
        let mut wagers: Map<Symbol, u64> = env
            .storage()
            .instance()
            .get(&wager_key)
            .unwrap_or_else(|| Map::new(&env));
        let prev = wagers.get(outcome.clone()).unwrap_or(0);
        wagers.set(outcome.clone(), prev + amount);
        env.storage().instance().set(&wager_key, &wagers);

        // Update the aggregate total wagered on this outcome.
        let total_key = (symbol_short!("total"), market_id.clone(), outcome.clone());
        let total: u64 = env
            .storage()
            .instance()
            .get(&total_key)
            .unwrap_or(0);
        env.storage().instance().set(&total_key, &(total + amount));
    }

    /// Settle a market by declaring `winning_outcome`. Only the original
    /// creator may settle, and only while the market is still `open`. Once
    /// settled, no further wagers can be placed and winners may claim.
    pub fn settle_market(
        env: Env,
        creator: Address,
        market_id: Symbol,
        winning_outcome: Symbol,
    ) {
        creator.require_auth();

        let stored_creator: Address = env
            .storage()
            .instance()
            .get(&(symbol_short!("mkt"), market_id.clone()))
            .expect("market not found");
        if stored_creator != creator {
            panic!("only the creator can settle");
        }

        let status: Symbol = env
            .storage()
            .instance()
            .get(&(symbol_short!("status"), market_id.clone()))
            .expect("market not found");
        if status != symbol_short!("open") {
            panic!("market already settled");
        }

        let outcomes: Vec<Symbol> = env
            .storage()
            .instance()
            .get(&(symbol_short!("outcomes"), market_id.clone()))
            .expect("market not found");
        if !outcome_exists(&outcomes, &winning_outcome) {
            panic!("winning outcome not offered by market");
        }

        env.storage()
            .instance()
            .set(
                &(symbol_short!("status"), market_id.clone()),
                &symbol_short!("settled"),
            );
        env.storage()
            .instance()
            .set(&(symbol_short!("winner"), market_id.clone()), &winning_outcome);
    }

    /// Claim the caller's payout for a settled market. Returns the payout
    /// amount (in contract units) owed to the bettor, or 0 if they did not
    /// bet on the winning outcome. Each bettor may claim at most once per
    /// market. The payout is the caller's share of the entire pool,
    /// proportional to their contribution to the winning side.
    pub fn claim_payout(env: Env, bettor: Address, market_id: Symbol) -> u64 {
        bettor.require_auth();

        let status: Symbol = env
            .storage()
            .instance()
            .get(&(symbol_short!("status"), market_id.clone()))
            .expect("market not found");
        if status != symbol_short!("settled") {
            panic!("market not settled yet");
        }

        let claimed_key = (symbol_short!("claimed"), market_id.clone(), bettor.clone());
        if env.storage().instance().has(&claimed_key) {
            panic!("payout already claimed");
        }

        let winning: Symbol = env
            .storage()
            .instance()
            .get(&(symbol_short!("winner"), market_id.clone()))
            .expect("winning outcome missing");

        let wager_key = (symbol_short!("wager"), market_id.clone(), bettor.clone());
        let wagers: Map<Symbol, u64> = env
            .storage()
            .instance()
            .get(&wager_key)
            .unwrap_or_else(|| Map::new(&env));
        let my_winning = wagers.get(winning.clone()).unwrap_or(0);

        // Mark as claimed up front so any later failure still locks out
        // double-claims.
        env.storage().instance().set(&claimed_key, &true);

        if my_winning == 0 {
            return 0;
        }

        // Compute the total pool and the total on the winning side.
        let outcomes: Vec<Symbol> = env
            .storage()
            .instance()
            .get(&(symbol_short!("outcomes"), market_id.clone()))
            .expect("market not found");

        let mut total_pool: u64 = 0;
        let mut total_winning: u64 = 0;
        for i in 0..outcomes.len() {
            let o = outcomes.get(i).unwrap();
            let amt: u64 = env
                .storage()
                .instance()
                .get(&(symbol_short!("total"), market_id.clone(), o.clone()))
                .unwrap_or(0);
            total_pool = total_pool.checked_add(amt).expect("pool overflow");
            if o == winning {
                total_winning = amt;
            }
        }

        if total_winning == 0 {
            return 0;
        }

        // Caller's share of the whole pool, proportional to their winning bet.
        my_winning
            .checked_mul(total_pool)
            .expect("payout overflow")
            / total_winning
    }

    /// Read the aggregate amount wagered on a specific outcome of a market.
    /// Returns 0 if the market or outcome does not exist.
    pub fn wager_total(env: Env, market_id: Symbol, outcome: Symbol) -> u64 {
        env.storage()
            .instance()
            .get(&(symbol_short!("total"), market_id, outcome))
            .unwrap_or(0)
    }
}

/// Returns true if `outcome` is present in `outcomes`.
fn outcome_exists(outcomes: &Vec<Symbol>, outcome: &Symbol) -> bool {
    for i in 0..outcomes.len() {
        if outcomes.get(i).unwrap() == *outcome {
            return true;
        }
    }
    false
}
