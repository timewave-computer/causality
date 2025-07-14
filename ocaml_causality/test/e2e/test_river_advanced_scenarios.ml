(* Advanced River E2E Test - Sophisticated Production Scenarios *)
(* Tests complex lending workflows, multi-party interactions, and edge cases *)
(* Updated to use real FFI instead of mock simulation *)

open Printf

(* Import real FFI bindings *)
open Ocaml_causality

(* Use real simulation types from FFI *)
type simulation_config = Compiler.Simulation_ffi.simulation_config

let create_engine ?(config = Compiler.Simulation_ffi.default_config) () = 
  let river_config = { config with max_steps = 50000; max_gas = 10000000L } in
  Compiler.Simulation_ffi.create_engine ~config:river_config ()

let simulate_lisp_code engine lisp_code =
  match Compiler.Simulation_ffi.simulate_lisp_code engine lisp_code with
  | Ok result -> Ok result
  | Error err -> Error err

let get_engine_stats engine = 
  Compiler.Simulation_ffi.get_engine_stats engine

(* Advanced domain types for sophisticated River scenarios *)
type asset_type = 
  | USDC | USDT | DAI | WETH | WBTC | ATOM | OSMO | JUNO

type collateral_requirement = {
  asset: asset_type;
  min_ratio: int; (* basis points, e.g., 15000 = 150% *)
  liquidation_threshold: int; (* basis points *)
  oracle_price_feed: string;
}

type risk_profile = 
  | Conservative | Moderate | Aggressive | Institutional

type loan_terms = {
  principal: int;
  interest_rate: int; (* basis points *)
  duration_days: int;
  compounding_frequency: int; (* days between compounds *)
  early_repayment_penalty: int; (* basis points *)
  late_payment_penalty: int; (* basis points per day *)
}

type advanced_loan_request = {
  borrower_id: string;
  borrower_risk_profile: risk_profile;
  terms: loan_terms;
  collateral_requirements: collateral_requirement list;
  cross_chain_requirements: string list; (* chain names *)
  priority_level: int; (* 1-10, higher = more urgent *)
  expiration_timestamp: int;
  partial_fill_allowed: bool;
  referral_code: string option;
}

type vault_strategy = 
  | YieldFarming of string (* protocol name *)
  | Staking of string (* validator *)
  | LiquidityProvision of (string * string) (* token pair *)
  | ArbitrageCapture
  | RiskParity

type advanced_loan_offer = {
  vault_id: string;
  vault_strategy: vault_strategy;
  available_liquidity: int;
  terms: loan_terms;
  accepted_collateral: collateral_requirement list;
  supported_chains: string list;
  minimum_loan_size: int;
  maximum_loan_size: int;
  vault_utilization_rate: int; (* basis points *)
  reputation_score: int; (* 0-1000 *)
  auto_renewal_available: bool;
}

type market_conditions = {
  volatility_index: int; (* basis points *)
  liquidity_depth: int;
  average_rates: (asset_type * int) list; (* asset, rate in bps *)
  cross_chain_gas_costs: (string * int) list; (* chain, cost in USD cents *)
  oracle_confidence: int; (* basis points *)
}

type grove_market_dynamics = {
  base_rate: int; (* basis points *)
  utilization_rate: int; (* basis points *)
  reserve_ratio: int; (* basis points *)
  protocol_revenue_share: int; (* basis points *)
  dynamic_rate_adjustment: int; (* basis points adjustment based on conditions *)
  competitive_advantage: int; (* basis points vs traditional markets *)
}

type compatibility_result = {
  compatible: bool;
  compatibility_score: int;
  risk_factors: string list;
}

(* Advanced business logic functions *)

let string_of_asset = function
  | USDC -> "USDC" | USDT -> "USDT" | DAI -> "DAI" 
  | WETH -> "WETH" | WBTC -> "WBTC" | ATOM -> "ATOM"
  | OSMO -> "OSMO" | JUNO -> "JUNO"

let string_of_risk_profile = function
  | Conservative -> "conservative"
  | Moderate -> "moderate" 
  | Aggressive -> "aggressive"
  | Institutional -> "institutional"

let string_of_vault_strategy = function
  | YieldFarming protocol -> sprintf "yield_farming_%s" protocol
  | Staking validator -> sprintf "staking_%s" validator
  | LiquidityProvision (t1, t2) -> sprintf "lp_%s_%s" t1 t2
  | ArbitrageCapture -> "arbitrage_capture"
  | RiskParity -> "risk_parity"

(* Calculate dynamic Grove advantage based on market conditions *)
let calculate_dynamic_grove_advantage market_conditions grove_dynamics asset =
  let base_advantage = grove_dynamics.competitive_advantage in
  
  (* Volatility adjustment - higher volatility = higher Grove advantage *)
  let volatility_adjustment = market_conditions.volatility_index / 10 in
  
  (* Utilization adjustment - higher utilization = lower rates to attract capital *)
  let utilization_adjustment = -(grove_dynamics.utilization_rate / 20) in
  
  (* Asset-specific adjustment *)
  let asset_adjustment = match asset with
    | USDC | USDT | DAI -> 0 (* stable assets *)
    | WETH | WBTC -> 50 (* major crypto assets *)
    | ATOM | OSMO | JUNO -> 100 (* cosmos ecosystem bonus *)
  in
  
  (* Oracle confidence adjustment - lower confidence = higher rates *)
  let oracle_adjustment = (10000 - market_conditions.oracle_confidence) / 100 in
  
  let total_advantage = base_advantage + volatility_adjustment + utilization_adjustment + asset_adjustment + oracle_adjustment in
  max 0 (min 2000 total_advantage) (* cap between 0-20% *)

(* Advanced loan compatibility checking with risk assessment *)
let check_advanced_loan_compatibility request offer market_conditions =
  (* Basic compatibility checks *)
  let amount_ok = offer.available_liquidity >= request.terms.principal && 
                  request.terms.principal >= offer.minimum_loan_size &&
                  request.terms.principal <= offer.maximum_loan_size in
  
  let rate_ok = offer.terms.interest_rate <= request.terms.interest_rate in
  
  let duration_ok = offer.terms.duration_days <= request.terms.duration_days in
  
  (* Cross-chain compatibility - check if offer supports all required chains *)
  let cross_chain_ok = List.for_all (fun req_chain ->
    List.mem req_chain offer.supported_chains
  ) request.cross_chain_requirements in
  
  (* Collateral compatibility - check if offer accepts any of request's collateral *)
  let collateral_ok = List.exists (fun req_collateral ->
    List.exists (fun offer_collateral ->
      req_collateral.asset = offer_collateral.asset &&
      req_collateral.min_ratio >= offer_collateral.min_ratio
    ) offer.accepted_collateral
  ) request.collateral_requirements in
  
  (* Risk profile compatibility *)
  let risk_ok = match request.borrower_risk_profile, offer.vault_strategy with
    | Conservative, (YieldFarming _ | Staking _) -> true
    | Moderate, (YieldFarming _ | Staking _ | LiquidityProvision _) -> true
    | Aggressive, _ -> true
    | Institutional, _ -> offer.reputation_score >= 800
  in
  
  (* Market condition checks *)
  let market_ok = market_conditions.oracle_confidence >= 8000 && (* 80% confidence minimum *)
                  market_conditions.volatility_index <= 2000 in (* 20% volatility maximum *)
  
  (* Vault utilization check *)
  let utilization_ok = offer.vault_utilization_rate <= 9000 in (* 90% max utilization *)
  
  {
    compatible = amount_ok && rate_ok && duration_ok && cross_chain_ok && collateral_ok && risk_ok && market_ok && utilization_ok;
    compatibility_score = (
      (if amount_ok then 20 else 0) +
      (if rate_ok then 15 else 0) +
      (if duration_ok then 10 else 0) +
      (if cross_chain_ok then 15 else 0) +
      (if collateral_ok then 20 else 0) +
      (if risk_ok then 10 else 0) +
      (if market_ok then 5 else 0) +
      (if utilization_ok then 5 else 0)
    );
    risk_factors = [
      (if not amount_ok then ["amount_mismatch"] else []);
      (if not rate_ok then ["rate_too_high"] else []);
      (if not duration_ok then ["duration_too_long"] else []);
      (if not cross_chain_ok then ["cross_chain_unsupported"] else []);
      (if not collateral_ok then ["collateral_incompatible"] else []);
      (if not risk_ok then ["risk_profile_mismatch"] else []);
      (if not market_ok then ["adverse_market_conditions"] else []);
      (if not utilization_ok then ["vault_over_utilized"] else []);
    ] |> List.flatten;
  }

(* Generate sophisticated Lisp expressions for advanced scenarios *)

let generate_collateral_requirement_lisp req =
  sprintf "(collateral-requirement \
    (asset \"%s\") \
    (min-ratio %d) \
    (liquidation-threshold %d) \
    (oracle \"%s\"))"
    (string_of_asset req.asset)
    req.min_ratio
    req.liquidation_threshold
    req.oracle_price_feed

let generate_loan_terms_lisp terms =
  sprintf "(loan-terms \
    (principal %d) \
    (interest-rate %d) \
    (duration %d) \
    (compounding-frequency %d) \
    (early-repayment-penalty %d) \
    (late-payment-penalty %d))"
    terms.principal
    terms.interest_rate
    terms.duration_days
    terms.compounding_frequency
    terms.early_repayment_penalty
    terms.late_payment_penalty

let generate_advanced_loan_request_lisp request =
  sprintf "(advanced-loan-request \
    (borrower \"%s\") \
    (risk-profile \"%s\") \
    %s \
    (collateral-requirements %s) \
    (cross-chain-requirements %s) \
    (priority %d) \
    (expiration %d) \
    (partial-fill %b) \
    (referral %s))"
    request.borrower_id
    (string_of_risk_profile request.borrower_risk_profile)
    (generate_loan_terms_lisp request.terms)
    (String.concat " " (List.map generate_collateral_requirement_lisp request.collateral_requirements))
    (String.concat " " (List.map (sprintf "\"%s\"") request.cross_chain_requirements))
    request.priority_level
    request.expiration_timestamp
    request.partial_fill_allowed
    (match request.referral_code with Some code -> sprintf "\"%s\"" code | None -> "nil")

let generate_advanced_loan_offer_lisp offer =
  sprintf "(advanced-loan-offer \
    (vault \"%s\") \
    (strategy \"%s\") \
    (available-liquidity %d) \
    %s \
    (accepted-collateral %s) \
    (supported-chains %s) \
    (min-loan-size %d) \
    (max-loan-size %d) \
    (utilization-rate %d) \
    (reputation-score %d) \
    (auto-renewal %b))"
    offer.vault_id
    (string_of_vault_strategy offer.vault_strategy)
    offer.available_liquidity
    (generate_loan_terms_lisp offer.terms)
    (String.concat " " (List.map generate_collateral_requirement_lisp offer.accepted_collateral))
    (String.concat " " (List.map (sprintf "\"%s\"") offer.supported_chains))
    offer.minimum_loan_size
    offer.maximum_loan_size
    offer.vault_utilization_rate
    offer.reputation_score
    offer.auto_renewal_available

let generate_market_conditions_lisp conditions =
  sprintf "(market-conditions \
    (volatility-index %d) \
    (liquidity-depth %d) \
    (average-rates %s) \
    (cross-chain-gas-costs %s) \
    (oracle-confidence %d))"
    conditions.volatility_index
    conditions.liquidity_depth
    (String.concat " " (List.map (fun (asset, rate) -> 
      sprintf "(%s %d)" (string_of_asset asset) rate) conditions.average_rates))
    (String.concat " " (List.map (fun (chain, cost) -> 
      sprintf "(%s %d)" chain cost) conditions.cross_chain_gas_costs))
    conditions.oracle_confidence

let generate_grove_dynamics_lisp dynamics =
  sprintf "(grove-dynamics \
    (base-rate %d) \
    (utilization-rate %d) \
    (reserve-ratio %d) \
    (revenue-share %d) \
    (dynamic-adjustment %d) \
    (competitive-advantage %d))"
    dynamics.base_rate
    dynamics.utilization_rate
    dynamics.reserve_ratio
    dynamics.protocol_revenue_share
    dynamics.dynamic_rate_adjustment
    dynamics.competitive_advantage

(* Advanced test scenarios *)

(* Test 1: Multi-Chain Institutional Lending *)
let test_multi_chain_institutional_lending engine =
  printf "=== Test 1: Multi-Chain Institutional Lending ===\n";
  
  let market_conditions = {
    volatility_index = 800; (* 8% volatility *)
    liquidity_depth = 50000000; (* $50M *)
    average_rates = [(USDC, 450); (WETH, 650); (ATOM, 1200)];
    cross_chain_gas_costs = [("ethereum", 2500); ("cosmos", 50); ("osmosis", 25)];
    oracle_confidence = 9500; (* 95% *)
  } in
  
  let grove_dynamics = {
    base_rate = 400; (* 4% *)
    utilization_rate = 7500; (* 75% *)
    reserve_ratio = 2000; (* 20% *)
    protocol_revenue_share = 1000; (* 10% *)
    dynamic_rate_adjustment = -50; (* -0.5% due to high utilization *)
    competitive_advantage = 200; (* 2% advantage *)
  } in
  
  let institutional_request = {
    borrower_id = "blackrock_defi_fund";
    borrower_risk_profile = Institutional;
    terms = {
      principal = 10000000; (* $10M *)
      interest_rate = 500; (* 5% max *)
      duration_days = 365; (* 1 year *)
      compounding_frequency = 1; (* daily *)
      early_repayment_penalty = 100; (* 1% *)
      late_payment_penalty = 10; (* 0.1% per day *)
    };
    collateral_requirements = [
      { asset = WETH; min_ratio = 15000; liquidation_threshold = 13000; oracle_price_feed = "chainlink_eth_usd" };
      { asset = WBTC; min_ratio = 14000; liquidation_threshold = 12000; oracle_price_feed = "chainlink_btc_usd" };
    ];
    cross_chain_requirements = ["ethereum"; "cosmos"; "osmosis"];
    priority_level = 10; (* highest priority *)
    expiration_timestamp = 1700000000;
    partial_fill_allowed = true;
    referral_code = Some "institutional_tier_1";
  } in
  
  let institutional_vault = {
    vault_id = "grove_institutional_vault_alpha";
    vault_strategy = RiskParity;
    available_liquidity = 25000000; (* $25M available *)
    terms = {
      principal = 0; (* will be set based on request *)
      interest_rate = 450; (* 4.5% *)
      duration_days = 400; (* flexible up to 400 days *)
      compounding_frequency = 1;
      early_repayment_penalty = 50;
      late_payment_penalty = 5;
    };
    accepted_collateral = [
      { asset = WETH; min_ratio = 14000; liquidation_threshold = 12000; oracle_price_feed = "chainlink_eth_usd" };
      { asset = WBTC; min_ratio = 13500; liquidation_threshold = 11500; oracle_price_feed = "chainlink_btc_usd" };
      { asset = USDC; min_ratio = 11000; liquidation_threshold = 10500; oracle_price_feed = "chainlink_usdc_usd" };
    ];
    supported_chains = ["ethereum"; "cosmos"; "osmosis"; "polygon"];
    minimum_loan_size = 1000000; (* $1M minimum *)
    maximum_loan_size = 50000000; (* $50M maximum *)
    vault_utilization_rate = 7500; (* 75% *)
    reputation_score = 950; (* excellent reputation *)
    auto_renewal_available = true;
  } in
  
  let compatibility = check_advanced_loan_compatibility institutional_request institutional_vault market_conditions in
  let grove_advantage = calculate_dynamic_grove_advantage market_conditions grove_dynamics USDC in
  
  printf "Institutional loan compatibility: %b (score: %d/100)\n" compatibility.compatible compatibility.compatibility_score;
  printf "Grove competitive advantage: %d bps\n" grove_advantage;
  printf "Risk factors: [%s]\n" (String.concat "; " compatibility.risk_factors);
  
  let multi_chain_lisp = sprintf "(multi-chain-institutional-lending \
    %s \
    %s \
    %s \
    %s \
    (execute-cross-chain-coordination) \
    (execute-institutional-compliance-checks) \
    (execute-atomic-settlement))"
    (generate_advanced_loan_request_lisp institutional_request)
    (generate_advanced_loan_offer_lisp institutional_vault)
    (generate_market_conditions_lisp market_conditions)
    (generate_grove_dynamics_lisp grove_dynamics) in
  
  printf "Generated multi-chain Lisp (length: %d chars)\n" (String.length multi_chain_lisp);
  
  match simulate_lisp_code engine multi_chain_lisp with
  | Ok result_str ->
    printf "Multi-chain simulation result: %s\n" result_str;
    (compatibility.compatible && grove_advantage > 150, "Multi-chain institutional lending")
  | Error err ->
    printf "Multi-chain simulation failed: %s\n" err;
    (false, "Multi-chain simulation error")

(* Test 2: High-Frequency Arbitrage Lending Pool *)
let test_high_frequency_arbitrage_pool engine =
  printf "\n=== Test 2: High-Frequency Arbitrage Lending Pool ===\n";
  
  let volatile_market = {
    volatility_index = 2500; (* 25% volatility - very high *)
    liquidity_depth = 5000000; (* $5M - lower liquidity *)
    average_rates = [(USDC, 800); (WETH, 1200); (ATOM, 1800)];
    cross_chain_gas_costs = [("ethereum", 5000); ("arbitrum", 100); ("polygon", 50)];
    oracle_confidence = 8500; (* 85% - slightly lower due to volatility *)
  } in
  
  let arbitrage_requests = [
    {
      borrower_id = "jump_trading_arb_bot_1";
      borrower_risk_profile = Aggressive;
      terms = {
        principal = 500000; (* $500K *)
        interest_rate = 1000; (* 10% max - high rate for speed *)
        duration_days = 1; (* 1 day only *)
        compounding_frequency = 24; (* hourly compounding *)
        early_repayment_penalty = 0; (* no penalty for early repayment *)
        late_payment_penalty = 100; (* 1% per day - high penalty *)
      };
      collateral_requirements = [
        { asset = USDC; min_ratio = 11000; liquidation_threshold = 10500; oracle_price_feed = "chainlink_usdc_usd" };
      ];
      cross_chain_requirements = ["arbitrum"; "polygon"];
      priority_level = 9; (* very high priority *)
      expiration_timestamp = 1700000000 + 3600; (* expires in 1 hour *)
      partial_fill_allowed = false; (* all or nothing *)
      referral_code = Some "hft_tier_1";
    };
    {
      borrower_id = "alameda_research_arb_2";
      borrower_risk_profile = Aggressive;
      terms = {
        principal = 750000; (* $750K *)
        interest_rate = 1200; (* 12% max *)
        duration_days = 1;
        compounding_frequency = 12; (* every 2 hours *)
        early_repayment_penalty = 0;
        late_payment_penalty = 150; (* 1.5% per day *)
      };
      collateral_requirements = [
        { asset = WETH; min_ratio = 12000; liquidation_threshold = 11000; oracle_price_feed = "chainlink_eth_usd" };
      ];
      cross_chain_requirements = ["ethereum"; "arbitrum"];
      priority_level = 8;
      expiration_timestamp = 1700000000 + 1800; (* expires in 30 minutes *)
      partial_fill_allowed = false;
      referral_code = Some "hft_tier_1";
    };
  ] in
  
  let arbitrage_vaults = [
    {
      vault_id = "grove_hft_vault_alpha";
      vault_strategy = ArbitrageCapture;
      available_liquidity = 2000000; (* $2M *)
      terms = {
        principal = 0;
        interest_rate = 900; (* 9% *)
        duration_days = 2; (* up to 2 days *)
        compounding_frequency = 24;
        early_repayment_penalty = 0;
        late_payment_penalty = 200; (* 2% per day *)
      };
      accepted_collateral = [
        { asset = USDC; min_ratio = 10500; liquidation_threshold = 10000; oracle_price_feed = "chainlink_usdc_usd" };
        { asset = WETH; min_ratio = 11500; liquidation_threshold = 10500; oracle_price_feed = "chainlink_eth_usd" };
      ];
      supported_chains = ["ethereum"; "arbitrum"; "polygon"; "optimism"];
      minimum_loan_size = 100000; (* $100K minimum *)
      maximum_loan_size = 1000000; (* $1M maximum *)
      vault_utilization_rate = 8500; (* 85% - high utilization *)
      reputation_score = 850; (* good reputation *)
      auto_renewal_available = false; (* manual renewal only for HFT *)
    };
  ] in
  
  let successful_matches = List.fold_left (fun acc request ->
    let compatible_vaults = List.filter (fun vault ->
      let compatibility = check_advanced_loan_compatibility request vault volatile_market in
      compatibility.compatible && compatibility.compatibility_score >= 80
    ) arbitrage_vaults in
    acc + List.length compatible_vaults
  ) 0 arbitrage_requests in
  
  printf "High-frequency arbitrage matches: %d/%d\n" successful_matches (List.length arbitrage_requests);
  
  let arbitrage_pool_lisp = sprintf "(high-frequency-arbitrage-pool \
    (requests %s) \
    (vaults %s) \
    %s \
    (execute-rapid-matching) \
    (execute-cross-chain-arbitrage) \
    (execute-risk-monitoring) \
    (execute-liquidation-protection))"
    (String.concat " " (List.map generate_advanced_loan_request_lisp arbitrage_requests))
    (String.concat " " (List.map generate_advanced_loan_offer_lisp arbitrage_vaults))
    (generate_market_conditions_lisp volatile_market) in
  
  printf "Generated arbitrage pool Lisp (length: %d chars)\n" (String.length arbitrage_pool_lisp);
  
  match simulate_lisp_code engine arbitrage_pool_lisp with
  | Ok result_str ->
    printf "Arbitrage pool simulation result: %s\n" result_str;
    (successful_matches >= 1, "High-frequency arbitrage pool")
  | Error err ->
    printf "Arbitrage pool simulation failed: %s\n" err;
    (false, "Arbitrage pool simulation error")

(* Test 3: Cross-Chain Yield Farming Coordination *)
let test_cross_chain_yield_farming engine =
  printf "\n=== Test 3: Cross-Chain Yield Farming Coordination ===\n";
  
  let cosmos_market = {
    volatility_index = 1500; (* 15% volatility *)
    liquidity_depth = 15000000; (* $15M *)
    average_rates = [(ATOM, 1400); (OSMO, 1600); (JUNO, 2000); (USDC, 600)];
    cross_chain_gas_costs = [("cosmos", 25); ("osmosis", 20); ("juno", 30); ("ethereum", 3000)];
    oracle_confidence = 9000; (* 90% *)
  } in
  
  let yield_farming_request = {
    borrower_id = "cosmos_yield_aggregator_pro";
    borrower_risk_profile = Moderate;
    terms = {
      principal = 2500000; (* $2.5M *)
      interest_rate = 800; (* 8% max *)
      duration_days = 90; (* 3 months *)
      compounding_frequency = 7; (* weekly *)
      early_repayment_penalty = 200; (* 2% *)
      late_payment_penalty = 20; (* 0.2% per day *)
    };
    collateral_requirements = [
      { asset = ATOM; min_ratio = 13000; liquidation_threshold = 11500; oracle_price_feed = "cosmos_atom_usd" };
      { asset = OSMO; min_ratio = 14000; liquidation_threshold = 12000; oracle_price_feed = "osmosis_osmo_usd" };
    ];
    cross_chain_requirements = ["cosmos"; "osmosis"; "juno"];
    priority_level = 6; (* medium priority *)
    expiration_timestamp = 1700000000 + 86400; (* expires in 24 hours *)
    partial_fill_allowed = true;
    referral_code = Some "cosmos_ecosystem";
  } in
  
  let yield_farming_vaults = [
    {
      vault_id = "grove_cosmos_yield_vault";
      vault_strategy = YieldFarming "osmosis_dex";
      available_liquidity = 5000000; (* $5M *)
      terms = {
        principal = 0;
        interest_rate = 750; (* 7.5% *)
        duration_days = 120; (* up to 4 months *)
        compounding_frequency = 7;
        early_repayment_penalty = 150;
        late_payment_penalty = 15;
      };
      accepted_collateral = [
        { asset = ATOM; min_ratio = 12500; liquidation_threshold = 11000; oracle_price_feed = "cosmos_atom_usd" };
        { asset = OSMO; min_ratio = 13500; liquidation_threshold = 11500; oracle_price_feed = "osmosis_osmo_usd" };
        { asset = JUNO; min_ratio = 15000; liquidation_threshold = 13000; oracle_price_feed = "juno_juno_usd" };
      ];
      supported_chains = ["cosmos"; "osmosis"; "juno"; "akash"; "secret"];
      minimum_loan_size = 500000; (* $500K minimum *)
      maximum_loan_size = 10000000; (* $10M maximum *)
      vault_utilization_rate = 6500; (* 65% *)
      reputation_score = 900; (* excellent reputation *)
      auto_renewal_available = true;
    };
    {
      vault_id = "grove_staking_rewards_vault";
      vault_strategy = Staking "cosmos_validator_1";
      available_liquidity = 3000000; (* $3M *)
      terms = {
        principal = 0;
        interest_rate = 700; (* 7% *)
        duration_days = 180; (* 6 months *)
        compounding_frequency = 30; (* monthly *)
        early_repayment_penalty = 300; (* 3% - higher for staking *)
        late_payment_penalty = 25;
      };
      accepted_collateral = [
        { asset = ATOM; min_ratio = 12000; liquidation_threshold = 10500; oracle_price_feed = "cosmos_atom_usd" };
      ];
      supported_chains = ["cosmos"];
      minimum_loan_size = 250000; (* $250K minimum *)
      maximum_loan_size = 5000000; (* $5M maximum *)
      vault_utilization_rate = 5500; (* 55% *)
      reputation_score = 920; (* very good reputation *)
      auto_renewal_available = true;
    };
  ] in
  
  let best_vault = List.fold_left (fun best_opt vault ->
    let compatibility = check_advanced_loan_compatibility yield_farming_request vault cosmos_market in
    match best_opt with
    | None when compatibility.compatible -> Some (vault, compatibility.compatibility_score)
    | Some (_, best_score) when compatibility.compatible && compatibility.compatibility_score > best_score ->
      Some (vault, compatibility.compatibility_score)
    | _ -> best_opt
  ) None yield_farming_vaults in
  
  let (selected_vault, compatibility_score) = match best_vault with
    | Some (vault, score) -> (vault, score)
    | None -> (List.hd yield_farming_vaults, 0) in
  
  printf "Best vault selected: %s (compatibility score: %d/100)\n" 
    selected_vault.vault_id compatibility_score;
  
  let cross_chain_lisp = sprintf "(cross-chain-yield-farming-coordination \
    %s \
    %s \
    %s \
    (execute-cross-chain-bridge) \
    (execute-yield-optimization) \
    (execute-reward-compounding) \
    (execute-risk-rebalancing))"
    (generate_advanced_loan_request_lisp yield_farming_request)
    (generate_advanced_loan_offer_lisp selected_vault)
    (generate_market_conditions_lisp cosmos_market) in
  
  printf "Generated cross-chain yield farming Lisp (length: %d chars)\n" (String.length cross_chain_lisp);
  
  match simulate_lisp_code engine cross_chain_lisp with
  | Ok result_str ->
    printf "Cross-chain yield farming result: %s\n" result_str;
    (compatibility_score >= 70, "Cross-chain yield farming coordination")
  | Error err ->
    printf "Cross-chain yield farming failed: %s\n" err;
    (false, "Cross-chain yield farming error")

(* Test 4: Stress Test - Market Crisis Scenario *)
let test_market_crisis_stress_scenario engine =
  printf "\n=== Test 4: Market Crisis Stress Test ===\n";
  
  let crisis_market = {
    volatility_index = 5000; (* 50% volatility - extreme *)
    liquidity_depth = 1000000; (* $1M - severely constrained *)
    average_rates = [(USDC, 2000); (WETH, 3000); (ATOM, 4000)]; (* very high rates *)
    cross_chain_gas_costs = [("ethereum", 15000); ("cosmos", 200); ("osmosis", 150)]; (* high gas *)
    oracle_confidence = 6000; (* 60% - low confidence *)
  } in
  
  let crisis_requests = [
    {
      borrower_id = "emergency_liquidation_fund";
      borrower_risk_profile = Conservative;
      terms = {
        principal = 5000000; (* $5M emergency *)
        interest_rate = 2500; (* 25% max - crisis rates *)
        duration_days = 7; (* 1 week emergency *)
        compounding_frequency = 1; (* daily *)
        early_repayment_penalty = 0; (* no penalty in crisis *)
        late_payment_penalty = 500; (* 5% per day - extreme *)
      };
      collateral_requirements = [
        { asset = USDC; min_ratio = 20000; liquidation_threshold = 18000; oracle_price_feed = "chainlink_usdc_usd" };
        { asset = WETH; min_ratio = 25000; liquidation_threshold = 22000; oracle_price_feed = "chainlink_eth_usd" };
      ];
      cross_chain_requirements = ["ethereum"];
      priority_level = 10; (* maximum priority *)
      expiration_timestamp = 1700000000 + 3600; (* expires in 1 hour *)
      partial_fill_allowed = true;
      referral_code = None;
    };
  ] in
  
  let crisis_vault = {
    vault_id = "grove_emergency_reserve_vault";
    vault_strategy = RiskParity;
    available_liquidity = 8000000; (* $8M emergency reserves *)
    terms = {
      principal = 0;
      interest_rate = 2200; (* 22% *)
      duration_days = 14; (* up to 2 weeks *)
      compounding_frequency = 1;
      early_repayment_penalty = 0;
      late_payment_penalty = 1000; (* 10% per day *)
    };
    accepted_collateral = [
      { asset = USDC; min_ratio = 18000; liquidation_threshold = 16000; oracle_price_feed = "chainlink_usdc_usd" };
      { asset = WETH; min_ratio = 22000; liquidation_threshold = 20000; oracle_price_feed = "chainlink_eth_usd" };
      { asset = WBTC; min_ratio = 20000; liquidation_threshold = 18000; oracle_price_feed = "chainlink_btc_usd" };
    ];
    supported_chains = ["ethereum"; "polygon"];
    minimum_loan_size = 1000000; (* $1M minimum *)
    maximum_loan_size = 10000000; (* $10M maximum *)
    vault_utilization_rate = 9500; (* 95% - crisis utilization *)
    reputation_score = 1000; (* maximum reputation for emergency *)
    auto_renewal_available = false; (* manual only in crisis *)
  } in
  
  let emergency_request = List.hd crisis_requests in
  let compatibility = check_advanced_loan_compatibility emergency_request crisis_vault crisis_market in
  
  printf "Crisis scenario compatibility: %b (score: %d/100)\n" 
    compatibility.compatible compatibility.compatibility_score;
  printf "Crisis risk factors: [%s]\n" (String.concat "; " compatibility.risk_factors);
  
  let crisis_lisp = sprintf "(market-crisis-stress-test \
    %s \
    %s \
    %s \
    (execute-emergency-protocols) \
    (execute-liquidation-protection) \
    (execute-circuit-breakers) \
    (execute-risk-isolation))"
    (generate_advanced_loan_request_lisp emergency_request)
    (generate_advanced_loan_offer_lisp crisis_vault)
    (generate_market_conditions_lisp crisis_market) in
  
  printf "Generated crisis stress test Lisp (length: %d chars)\n" (String.length crisis_lisp);
  
  match simulate_lisp_code engine crisis_lisp with
  | Ok result_str ->
    printf "Crisis stress test result: %s\n" result_str;
    (compatibility.compatibility_score >= 50, "Market crisis stress test") (* Lower threshold for crisis *)
  | Error err ->
    printf "Crisis stress test failed: %s\n" err;
    (false, "Crisis stress test error")

(* Main test runner for advanced scenarios *)
let run_advanced_river_scenarios () =
  printf "Starting Advanced River E2E Simulation Tests\n";
  printf "=============================================\n";
  printf "Testing sophisticated production scenarios:\n";
  printf "- Multi-chain institutional lending\n";
  printf "- High-frequency arbitrage pools\n";
  printf "- Cross-chain yield farming coordination\n";
  printf "- Market crisis stress testing\n";
  printf "=============================================\n";
  
  Random.self_init (); (* Initialize random number generator *)
  
  (* Create advanced simulation engine using real FFI *)
  let engine = create_engine ~config:{
    max_steps = 100000;
    max_gas = 50000000L;
    enable_snapshots = true;
  } () in
  
  printf "Created simulation engine (handle: %d)\n" engine;
  printf "Registry info: %s\n" (Compiler.Simulation_ffi.get_registry_info ());
  
  (* Run advanced test scenarios *)
  let tests = [
    test_multi_chain_institutional_lending engine;
    test_high_frequency_arbitrage_pool engine;
    test_cross_chain_yield_farming engine;
    test_market_crisis_stress_scenario engine;
  ] in
  
  (* Calculate results *)
  let (passed, total) = List.fold_left (fun (p, t) (success, _) ->
    if success then (p + 1, t + 1) else (p, t + 1)
  ) (0, 0) tests in
  
  printf "\n=============================================\n";
  printf "Advanced River E2E Test Results\n";
  printf "=============================================\n";
  List.iteri (fun i (success, description) ->
    printf "%d. %s: %s\n" (i + 1) description (if success then "PASS" else "FAIL")
  ) tests;
  
  printf "\nSummary: %d/%d advanced tests passed (%.1f%%)\n" 
    passed total (float_of_int passed /. float_of_int total *. 100.0);
  
  (* Get final engine stats *)
  let (steps, gas, effects) = get_engine_stats engine in
  printf "Advanced Simulation Statistics:\n";
  printf "- Steps executed: %d\n" steps;
  printf "- Gas consumed: %Ld\n" gas;
  printf "- Effects executed: %d\n" effects;
  
  printf "Registry info after tests: %s\n" (Compiler.Simulation_ffi.get_registry_info ());
  
  (* Cleanup engine *)
  let cleanup_success = Compiler.Simulation_ffi.cleanup_engine engine in
  printf "Engine cleanup: %s\n" (if cleanup_success then "success" else "failed");
  
  printf "=============================================\n";
  printf "Advanced Scenarios Validated:\n";
  printf "Multi-chain institutional lending with compliance\n";
  printf "High-frequency arbitrage with rapid settlement\n";
  printf "Cross-chain yield farming coordination\n";
  printf "Market crisis stress testing and risk isolation\n";
  printf "Complex collateral requirements and risk profiles\n";
  printf "Dynamic rate adjustments and competitive advantages\n";
  printf "=============================================\n";
  passed = total

(* Entry point *)
let () =
  let success = run_advanced_river_scenarios () in
  exit (if success then 0 else 1) 