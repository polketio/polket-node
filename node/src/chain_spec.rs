use hex_literal::hex;
use jsonrpc_core::serde_json::Map;
use polket_runtime::{constants::currency::DOLLARS, opaque::SessionKeys, AccountId, ObjectId, AssetsConfig,
					 BabeConfig, Balance, BalancesConfig, CouncilConfig, Forcing, GenesisConfig,
					 SessionConfig, Signature, StakerStatus, StakingConfig, SudoConfig,
					 SystemConfig, TechnicalMembershipConfig, BABE_GENESIS_EPOCH_CONFIG, WASM_BINARY};
use sc_service::{ChainType, Properties};
use sc_telemetry::TelemetryEndpoints;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::{
	traits::{IdentifyAccount, Verify},
	Perbill, BoundedVec,
};

// The URL for the telemetry server.
const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";
const DEFAULT_PROTOCOL_ID: &str = "POLKET";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
	where
		AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AccountId, AccountId, BabeId, GrandpaId) {
	(
		get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", s)),
		get_account_id_from_seed::<sr25519::Public>(s),
		get_from_seed::<BabeId>(s),
		get_from_seed::<GrandpaId>(s),
	)
}

fn session_keys(babe: BabeId, grandpa: GrandpaId) -> SessionKeys {
	SessionKeys { babe, grandpa }
}

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice")],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
					get_account_id_from_seed::<sr25519::Public>("Dave"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie"),
					// get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
					get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
					get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
				],
				vec![],
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
				],
				vec![
					get_account_id_from_seed::<sr25519::Public>("Dave"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie"),
				],
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Fork ID
		None,
		// Properties
		Some(properties()),
		// Extensions
		None,
	))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
					get_account_id_from_seed::<sr25519::Public>("Dave"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie"),
					// get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					// get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
					get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
					get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
				],
				vec![],
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
				],
				vec![
					get_account_id_from_seed::<sr25519::Public>("Dave"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie"),
				],
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		//Fork ID
		None,
		// Properties
		Some(properties()),
		// Extensions
		None,
	))
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AccountId, AccountId, BabeId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	initial_assets: Vec<(AccountId, Balance, Vec<u8>, Vec<u8>, u8, Balance)>,
	council: Vec<AccountId>,
	technical_committee: Vec<AccountId>,
	_enable_println: bool,
) -> GenesisConfig {
	const ENDOWMENT: Balance = 10_000_000_000 * DOLLARS;
	const STASH: Balance = 10_000 * DOLLARS;

	//Default 0 is Native Coin
	let mut initial_assets = initial_assets;
	initial_assets.insert(
		0,
		(root_key.clone(), 1, "Polket".into(), "PNT".into(), 12, 0),
	);

	GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
		},
		balances: BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, ENDOWMENT))
				.chain(initial_authorities.iter().map(|x| (x.0.clone(), STASH)))
				.collect(),
		},
		babe: BabeConfig {
			authorities: Default::default(),
			epoch_config: Some(BABE_GENESIS_EPOCH_CONFIG),
		},
		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| (x.0.clone(), x.0.clone(), session_keys(x.2.clone(), x.3.clone())))
				.collect::<Vec<_>>(),
		},
		staking: StakingConfig {
			validator_count: 50,
			minimum_validator_count: 4,
			stakers: initial_authorities
				.iter()
				.map(|x| (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator))
				.collect(),
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			force_era: Forcing::ForceNone,
			slash_reward_fraction: Perbill::from_percent(10),
			..Default::default()
		},
		grandpa: Default::default(),
		sudo: SudoConfig {
			// Assign network admin rights.
			key: Some(root_key.clone()),
		},
		assets: AssetsConfig {
			assets: initial_assets
				.iter()
				.enumerate()
				.map(|(i, x)| (i as ObjectId, x.0.clone(), true, x.1.clone()))
				.collect::<Vec<_>>(),
			metadata: initial_assets
				.iter()
				.enumerate()
				.map(|(i, x)| (i as ObjectId, x.2.clone(), x.3.clone(), x.4.clone()))
				.collect::<Vec<_>>(),
			accounts: initial_assets
				.iter()
				.enumerate()
				.map(|(i, x)| (i as ObjectId, x.0.clone(), 0))
				.collect::<Vec<_>>(),
		},
		council: CouncilConfig { members: council, phantom: Default::default() },
		technical_committee: Default::default(),
		technical_membership: TechnicalMembershipConfig {
			members: BoundedVec::truncate_from(technical_committee),
			phantom: Default::default(),
		},
		treasury: Default::default(),
	}
}

pub fn polket_staging_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or("polket development wasm not available")?;
	let boot_nodes = vec![];

	Ok(ChainSpec::from_genesis(
		"Polket Staging Testnet",
		"polket_staging_testnet",
		ChainType::Live,
		move || polket_staging_config_genesis(wasm_binary),
		boot_nodes,
		Some(
			TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
				.expect("Polket Staging telemetry url is valid; qed"),
		),
		Some(DEFAULT_PROTOCOL_ID),
		None,
		Some(properties()),
		Default::default(),
	))
}

fn polket_staging_config_genesis(wasm_binary: &[u8]) -> GenesisConfig {
	testnet_genesis(
		wasm_binary,
		vec![
			(
				// 5CX2rSf9XXCyaH8noAtayX99NwRbLjqVVtXJZ3rGdNT1A5vt
				hex!["141453be5b523422d80b8882fe771667129240675bfaea762fb2dc764e65c77c"].into(),
				hex!["141453be5b523422d80b8882fe771667129240675bfaea762fb2dc764e65c77c"].into(),
				hex!["141453be5b523422d80b8882fe771667129240675bfaea762fb2dc764e65c77c"]
					.unchecked_into(),
				// 5DY1hWzF8RU9RiGxo8rR8T89FGc9YPLpMxqhmwCpKFNejtzT
				hex!["410febf1a4a78f9894528be5a1df73412965388187dee8b0c7d887fc4b29ead1"]
					.unchecked_into(),
			),
			(
				// 5EpxMADT7bTRQentyuvsyiEyQcjAjXqiNtDpL9ZxUZMAUc13
				hex!["7a380be7a24d35076f36f1d4edb3066f7fe29e79b24a85432c0492dfcd9d7d0f"].into(),
				hex!["7a380be7a24d35076f36f1d4edb3066f7fe29e79b24a85432c0492dfcd9d7d0f"].into(),
				hex!["7a380be7a24d35076f36f1d4edb3066f7fe29e79b24a85432c0492dfcd9d7d0f"]
					.unchecked_into(),
				// 5Dy8NqPnuWvXRjXJo8gzkgyzHuN7MLpXgYgC9DXZ4xvk5fsa
				hex!["5437895a11f5fd777ad82e1e9982d57846b36f6e7d4c0ef1e8429817754160c0"]
					.unchecked_into(),
			),
		],
		// 5HEPAgfDwjhuCib9aqBqZovr5v7mF2bT953cVHsdHSacDT41
		hex!["e48eb50db3954253023a5b145c535b66d0115eda3ca52333e501851ccab7b819"].into(),
		vec![
			// 5HEPAgfDwjhuCib9aqBqZovr5v7mF2bT953cVHsdHSacDT41
			hex!["e48eb50db3954253023a5b145c535b66d0115eda3ca52333e501851ccab7b819"].into(),
		],
		vec![],
		vec![],
		vec![],
		true,
	)
}

fn properties() -> Properties {
	let mut properties = Map::new();
	properties.insert("tokenSymbol".into(), "PNT".into());
	properties.insert("tokenDecimals".into(), 12.into());
	return properties;
}
