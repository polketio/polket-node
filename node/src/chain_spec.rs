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
	Perbill, Permill, BoundedVec,
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
		None,
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
		None,
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
	initial_assets: Vec<(AccountId, Balance, Vec<u8>, Vec<u8>, u8, Balance, Permill)>,
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
		(root_key.clone(), 1, "Kusama".into(), "KSM".into(), 12, 0, Permill::from_percent(0)),
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
				hex!["aa20c176700afd883bb8f157bc0a9e83a8bb1c52c3b20d8f7d4d3dab771d5d60"].into(),
				hex!["aa20c176700afd883bb8f157bc0a9e83a8bb1c52c3b20d8f7d4d3dab771d5d60"].into(),
				hex!["aa20c176700afd883bb8f157bc0a9e83a8bb1c52c3b20d8f7d4d3dab771d5d60"]
					.unchecked_into(),
				hex!["f9175ab5a7ebaa82304f814df70fdfed2e02fed9bbf81cfab59d1be9af881520"]
					.unchecked_into(),
			),
			(
				hex!["589114c02813a31cae0242ada338955c7caf8f88a1646e5de0dbc1793ab1f57c"].into(),
				hex!["589114c02813a31cae0242ada338955c7caf8f88a1646e5de0dbc1793ab1f57c"].into(),
				hex!["589114c02813a31cae0242ada338955c7caf8f88a1646e5de0dbc1793ab1f57c"]
					.unchecked_into(),
				hex!["92cc3a02ff60da96ca2d1a2536e505f74f88992c2860be7fbfdf3edfd11e8759"]
					.unchecked_into(),
			),
		],
		// 5FX1PbLwgJHuvjmVwk1yQAz3KTVsgopGdLyB44Ja6xJ2szKS
		hex!["98c41a5155c2f06ea566dc60ab40ce2738b6ade3c55ef15ad77fa3f60c1b4603"].into(),
		vec![
			// 5FX1PbLwgJHuvjmVwk1yQAz3KTVsgopGdLyB44Ja6xJ2szKS
			hex!["98c41a5155c2f06ea566dc60ab40ce2738b6ade3c55ef15ad77fa3f60c1b4603"].into(),
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
