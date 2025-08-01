pub mod consensus;
pub mod genesis;

use std::collections::{BTreeMap, BTreeSet, btree_map::Entry};

use bytes::Bytes;
use copper_base::{
	Address, Initialized, MsgHandler, SignerExtractor,
	block::{Block, BlockHash, BlockHeight},
	context::MutContext,
	tx::{Signed, Tx},
};
use copper_facet_account::AccountFacet;
use copper_store::{CommitKVStore, GetKVStore, InsertKVStore, RemoveKVStore};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use nebz::NonEmptyBz;
use sha2::{Digest, Sha256};

pub fn derive_address(vk: &VerifyingKey) -> Address {
	let hash = Sha256::digest(vk.as_bytes());
	let mut address = [0; 20];
	address.copy_from_slice(&hash[hash.len() - 20..]);
	address
}

pub struct App<'f, S, R> {
	store: S,
	chain_id: Bytes,
	facets: BTreeMap<&'static str, &'f (dyn Facet<S> + Sync)>,
	register: R,
}

pub struct Registering;

pub struct Registered<'a> {
	account_facet: &'a AccountFacet<Initialized>,
}

trait Facet<S>: SignerExtractor<S> + MsgHandler<S> {}

impl<'f, S> App<'f, S, Registering> {
	pub fn new(store: S, chain_id: Bytes) -> Self {
		Self { store, chain_id, facets: BTreeMap::new(), register: Registering }
	}

	pub fn register_facet<F>(&mut self, name: &'static str, facet: &'f F) -> anyhow::Result<()>
	where
		F: SignerExtractor<S> + MsgHandler<S> + Sync + 'static,
	{
		let Entry::Vacant(entry) = self.facets.entry(name) else {
			anyhow::bail!("facet with name {name} already registered");
		};

		entry.insert(facet as _);

		Ok(())
	}

	pub fn into_registered(
		self,
		account_facet: &'f AccountFacet<Initialized>,
	) -> App<'f, S, Registered<'f>> {
		let Self { store, chain_id, facets, .. } = self;

		let register = Registered { account_facet };

		App { store, chain_id, facets, register }
	}
}

impl<S> App<'_, S, Registered<'_>>
where
	S: GetKVStore + InsertKVStore<Value: From<Vec<u8>>> + RemoveKVStore,
	NonEmptyBz<S::Key>: for<'k> From<NonEmptyBz<&'k [u8]>>,
{
	pub fn process_block(&mut self, block: &Block) -> anyhow::Result<()> {
		for tx in block.txs() {
			self.process_tx(block.header().height(), tx)?;
		}

		Ok(())
	}

	fn process_tx(&mut self, height: BlockHeight, tx: &Tx<Signed>) -> anyhow::Result<()> {
		let signers = self.verify_tx_sigs(tx)?;

		let tx_bz = borsh::to_vec(tx)?;

		for msg in tx.msgs() {
			let mut ctx = MutContext::new(height, &tx_bz, &mut self.store);

			let facet = self
				.facets
				.get(msg.facet())
				.ok_or_else(|| anyhow::anyhow!("no signer extractor for facet: {}", msg.facet()))?;

			// verify msg signers
			if !facet.extract_signers(&mut ctx, msg)?.iter().all(|s| signers.contains(s)) {
				anyhow::bail!("missing required signer");
			}

			// route msg to msg handler
			facet.handle_msg(&mut ctx, msg)?;
		}

		for signer in signers {
			self.register.account_facet.keeper().increment_sequence(&mut self.store, &signer)?;
		}

		Ok(())
	}

	fn verify_tx_sigs(&mut self, tx: &Tx<Signed>) -> anyhow::Result<BTreeSet<Address>> {
		let hash_to_sign = tx.hash_to_sign(&self.chain_id);

		let mut signers = BTreeSet::new();

		for (signer_info, sig) in tx.signer_infos().iter().zip(tx.sigs()) {
			let tx_vk = VerifyingKey::from_bytes(signer_info.pub_key().as_ref().try_into()?)?;

			let signer = derive_address(&tx_vk);

			signers.insert(signer);

			let address_hex = const_hex::const_encode::<20, false>(&signer);

			let account_keeper = self.register.account_facet.keeper();
			let account = match account_keeper.get_account(&self.store, &signer)? {
				Some(a) => a,
				None => account_keeper.init_account(&mut self.store, &signer)?,
			};

			let account_vk = account
				.pub_key()
				.map(|pk| pk.as_ref().try_into())
				.transpose()?
				.map(|bz| VerifyingKey::from_bytes(&bz))
				.transpose()?;

			if let Some(vk) = &account_vk
				&& vk.as_bytes().as_slice() != signer_info.pub_key()
			{
				anyhow::bail!(
					"account public key conflict for address {}",
					address_hex.as_str(),
				);
			}

			if signer_info.sequence() != account.sequence() {
				anyhow::bail!(
					"account sequence {} invalid for address {}; expected {}",
					signer_info.sequence(),
					address_hex.as_str(),
					account.sequence(),
				);
			}

			sig.as_ref()
				.try_into()
				.map(Signature::from_bytes)
				.map(|s| tx_vk.verify(&hash_to_sign, &s))??;

			if account_vk.is_none() {
				account_keeper.set_pub_key(
					&mut self.store,
					&signer,
					signer_info.pub_key().clone(),
				)?;
			}
		}

		Ok(signers)
	}
}

impl<S> App<'_, S, Registered<'_>> {
	pub fn account_facet(&self) -> &AccountFacet<Initialized> {
		self.register.account_facet
	}

	pub fn commit(&mut self) -> anyhow::Result<BlockHash>
	where
		S: CommitKVStore<Hash = [u8; 32]>,
	{
		self.store.commit().map_err(|_| anyhow::anyhow!("failed to commit"))?;

		Ok(self.store.hash())
	}
}

impl<S, T> Facet<S> for T where T: SignerExtractor<S> + MsgHandler<S> {}
