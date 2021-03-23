use crate::{*, self as multi_phase};
pub use frame_support::{assert_noop, assert_ok};
use frame_support::{parameter_types, weights::Weight};
use sp_core::{
	offchain::{
		testing::{TestOffchainExt, TestTransactionPoolExt},
		OffchainDbExt, OffchainWorkerExt, TransactionPoolExt,
	},
	H256
};
use sp_runtime::{traits::{BlakeTwo256, IdentityLookup}, PerU16};

pub type Header = sp_runtime::generic::Header<u32, BlakeTwo256>;
pub type Block = sp_runtime::generic::Block<Header, UncheckedExtrinsic>;
pub type UncheckedExtrinsic = sp_runtime::generic::UncheckedExtrinsic<AccountId, Call, (), ()>;

frame_support::construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Pallet, Call, Event<T>, Config},
		Balances: pallet_balances::{Pallet, Call, Event<T>, Config<T>},
		ElectionProviderMultiPhase: multi_phase::{Pallet, Call, Event<T>},
	}
);

pub(crate) type Balance = u128;
pub(crate) type AccountId = sp_runtime::AccountId32;
pub(crate) type VoterIndex = u32;
pub(crate) type TargetIndex = u16;

sp_npos_elections::generate_solution_type!(
	#[compact]
	pub struct TestCompact::<VoterIndex, TargetIndex, PerU16>(16)
);

const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
	pub BlockWeights: frame_system::limits::BlockWeights = frame_system::limits::BlockWeights
		::with_sensible_defaults(2 * frame_support::weights::constants::WEIGHT_PER_SECOND, NORMAL_DISPATCH_RATIO);
}

impl frame_system::Config for Runtime {
	type SS58Prefix = ();
	type BaseCallFilter = ();
	type Origin = Origin;
	type Index = u32;
	type BlockNumber = u32;
	type Call = Call;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = ();
	type DbWeight = ();
	type BlockLength = ();
	type BlockWeights = BlockWeights;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
}

impl pallet_balances::Config for Runtime {
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type MaxLocks = ();
	type WeightInfo = ();
}

parameter_types! {
	pub static Fallback: FallbackStrategy = FallbackStrategy::OnChain;
	pub static MinerMaxWeight: Weight = BlockWeights::get().max_block;
	pub static MinerMaxLength: u32 = 22 * 1024;
	pub static MinerMaxIterations: u32 = 20;
	pub static MockWeightInfo: bool = false;
}

// Hopefully this won't be too much of a hassle to maintain.
pub struct DualMockWeightInfo;
impl multi_phase::weights::WeightInfo for DualMockWeightInfo {
	fn on_initialize_nothing() -> Weight {
		if MockWeightInfo::get() {
			Zero::zero()
		} else {
			<() as multi_phase::weights::WeightInfo>::on_initialize_nothing()
		}
	}
	fn on_initialize_open_signed() -> Weight {
		if MockWeightInfo::get() {
			Zero::zero()
		} else {
			<() as multi_phase::weights::WeightInfo>::on_initialize_open_signed()
		}
	}
	fn on_initialize_open_unsigned_with_snapshot() -> Weight {
		if MockWeightInfo::get() {
			Zero::zero()
		} else {
			<() as multi_phase::weights::WeightInfo>::on_initialize_open_unsigned_with_snapshot()
		}
	}
	fn on_initialize_open_unsigned_without_snapshot() -> Weight {
		if MockWeightInfo::get() {
			Zero::zero()
		} else {
			<() as multi_phase::weights::WeightInfo>::on_initialize_open_unsigned_without_snapshot()
		}
	}
	fn elect_queued() -> Weight {
		if MockWeightInfo::get() {
			Zero::zero()
		} else {
			<() as multi_phase::weights::WeightInfo>::elect_queued()
		}
	}
	fn submit_unsigned(v: u32, t: u32, a: u32, d: u32) -> Weight {
		if MockWeightInfo::get() {
			0
		} else {
			<() as multi_phase::weights::WeightInfo>::submit_unsigned(v, t, a, d)
		}
	}
	fn feasibility_check(v: u32, t: u32, a: u32, d: u32) -> Weight {
		if MockWeightInfo::get() {
			0
		} else {
			<() as multi_phase::weights::WeightInfo>::feasibility_check(v, t, a, d)
		}
	}
}

impl crate::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type SignedPhase = ();
	type UnsignedPhase = ();
	type SolutionImprovementThreshold = ();
	type MinerMaxIterations = MinerMaxIterations;
	type MinerMaxWeight = MinerMaxWeight;
	type MinerMaxLength = MinerMaxLength;
	type MinerTxPriority = ();
	type DataProvider = ();
	type WeightInfo = DualMockWeightInfo;
	type BenchmarkingConfig = ();
	type OnChainAccuracy = Perbill;
	type Fallback = Fallback;
	type CompactSolution = TestCompact;
}

impl<LocalCall> frame_system::offchain::SendTransactionTypes<LocalCall> for Runtime
where
	Call: From<LocalCall>,
{
	type OverarchingCall = Call;
	type Extrinsic = Extrinsic;
}
pub type Extrinsic = sp_runtime::testing::TestXt<Call, ()>;

#[tokio::test]
async fn polkadot() {
	use remote_externalities::{Mode, CacheConfig, OfflineConfig};
	sp_tracing::try_init_simple();
	let mut ext = remote_externalities::Builder::new()
	.mode(Mode::Offline(OfflineConfig {
		cache: CacheConfig {
			name: "kusama@0x03822d2fdc281d420d3136afb561c5977f08709da4c7c91841c5737b464f4ad4"
				.into(),
			directory: ".".into(),
		}
	}))
	// .mode(Mode::Online(remote_externalities::OnlineConfig {
	// 	uri: "http://substrate-archive-0.parity-vpn.parity.io:9933/".into(),
	// 	at: Some(
	// 		hex_literal::hex!["03822d2fdc281d420d3136afb561c5977f08709da4c7c91841c5737b464f4ad4"]
	// 			.into(),
	// 	),
	// 	cache: Some(CacheConfig {
	// 		name: "kusama@0x03822d2fdc281d420d3136afb561c5977f08709da4c7c91841c5737b464f4ad4"
	// 			.into(),
	// 		directory: ".".into(),
	// 	}),
	// 	..Default::default()
	// }))
	.build()
	.await
	.unwrap();

	let (offchain, offchain_state) = TestOffchainExt::new();
	let (pool, pool_state) = TestTransactionPoolExt::new();

	let mut seed = [0_u8; 32];
	seed[0..4].copy_from_slice(&20u32.to_le_bytes());
	// ----------------------- ^^ random number of the offchain env.
	offchain_state.write().seed = seed;
	ext.register_extension(OffchainDbExt::new(offchain.clone()));
	ext.register_extension(OffchainWorkerExt::new(offchain));
	ext.register_extension(TransactionPoolExt::new(pool));

	ext.execute_with(|| {
		// Kill the QueuedElected storage item if needed.
		<multi_phase::QueuedSolution<Runtime>>::kill();
		let result = ElectionProviderMultiPhase::mine_check_and_submit().unwrap();
	});
}
