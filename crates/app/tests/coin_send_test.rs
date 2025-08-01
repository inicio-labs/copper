use core::num::NonZeroU64;

use app::{App, genesis::AppGenesis};
use bytes::Bytes;
use copper_base::{
	GenesisInitializer,
	block::{Block, BlockHeader},
	coin::Coin,
	msg::Msg,
	tx::{SignerInfo, Tx},
};
use copper_facet_account::{AccountFacet, genesis::AccountGenesis};
use copper_facet_bank::{
	BankFacet,
	genesis::{Balance, BankGenesis},
	types::CoinSend,
};
use copper_store::{CommitKVStore, iavl::IavlStore};
use ed25519_dalek::{SigningKey, ed25519::signature::SignerMut};
use redb::{Database, backends::InMemoryBackend};

const CHAIN_ID: &str = "poc-chain";

#[test]
fn coin_send_works() -> anyhow::Result<()> {
	let mut alice_sk = {
		let sk_bytes = const_hex::decode_to_array(
			"02360086918045981f7436366a171c6ea2943e0b42dc7d07336f0710c6b3a95e",
		)?;

		SigningKey::from_bytes(&sk_bytes)
	};

	let alice_vk = alice_sk.verifying_key();

	let alice_address = app::derive_address(&alice_vk);

	println!(
		"alice address = {}",
		const_hex::const_encode::<20, false>(&alice_address).as_str(),
	);

	let genesis = {
		let account_genesis = AccountGenesis::new(vec![]);

		let balance = Balance::new(alice_address, vec![Coin::new("mudra".parse()?, 10_000)]);
		let bank_genesis = BankGenesis::new(vec![balance]);

		AppGenesis::new(
			CHAIN_ID.into(),
			NonZeroU64::MIN,
			account_genesis,
			bank_genesis,
		)
	};

	let db = Database::builder().create_with_backend(InMemoryBackend::new())?;
	let mut store = IavlStore::with_redb(db, "poc-store")?;

	let account_facet = AccountFacet::new().init_genesis(&mut store, &genesis.account)?;
	let bank_facet = BankFacet::new().init_genesis(&mut store, &genesis.bank)?;

	store.commit()?;

	let balance =
		bank_facet.keeper().balance(&store, &app::derive_address(&alice_vk), &"moola".parse()?);
	println!("{balance:?}");

	let chain_id = Bytes::copy_from_slice(genesis.chain_id.as_bytes());
	let mut uapp = App::new(store, chain_id);

	uapp.register_facet("account", &account_facet)?;
	uapp.register_facet("bank", &bank_facet)?;

	let mut app = uapp.into_registered(&account_facet);

	let mut bob_sk = {
		let sk_bytes = const_hex::decode_to_array(
			"9fc1c9699bffa8f97bfcd022c39bb88bebcbc33403a8999713bfabb68cf6a759",
		)?;

		SigningKey::from_bytes(&sk_bytes)
	};

	let bob_vk = bob_sk.verifying_key();

	let bob_address = app::derive_address(&bob_vk);

	println!(
		"bob address = {}",
		const_hex::const_encode::<20, false>(&bob_address).as_str(),
	);

	let msg_one = {
		let coin = Coin::new("mudra".parse()?, 4_000);
		let send_coin_msg = CoinSend::new(app::derive_address(&alice_vk), bob_address, coin);

		send_coin_msg.to_routable_msg()
	};

	let msg_two = {
		let coin = Coin::new("mudra".parse()?, 3_000);
		let send_coin_msg = CoinSend::new(app::derive_address(&bob_vk), alice_address, coin);

		send_coin_msg.to_routable_msg()
	};

	let alice_signer_info = SignerInfo::new(Bytes::copy_from_slice(alice_vk.as_bytes()), 0);
	let bob_signer_info = SignerInfo::new(Bytes::copy_from_slice(bob_vk.as_bytes()), 0);

	let tx = Tx::new(
		vec![msg_one, msg_two],
		vec![alice_signer_info, bob_signer_info],
	);

	let hash_to_sign = tx.hash_to_sign(CHAIN_ID.as_bytes());

	let alice_sig = alice_sk.try_sign(&hash_to_sign)?;
	let bob_sig = bob_sk.try_sign(&hash_to_sign)?;

	let signed_tx = tx
		.into_signed(vec![alice_sig.to_vec().into(), bob_sig.to_vec().into()])
		.map_err(|_| anyhow::anyhow!("every signer's sign must be present"))?;

	let header = BlockHeader::new(1.try_into()?);

	let block = Block::new(header, vec![signed_tx]);

	app.process_block(&block)?;

	app.commit()?;

	Ok(())
}
