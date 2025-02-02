use super::*;

use crate as kitties;
use std::cell::RefCell;
use sp_core::H256;
use frame_support::{parameter_types, assert_ok, assert_noop};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup}, testing::Header,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Module, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
		NFT: orml_nft::{Module, Storage},
		KittiesModule: kitties::{Module, Call, Storage, Event<T>, Config},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Test {
	type BaseCallFilter = ();
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
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
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
}
impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type Balance = u64;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

impl orml_nft::Config for Test {
    type ClassId = u32;
	type TokenId = u32;
	type ClassData = ();
	type TokenData = Kitty;
}

thread_local! {
    static RANDOM_PAYLOAD: RefCell<H256> = RefCell::new(Default::default());
}

pub struct MockRandom;

impl Randomness<H256> for MockRandom {
    fn random(_subject: &[u8]) -> H256 {
        RANDOM_PAYLOAD.with(|v| *v.borrow())
    }
}

fn set_random(val: H256) {
    RANDOM_PAYLOAD.with(|v| *v.borrow_mut() = val)
}

impl Config for Test {
    type Event = Event;
    type Randomness = MockRandom;
    type Currency = Balances;
    type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

    pallet_balances::GenesisConfig::<Test>{
		balances: vec![(200, 500)],
    }.assimilate_storage(&mut t).unwrap();

    crate::GenesisConfig::default().assimilate_storage::<Test>(&mut t).unwrap();

    let mut t: sp_io::TestExternalities = t.into();

    t.execute_with(|| System::set_block_number(1) );
    t
}

fn last_event() -> Event {
    System::events().last().unwrap().event.clone()
}

#[test]
fn can_create() {
    new_test_ext().execute_with(|| {
        assert_ok!(KittiesModule::create(Origin::signed(100)));

        let kitty = Kitty([59, 250, 138, 82, 209, 39, 141, 109, 163, 238, 183, 145, 235, 168, 18, 122]);

        assert_eq!(KittiesModule::kitties(&100, 0), Some(kitty.clone()));
        assert_eq!(NFT::tokens(KittiesModule::class_id(), 0).unwrap().owner, 100);

        assert_eq!(last_event(), Event::kitties(crate::Event::<Test>::KittyCreated(100, 0, kitty)));
    });
}

#[test]
fn gender() {
    assert_eq!(Kitty([0; 16]).gender(), KittyGender::Male);
    assert_eq!(Kitty([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]).gender(), KittyGender::Female);
}

#[test]
fn can_breed() {
    new_test_ext().execute_with(|| {
        assert_ok!(KittiesModule::create(Origin::signed(100)));

        set_random(H256::from([2; 32]));

        assert_ok!(KittiesModule::create(Origin::signed(100)));

        assert_noop!(KittiesModule::breed(Origin::signed(100), 0, 11), Error::<Test>::InvalidKittyId);
        assert_noop!(KittiesModule::breed(Origin::signed(100), 0, 0), Error::<Test>::SameGender);
        assert_noop!(KittiesModule::breed(Origin::signed(101), 0, 1), Error::<Test>::InvalidKittyId);

        assert_ok!(KittiesModule::breed(Origin::signed(100), 0, 1));

        let kitty = Kitty([187, 250, 235, 118, 211, 247, 237, 253, 187, 239, 191, 185, 239, 171, 211, 122]);

        assert_eq!(KittiesModule::kitties(&100, 2), Some(kitty.clone()));
        assert_eq!(NFT::tokens(KittiesModule::class_id(), 2).unwrap().owner, 100);

        assert_eq!(last_event(), Event::kitties(crate::Event::<Test>::KittyBred(100u64, 2u32, kitty)));
    });
}

#[test]
fn can_transfer() {
    // TODO: update this test to check the updated behaviour regards to KittyPrices
    new_test_ext().execute_with(|| {
         //Setup
         assert_ok!(KittiesModule::create(Origin::signed(100)));
         assert_eq!(KittiesModule::kitty_prices(0), None); 
 
         // Call functions 
         //Only Owner
         assert_noop!(KittiesModule::transfer(Origin::signed(101), 300, 0), orml_nft::Error::<Test>::NoPermission);
         assert_noop!(KittiesModule::transfer(Origin::signed(300), 300, 0), orml_nft::Error::<Test>::NoPermission);
         assert_noop!(KittiesModule::set_price(Origin::signed(101), 0, Some(400)), Error::<Test>::NotOwner);
         assert_noop!(KittiesModule::set_price(Origin::signed(100), 1, Some(400)), Error::<Test>::NotOwner);
         assert_noop!(KittiesModule::transfer(Origin::signed(100), 100, 1), orml_nft::Error::<Test>::TokenNotFound);
 
         //Storage 
         //price
         assert_ok!(KittiesModule::set_price(Origin::signed(100), 0, Some(400)));
         assert_eq!(KittiesModule::kitty_prices(0), Some(400));
         assert_eq!(last_event(), Event::kitties(RawEvent::KittyPriceUpdated(100, 0, Some(400))));
         //transfer
         assert_ok!(KittiesModule::transfer(Origin::signed(100), 300, 0));
         assert_eq!(KittiesModule::kitty_prices(0), None);
 
         assert_eq!(NFT::tokens(KittiesModule::class_id(), 0).unwrap().owner, 300);
         assert!(KittiesModule::kitties(&300,0).is_some());
 
         //check balances
         assert_eq!(Balances::free_balance(100), 0);
         assert_eq!(Balances::free_balance(300), 0);
 
         assert_eq!(last_event(), Event::kitties(RawEvent::KittyTransferred(100, 300, 0)));
    });
}

#[test]
fn handle_self_transfer() {
    new_test_ext().execute_with(|| {
        assert_ok!(KittiesModule::create(Origin::signed(100)));

        System::reset_events();

        assert_noop!(KittiesModule::transfer(Origin::signed(100), 100, 1), orml_nft::Error::<Test>::TokenNotFound);

        assert_ok!(KittiesModule::transfer(Origin::signed(100), 100, 0));

        assert_eq!(NFT::tokens(KittiesModule::class_id(), 0).unwrap().owner, 100);

        // no transfer event because no actual transfer is executed
        assert_eq!(System::events().len(), 0);
    });
}

#[test]
fn can_set_price() {
    new_test_ext().execute_with(|| {
        // Setup
        assert_ok!(KittiesModule::create(Origin::signed(100)));
        System::reset_events();
        //Call Functions
        assert_noop!(KittiesModule::set_price(Origin::signed(101), 0, Some(400)), Error::<Test>::NotOwner);
        assert_noop!(KittiesModule::set_price(Origin::signed(100), 1, Some(400)), Error::<Test>::NotOwner);

        //Storage
        assert_eq!(KittiesModule::kitty_prices(0), None); 
        assert_ok!(KittiesModule::set_price(Origin::signed(100), 0, Some(400)));
        assert!(KittiesModule::kitties(&100,0).is_some()); 
        assert_eq!(KittiesModule::kitty_prices(0), Some(400));
        assert_eq!(last_event(), Event::kitties(RawEvent::KittyPriceUpdated(100, 0, Some(400))));
    });
}

#[test]
fn can_buy() {
    new_test_ext().execute_with(|| {
     //Setup - Create Kitty
     assert_ok!(KittiesModule::create(Origin::signed(100)));
     System::reset_events();

     assert_ok!(KittiesModule::set_price(Origin::signed(100), 0, Some(400)));
     assert_eq!(KittiesModule::kitty_prices(0),Some(400)); //check price
     
     //Call Functions
     assert_noop!(KittiesModule::buy(Origin::signed(200), 100, 0,399), Error::<Test>::PriceTooLow);
     assert_noop!(KittiesModule::buy(Origin::signed(200), 100, 1, 400), Error::<Test>::NotForSale);


     //Storage
     assert_ok!(KittiesModule::buy(Origin::signed(200), 100, 0, 400));
     assert!(KittiesModule::kitties(&200,0).is_some());
     assert_eq!(KittiesModule::kitty_prices(0),None);

     assert_eq!(Balances::free_balance(100), 400);
     assert_eq!(Balances::free_balance(200), 100);

     assert_eq!(last_event(), Event::kitties(RawEvent::KittySold(100, 200, 0, 400)));
    });
}
