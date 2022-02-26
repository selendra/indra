#![allow(non_snake_case)]

use super::*;
use codec::{Decode, Encode, MaxEncodedLen};
use enumflags2::BitFlags;
use frame_support::{
	traits::{ConstU32, Get},
	BoundedVec, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound,
};

use scale_info::{
	build::{Fields, Variants},
	meta_type, Path, Type, TypeInfo, TypeParameter,
};
use sp_runtime::{traits::Zero, RuntimeDebug};
use sp_std::{fmt::Debug, iter::once, ops::Add, prelude::*};

/// Either underlying data blob if it is at most 32 bytes, or a hash of it. If the data is greater
/// than 32-bytes then it will be truncated when encoding.
///
/// Can also be `None`.
#[derive(Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen)]
pub enum Data {
	/// No data here.
	None,
	/// The data is stored directly.
	Raw(BoundedVec<u8, ConstU32<32>>),
	/// Only the Blake2 hash of the data is stored. The preimage of the hash may be retrieved
	/// through some hash-lookup service.
	BlakeTwo256([u8; 32]),
	/// Only the SHA2-256 hash of the data is stored. The preimage of the hash may be retrieved
	/// through some hash-lookup service.
	Sha256([u8; 32]),
	/// Only the Keccak-256 hash of the data is stored. The preimage of the hash may be retrieved
	/// through some hash-lookup service.
	Keccak256([u8; 32]),
	/// Only the SHA3-256 hash of the data is stored. The preimage of the hash may be retrieved
	/// through some hash-lookup service.
	ShaThree256([u8; 32]),
}

impl Decode for Data {
	fn decode<I: codec::Input>(input: &mut I) -> sp_std::result::Result<Self, codec::Error> {
		let b = input.read_byte()?;
		Ok(match b {
			0 => Data::None,
			n @ 1..=33 => {
				let mut r: BoundedVec<_, _> = vec![0u8; n as usize - 1]
					.try_into()
					.expect("bound checked in match arm condition; qed");
				input.read(&mut r[..])?;
				Data::Raw(r)
			},
			34 => Data::BlakeTwo256(<[u8; 32]>::decode(input)?),
			35 => Data::Sha256(<[u8; 32]>::decode(input)?),
			36 => Data::Keccak256(<[u8; 32]>::decode(input)?),
			37 => Data::ShaThree256(<[u8; 32]>::decode(input)?),
			_ => return Err(codec::Error::from("invalid leading byte")),
		})
	}
}

impl Encode for Data {
	fn encode(&self) -> Vec<u8> {
		match self {
			Data::None => vec![0u8; 1],
			Data::Raw(ref x) => {
				let l = x.len().min(32);
				let mut r = vec![l as u8 + 1; l + 1];
				r[1..].copy_from_slice(&x[..l as usize]);
				r
			},
			Data::BlakeTwo256(ref h) => once(34u8).chain(h.iter().cloned()).collect(),
			Data::Sha256(ref h) => once(35u8).chain(h.iter().cloned()).collect(),
			Data::Keccak256(ref h) => once(36u8).chain(h.iter().cloned()).collect(),
			Data::ShaThree256(ref h) => once(37u8).chain(h.iter().cloned()).collect(),
		}
	}
}
impl codec::EncodeLike for Data {}

/// Add a Raw variant with the given index and a fixed sized byte array
macro_rules! data_raw_variants {
    ($variants:ident, $(($index:literal, $size:literal)),* ) => {
		$variants
		$(
			.variant(concat!("Raw", stringify!($size)), |v| v
				.index($index)
				.fields(Fields::unnamed().field(|f| f.ty::<[u8; $size]>()))
			)
		)*
    }
}

impl TypeInfo for Data {
	type Identity = Self;

	fn type_info() -> Type {
		let variants = Variants::new().variant("None", |v| v.index(0));

		// create a variant for all sizes of Raw data from 0-32
		let variants = data_raw_variants!(
			variants,
			(1, 0),
			(2, 1),
			(3, 2),
			(4, 3),
			(5, 4),
			(6, 5),
			(7, 6),
			(8, 7),
			(9, 8),
			(10, 9),
			(11, 10),
			(12, 11),
			(13, 12),
			(14, 13),
			(15, 14),
			(16, 15),
			(17, 16),
			(18, 17),
			(19, 18),
			(20, 19),
			(21, 20),
			(22, 21),
			(23, 22),
			(24, 23),
			(25, 24),
			(26, 25),
			(27, 26),
			(28, 27),
			(29, 28),
			(30, 29),
			(31, 30),
			(32, 31),
			(33, 32)
		);

		let variants = variants
			.variant("BlakeTwo256", |v| {
				v.index(34).fields(Fields::unnamed().field(|f| f.ty::<[u8; 32]>()))
			})
			.variant("Sha256", |v| {
				v.index(35).fields(Fields::unnamed().field(|f| f.ty::<[u8; 32]>()))
			})
			.variant("Keccak256", |v| {
				v.index(36).fields(Fields::unnamed().field(|f| f.ty::<[u8; 32]>()))
			})
			.variant("ShaThree256", |v| {
				v.index(37).fields(Fields::unnamed().field(|f| f.ty::<[u8; 32]>()))
			});

		Type::builder().path(Path::new("Data", module_path!())).variant(variants)
	}
}

impl Default for Data {
	fn default() -> Self {
		Self::None
	}
}

/// An identifier for a single name registrar/identity verification service.
pub type RegistrarIndex = u32;

/// The fields that we use to identify the owner of an account with. Each corresponds to a field
/// in the `IdentityInfo` struct.
#[repr(u64)]
#[derive(Clone, Copy, PartialEq, Eq, BitFlags, RuntimeDebug, TypeInfo)]
pub enum IdentityField {
	Display = 0b0000000000000000000000000000000000000000000000000000000000000001,
	Legal = 0b0000000000000000000000000000000000000000000000000000000000000010,
	Web = 0b0000000000000000000000000000000000000000000000000000000000000100,
	Riot = 0b0000000000000000000000000000000000000000000000000000000000001000,
	Email = 0b0000000000000000000000000000000000000000000000000000000000010000,
	PgpFingerprint = 0b0000000000000000000000000000000000000000000000000000000000100000,
	Image = 0b0000000000000000000000000000000000000000000000000000000001000000,
	Twitter = 0b0000000000000000000000000000000000000000000000000000000010000000,
}

/// Wrapper type for `BitFlags<IdentityField>` that implements `Codec`.
#[derive(Clone, Copy, PartialEq, Default, RuntimeDebug)]
pub struct IdentityFields(pub(crate) BitFlags<IdentityField>);

impl MaxEncodedLen for IdentityFields {
	fn max_encoded_len() -> usize {
		u64::max_encoded_len()
	}
}

impl Eq for IdentityFields {}
impl Encode for IdentityFields {
	fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
		self.0.bits().using_encoded(f)
	}
}
impl Decode for IdentityFields {
	fn decode<I: codec::Input>(input: &mut I) -> sp_std::result::Result<Self, codec::Error> {
		let field = u64::decode(input)?;
		Ok(Self(<BitFlags<IdentityField>>::from_bits(field as u64).map_err(|_| "invalid value")?))
	}
}
impl TypeInfo for IdentityFields {
	type Identity = Self;

	fn type_info() -> Type {
		Type::builder()
			.path(Path::new("BitFlags", module_path!()))
			.type_params(vec![TypeParameter::new("T", Some(meta_type::<IdentityField>()))])
			.composite(Fields::unnamed().field(|f| f.ty::<u64>().type_name("IdentityField")))
	}
}

#[derive(
	CloneNoBound, Encode, Decode, Eq, MaxEncodedLen, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo,
)]
#[codec(mel_bound(FieldLimit: Get<u32>))]
#[cfg_attr(test, derive(frame_support::DefaultNoBound))]
#[scale_info(skip_type_params(FieldLimit))]
pub struct IdentityInfoSel<FieldLimit: Get<u32>> {
	/// Additional fields of the identity that are not catered for with the struct's explicit
	/// fields.
	pub additional: BoundedVec<(Data, Data), FieldLimit>,

	/// A reasonable display name for the controller of the account. This should be whatever it is
	/// that it is typically known as and should not be confusable with other entities, given
	/// reasonable context.
	///
	/// Stored as UTF-8.
	pub display: Data,

	/// The full legal name in the local jurisdiction of the entity. This might be a bit
	/// long-winded.
	///
	/// Stored as UTF-8.
	pub legal: Data,

	/// A representative website held by the controller of the account.
	///
	/// NOTE: `https://` is automatically prepended.
	///
	/// Stored as UTF-8.
	pub web: Data,

	/// The Riot/Matrix handle held by the controller of the account.
	///
	/// Stored as UTF-8.
	pub referalhash: Data,

	/// The email address of the controller of the account.
	///
	/// Stored as UTF-8.
	pub email: Data,

	/// The PGP/GPG public key of the controller of the account.
	pub pgp_fingerprint: Option<[u8; 20]>,

	pub account: Data,

	/// A graphic image representing the controller of the account. Should be a company,
	/// organization or project logo or a headshot in the case of a human.
	pub image: Data,

	/// The Twitter identity. The leading `@` character may be elided.
	pub passwordhash: Data,
}

/// NOTE: This is stored separately primarily to facilitate the addition of extra fields in a
/// backwards compatible way through a specialized `Decode` impl.
#[derive(
	CloneNoBound, Encode, Eq, MaxEncodedLen, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo,
)]
#[codec(mel_bound(
	Balance: Encode + Decode + MaxEncodedLen + Copy + Clone + Debug + Eq + PartialEq + Zero + Add,
	AccountId: Encode + Decode + MaxEncodedLen +  Clone + Debug + Eq + PartialEq ,
	MaxAdditionalFields: Get<u32>,
))]
#[scale_info(skip_type_params(MaxAdditionalFields))]
pub struct RegistrationSel<
	Balance: Encode + Decode + MaxEncodedLen + Copy + Clone + Debug + Eq + PartialEq,
	AccountId: Encode + Decode + MaxEncodedLen + Clone + Debug + Eq + PartialEq,
	MaxAdditionalFields: Get<u32>,
> {
	/// Amount held on deposit for this information.
	pub deposit: Balance,

	pub accountId: AccountId,

	// Public-key of user
	//pub account: AccountId,
	/// Information on the identity.
	pub info: IdentityInfoSel<MaxAdditionalFields>,
}

impl<
		Balance: Encode + Decode + MaxEncodedLen + Copy + Clone + Debug + Eq + PartialEq + Zero + Add,
		AccountId: Encode + Decode + MaxEncodedLen + Clone + Debug + Eq + PartialEq,
		MaxAdditionalFields: Get<u32>,
	> RegistrationSel<Balance, AccountId, MaxAdditionalFields>
{
	#[allow(dead_code)]
	pub(crate) fn total_deposit(&self) -> Balance {
		self.deposit
	}
}

impl<
		Balance: Encode + Decode + MaxEncodedLen + Copy + Clone + Debug + Eq + PartialEq,
		AccountId: Encode + Decode + MaxEncodedLen + Clone + Debug + Eq + PartialEq,
		MaxAdditionalFields: Get<u32>,
	> Decode for RegistrationSel<Balance, AccountId, MaxAdditionalFields>
{
	fn decode<I: codec::Input>(input: &mut I) -> sp_std::result::Result<Self, codec::Error> {
		let (deposit, accountId, info) = Decode::decode(&mut AppendZerosInput::new(input))?;
		Ok(Self { deposit, accountId, info })
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn manual_data_type_info() {
		let mut registry = scale_info::Registry::new();
		let type_id = registry.register_type(&scale_info::meta_type::<Data>());
		let registry: scale_info::PortableRegistry = registry.into();
		let type_info = registry.resolve(type_id.id()).unwrap();

		let check_type_info = |data: &Data| {
			let variant_name = match data {
				Data::None => "None".to_string(),
				Data::BlakeTwo256(_) => "BlakeTwo256".to_string(),
				Data::Sha256(_) => "Sha256".to_string(),
				Data::Keccak256(_) => "Keccak256".to_string(),
				Data::ShaThree256(_) => "ShaThree256".to_string(),
				Data::Raw(bytes) => format!("Raw{}", bytes.len()),
			};
			if let scale_info::TypeDef::Variant(variant) = type_info.type_def() {
				let variant = variant
					.variants()
					.iter()
					.find(|v| v.name() == &variant_name)
					.expect(&format!("Expected to find variant {}", variant_name));

				let field_arr_len = variant
					.fields()
					.first()
					.and_then(|f| registry.resolve(f.ty().id()))
					.map(|ty| {
						if let scale_info::TypeDef::Array(arr) = ty.type_def() {
							arr.len()
						} else {
							panic!("Should be an array type")
						}
					})
					.unwrap_or(0);

				let encoded = data.encode();
				assert_eq!(encoded[0], variant.index());
				assert_eq!(encoded.len() as u32 - 1, field_arr_len);
			} else {
				panic!("Should be a variant type")
			};
		};

		let mut data = vec![
			Data::None,
			Data::BlakeTwo256(Default::default()),
			Data::Sha256(Default::default()),
			Data::Keccak256(Default::default()),
			Data::ShaThree256(Default::default()),
		];

		// A Raw instance for all possible sizes of the Raw data
		for n in 0..32 {
			data.push(Data::Raw(vec![0u8; n as usize].try_into().unwrap()))
		}

		for d in data.iter() {
			check_type_info(d);
		}
	}
}