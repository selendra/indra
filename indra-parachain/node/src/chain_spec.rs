use cumulus_primitives_core::ParaId;
use hex_literal::hex;
use indra_runtime::constants::currency::{EXISTENTIAL_DEPOSIT, UNITS};
use parachains_common::{AccountId, AuraId, Signature};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<indra_runtime::GenesisConfig, Extensions>;

pub fn indra_tesnet_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../res/parachain-indra.json")[..])
}

/// Helper function to generate a crypto pair from seed
pub fn get_public_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
	/// The relay chain of the Parachain.
	pub relay_chain: String,
	/// The id of the Parachain.
	pub para_id: u32,
}

impl Extensions {
	/// Try to get the extension from the given `ChainSpec`.
	pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
		sc_chain_spec::get_extension(chain_spec.extensions())
	}
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate collator keys from seed.
///
/// This function's return type must always match the session keys of the chain in tuple format.
pub fn get_collator_keys_from_seed(seed: &str) -> AuraId {
	get_public_from_seed::<AuraId>(seed)
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_public_from_seed::<TPublic>(seed)).into_account()
}

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn template_session_keys(keys: AuraId) -> indra_runtime::SessionKeys {
	indra_runtime::SessionKeys { aura: keys }
}

pub fn development_config() -> ChainSpec {
	// Give your base currency a sel name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "PSEL".into());
	properties.insert("tokenDecimals".into(), 18.into());
	properties.insert("ss58Format".into(), 42.into());

	ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || {
			testnet_genesis(
				// initial collators.
				vec![
					(
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						get_collator_keys_from_seed("Alice"),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						get_collator_keys_from_seed("Bob"),
					),
				],
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Dave"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
					get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
					get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
				],
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				1000.into(),
			)
		},
		Vec::new(),
		None,
		None,
		None,
		Extensions {
			relay_chain: "selendra-dev".into(), // You MUST set this to the correct network!
			para_id: 1000,
		},
	)
}

pub fn local_testnet_config() -> ChainSpec {
	// Give your base currency a sel name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "PSEL".into());
	properties.insert("tokenDecimals".into(), 18.into());
	properties.insert("ss58Format".into(), 42.into());

	ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				// initial collators.
				vec![
					(
						get_account_id_from_seed::<sr25519::Public>("Dave"),
						get_collator_keys_from_seed("Alice"),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Charlie"),
						get_collator_keys_from_seed("Bob"),
					),
				],
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
				],
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				1000.into(),
			)
		},
		// Bootnodes
		Vec::new(),
		// Telemetry
		None,
		// Protocol ID
		Some("sel"),
		// Properties
		Some(properties),
		// Extensions
		Extensions {
			relay_chain: "selendra-local".into(), // You MUST set this to the correct network!
			para_id: 1000,
		},
	)
}

pub fn indra_config() -> ChainSpec {
	// Give your base currency a sel name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "PSEL".into());
	properties.insert("tokenDecimals".into(), 18.into());
	properties.insert("ss58Format".into(), 42.into());

	ChainSpec::from_genesis(
		// Name
		"Indra Testnet",
		// ID
		"indra",
		ChainType::Live,
		move || {
			testnet_genesis(
				// initial collators.
				vec![
					(
						hex!("66006c1c52a614ddd82fac2241bf5f61e9edd4647b114dfe66842104fd40b56c")
							.into(),
						hex!("66006c1c52a614ddd82fac2241bf5f61e9edd4647b114dfe66842104fd40b56c")
							.unchecked_into(),
					),
					(
						hex!("dca5d29a84f0aaa2bc623f9f11d7f21cafa3b2e358bee3a950ae121fd58b8f31")
							.into(),
						hex!("dca5d29a84f0aaa2bc623f9f11d7f21cafa3b2e358bee3a950ae121fd58b8f31")
							.unchecked_into(),
					),
					(
						hex!("3c7bb743843eea4d4fdb2621bcc2d66fb1b2844fc1fd8ad0ed8faf1840894e26")
							.into(),
						hex!("3c7bb743843eea4d4fdb2621bcc2d66fb1b2844fc1fd8ad0ed8faf1840894e26")
							.unchecked_into(),
					),
				],
				// endowed_accounts
				vec![
					hex!("9065517c47c4710a44769adac5638b2f77db30b71ac10cb5b30e467b1af98f19").into(),
					hex!("cc4f1627136cc9ab173ca1079fcebcb4b1b374cee0d62ebe2ccbca355e3eaf74").into(),
					hex!("fe99233e10d00fb26b37f2c85e3690c8b9998b51f07f3c0bf519b8320a38803a").into(),
					hex!("bc061a86ddb82e9079feb48da3abc67da083b84455713887e83f1426de043b6e").into(),
				],
				// sudo key
				hex!("9065517c47c4710a44769adac5638b2f77db30b71ac10cb5b30e467b1af98f19").into(),
				// parachain id
				1000.into(),
			)
		},
		// Bootnodes
		Vec::new(),
		// Telemetry
		None,
		// Protocol ID
		Some("sel"),
		// Properties
		Some(properties),
		// Extensions
		Extensions {
			relay_chain: "selendra-testnet".into(), // You MUST set this to the correct network!
			para_id: 1000,
		},
	)
}

fn testnet_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<AccountId>,
	root_key: AccountId,
	id: ParaId,
) -> indra_runtime::GenesisConfig {
	const ENDOWMENT: u128 = 250000 * UNITS;

	indra_runtime::GenesisConfig {
		system: indra_runtime::SystemConfig {
			code: indra_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
		},
		balances: indra_runtime::BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|k| (k, ENDOWMENT)).collect(),
		},
		parachain_info: indra_runtime::ParachainInfoConfig { parachain_id: id },
		collator_selection: indra_runtime::CollatorSelectionConfig {
			invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: EXISTENTIAL_DEPOSIT * 16,
			..Default::default()
		},
		session: indra_runtime::SessionConfig {
			keys: invulnerables
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),                 // account id
						acc,                         // validator id
						template_session_keys(aura), // session keys
					)
				})
				.collect(),
		},
		// no need to pass anything to aura, in fact it will panic if we do. Session will take care
		// of this.
		aura: Default::default(),
		aura_ext: Default::default(),
		parachain_system: Default::default(),
		sudo: indra_runtime::SudoConfig { key: root_key },
		evm: Default::default(),
		ethereum: Default::default(),
		base_fee: Default::default(),
	}
}
