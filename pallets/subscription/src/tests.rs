#![cfg(test)]

use super::*;
use crate as pallet_subscription;

use frame_support::{assert_noop, assert_ok, parameter_types};

use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BadOrigin, BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Subscriptions: pallet_subscription::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type DbWeight = ();
	type Version = ();
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type PalletInfo = PalletInfo;
	type BlockWeights = ();
	type BlockLength = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type Balance = u64;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

parameter_types! {
	pub const BaseDeposit: u64 = 10;       // 258 bytes on-chain
	pub const MaxPublishing: u32 = 2;
	pub const MaximumNameLength: u32 = 10;
	pub const MaximumContractLength: u32 = 30;
	pub const MaximumDescriptionLength: u32 = 10;
	pub const DataDepositPerByte: u64 = 1;
}

impl pallet_subscription::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type BaseDeposit = BaseDeposit;
	type MaxPublishing = MaxPublishing;
	type MaximumNameLength = MaximumNameLength;
	type MaximumContractLength = MaximumContractLength;
	type MaximumDescriptionLength = MaximumDescriptionLength;
	type ApproveOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type ForceOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type OnSlash = ();
	type DataDepositPerByte = DataDepositPerByte;
	type WeightInfo = pallet_subscription::weights::SubstrateWeight<Test>;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_balances::GenesisConfig::<Test> {
		// Total issuance will be 222
		balances: vec![(0, 100), (1, 100), (2, 10), (3, 10), (4, 2)],
	}
	.assimilate_storage(&mut t)
	.unwrap();
	t.into()
}

type TestError = Error<Test>;

#[test]
fn request_approved_publisher_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(Subscriptions::request_approved_publisher(Origin::signed(4)));
		assert_eq!(Subscriptions::approved_publisher(4).unwrap(), PublisherStatus::Requested);
	});
}

#[test]
fn request_new_approved_publisher_fails() {
	new_test_ext().execute_with(|| {
		assert_ok!(Subscriptions::request_approved_publisher(Origin::signed(4)));
		assert_noop!(
			Subscriptions::request_approved_publisher(Origin::signed(4)),
			TestError::AlreadyApprovedOrRequested
		);
		assert_eq!(Subscriptions::approved_publisher(4).unwrap(), PublisherStatus::Requested);
	});
}

#[test]
fn approve_publisher_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(Subscriptions::request_approved_publisher(Origin::signed(4)));
		assert_ok!(Subscriptions::approve_publisher(Origin::root(), 4));
		assert_eq!(Subscriptions::approved_publisher(4).unwrap(), PublisherStatus::Approved);
	});
}

#[test]
fn approve_unrequest_publisher_fails() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Subscriptions::approve_publisher(Origin::root(), 4),
			TestError::NotRequestForApproval
		);
		assert_eq!(Subscriptions::approved_publisher(4), None);
		assert_ok!(Subscriptions::request_approved_publisher(Origin::signed(4)));
		assert_noop!(Subscriptions::approve_publisher(Origin::signed(4), 4), BadOrigin);
	});
}

#[test]
fn approved_request_for_new_approval_fails() {
	new_test_ext().execute_with(|| {
		assert_ok!(Subscriptions::request_approved_publisher(Origin::signed(4)));
		assert_ok!(Subscriptions::approve_publisher(Origin::root(), 4));
		assert_noop!(
			Subscriptions::approve_publisher(Origin::root(), 4),
			TestError::AlreadyApprovedPublisher
		);
	});
}

#[test]
fn revoke_approved_publisher_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(Subscriptions::request_approved_publisher(Origin::signed(4)));
		assert_ok!(Subscriptions::approve_publisher(Origin::root(), 4));
		assert_ok!(Subscriptions::revoke_publisher(Origin::root(), 4));
		assert_eq!(Subscriptions::approved_publisher(4), None);
	});
}

#[test]
fn revoke_unrequest_publisher_fails() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Subscriptions::revoke_publisher(Origin::root(), 3),
			TestError::NotRequestForApproval
		);
	})
}

#[test]
fn revoke_unapproved_publisher_fails() {
	new_test_ext().execute_with(|| {
		assert_ok!(Subscriptions::request_approved_publisher(Origin::signed(3)));
		assert_noop!(
			Subscriptions::revoke_publisher(Origin::root(), 3),
			TestError::NotApprovedPublisher
		);
	});
}

#[test]
fn not_root_revoke_publisher_fails() {
	new_test_ext().execute_with(|| {
		assert_ok!(Subscriptions::request_approved_publisher(Origin::signed(3)));
		assert_ok!(Subscriptions::approve_publisher(Origin::root(), 3));
		assert_noop!(Subscriptions::revoke_publisher(Origin::signed(0), 3), BadOrigin);
		assert_eq!(Subscriptions::approved_publisher(3).unwrap(), PublisherStatus::Approved);
	});
}

#[test]
fn publish_service_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Subscriptions::request_approved_publisher(Origin::signed(0)));
		assert_ok!(Subscriptions::approve_publisher(Origin::root(), 0));
		let name: Vec<u8> = b"nobody".to_vec().try_into().unwrap();
		let description: Vec<u8> = b"no desc".to_vec().try_into().unwrap();
		assert_ok!(Subscriptions::publish_service(
			Origin::signed(0),
			11,
			name.clone(),
			description.clone(),
			None
		));
		assert_eq!(Subscriptions::service_count(), 1);
		assert_eq!(Subscriptions::services(0).is_some(), true);
		assert_eq!(Subscriptions::service_descriptions(0).is_some(), true);
		let deposit: u64 = 10 + (name.len() + description.len()) as u64;
		assert_eq!(Balances::reserved_balance(0), deposit);
		assert_eq!(Balances::free_balance(0), 100 - deposit);
		// assert_eq!(
		//   Subscriptions::services(0).unwrap(),
		//   Service {
		//     publisher: 0,
		//     cost: 11,
		//     bond: deposit,
		//     name: b"nobody".to_vec().try_into().unwrap(),
		//     contract: b"https://contract-link".to_vec().try_into().unwrap(),
		// 		maybe_periodic: None,
		// 		status: ServiceStatus::Published{ on: 1 }
		//   }
		// )
	});
}

#[test]
fn publish_service_fails() {
	new_test_ext().execute_with(|| {
		let name: Vec<u8> = b"nobody".to_vec().try_into().unwrap();
		let description: Vec<u8> = b"no desc".to_vec().try_into().unwrap();
		assert_noop!(
			Subscriptions::publish_service(
				Origin::signed(1),
				22,
				name.clone(),
				description.clone(),
				None
			),
			TestError::NotRequestForApproval
		);
		assert_ok!(Subscriptions::request_approved_publisher(Origin::signed(1)));
		assert_noop!(
			Subscriptions::publish_service(
				Origin::signed(1),
				22,
				name.clone(),
				description.clone(),
				None
			),
			TestError::NotApprovedPublisher
		);
		assert_ok!(Subscriptions::approve_publisher(Origin::root(), 1));
		let too_long_name: Vec<u8> = b"nobody too long".to_vec().try_into().unwrap();
		let too_long_desc: Vec<u8> = b"no desc too long".to_vec().try_into().unwrap();
		assert_noop!(
			Subscriptions::publish_service(
				Origin::signed(1),
				22,
				too_long_name.clone(),
				description.clone(),
				None
			),
			TestError::NameTooLong
		);
		assert_noop!(
			Subscriptions::publish_service(
				Origin::signed(1),
				22,
				name.clone(),
				too_long_desc.clone(),
				None
			),
			TestError::DescriptionTooLong
		);
		assert_ok!(Subscriptions::request_approved_publisher(Origin::signed(2)));
		assert_ok!(Subscriptions::approve_publisher(Origin::root(), 2));
		assert_noop!(
			Subscriptions::publish_service(
				Origin::signed(2),
				22,
				name.clone(),
				description.clone(),
				None
			),
			TestError::InsufficientPublisherBalance
		);
	});
}

#[test]
fn subscribe_service_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Subscriptions::request_approved_publisher(Origin::signed(0)));
		assert_ok!(Subscriptions::approve_publisher(Origin::root(), 0));
		let name: Vec<u8> = b"nobody".to_vec().try_into().unwrap();
		let description: Vec<u8> = b"no desc".to_vec().try_into().unwrap();
		let publisher_bond: u64 = 10 + (name.len() + description.len()) as u64;
		assert_ok!(Subscriptions::publish_service(
			Origin::signed(0),
			10,
			name.clone(),
			description.clone(),
			None
		));
		assert_ok!(Subscriptions::subscribe_service(Origin::signed(1), 0));
		assert_eq!(Balances::free_balance(0), 100 - publisher_bond + 10);
		assert_eq!(Balances::free_balance(1), 100 - 10);
		assert_eq!(
			Subscriptions::subscriptions(0, 1).unwrap(),
			Subscription { start_on: 1, expire_on: None, active: true }
		)
	});
}

#[test]
fn subscribe_service_fails() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Subscriptions::subscribe_service(Origin::signed(4), 0),
			TestError::ServiceNotFound
		);
		assert_ok!(Subscriptions::request_approved_publisher(Origin::signed(1)));
		assert_ok!(Subscriptions::approve_publisher(Origin::root(), 1));
		let name: Vec<u8> = b"nobody".to_vec().try_into().unwrap();
		let description: Vec<u8> = b"no desc".to_vec().try_into().unwrap();
		assert_ok!(Subscriptions::publish_service(
			Origin::signed(1),
			10,
			name.clone(),
			description.clone(),
			None
		));
		assert_ok!(Subscriptions::subscribe_service(Origin::signed(0), 0));
		assert_noop!(
			Subscriptions::subscribe_service(Origin::signed(0), 0),
			TestError::AlreadySubscribed
		);
		assert_noop!(
			Subscriptions::subscribe_service(Origin::signed(4), 0),
			TestError::InsufficientSubscriberBalance
		);
	});
}

#[test]
fn unsubscribe_service_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(Subscriptions::request_approved_publisher(Origin::signed(0)));
		assert_ok!(Subscriptions::approve_publisher(Origin::root(), 0));
		let name: Vec<u8> = b"nobody".to_vec().try_into().unwrap();
		let description: Vec<u8> = b"no desc".to_vec().try_into().unwrap();
		assert_ok!(Subscriptions::publish_service(
			Origin::signed(0),
			10,
			name.clone(),
			description.clone(),
			None
		));
		assert_ok!(Subscriptions::subscribe_service(Origin::signed(1), 0));
		assert_ok!(Subscriptions::unsubscribe_service(Origin::signed(1), 0));
		assert_eq!(Subscriptions::subscriptions(0, 1).unwrap().active, false);
	});
}

#[test]
fn unsubscribe_service_fails() {
	new_test_ext().execute_with(|| {
		assert_ok!(Subscriptions::request_approved_publisher(Origin::signed(0)));
		assert_ok!(Subscriptions::approve_publisher(Origin::root(), 0));
		let name: Vec<u8> = b"nobody".to_vec().try_into().unwrap();
		let description: Vec<u8> = b"no desc".to_vec().try_into().unwrap();
		assert_ok!(Subscriptions::publish_service(
			Origin::signed(0),
			10,
			name.clone(),
			description.clone(),
			None
		));
		assert_ok!(Subscriptions::subscribe_service(Origin::signed(1), 0));
		assert_ok!(Subscriptions::unsubscribe_service(Origin::signed(1), 0));
		assert_noop!(
			Subscriptions::unsubscribe_service(Origin::signed(1), 0),
			TestError::SubscriptionInactive
		);
		assert_ok!(Subscriptions::unpublish_service(Origin::root(), 0));
		assert_noop!(
			Subscriptions::subscribe_service(Origin::signed(1), 0),
			TestError::ServiceAlreadyUnpublished
		);
	});
}
