// Copyright 2020-2023 Manta Network.
// This file is part of Manta.
//
// Manta is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Manta is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Manta.  If not, see <http://www.gnu.org/licenses/>.

use super::{Call, Event, Origin, ParachainInfo, ParachainSystem, PolkadotXcm, Runtime};

use frame_support::{match_types, parameter_types, traits::Nothing, weights::Weight};

use frame_system::EnsureRoot;
use manta_primitives::{types::AccountId, xcm::MultiNativeAsset};

use xcm::latest::prelude::*;
use xcm_builder::{
    AllowKnownQueryResponses, AllowSubscriptionsFrom, AllowUnpaidExecutionFrom, EnsureXcmOrigin,
    FixedRateOfFungible, FixedWeightBounds, LocationInverter, ParentAsSuperuser, TakeWeightCredit,
};
use xcm_executor::{Config, XcmExecutor};

parameter_types! {
    pub Ancestry: MultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();
    pub SelfReserve: MultiLocation = MultiLocation::new(1, X1(Parachain(ParachainInfo::parachain_id().into())));
}

/// This is the type to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`.
/// It uses some Rust magic macro to do the pattern matching sequentially.
/// There is an `OriginKind` which can biases the kind of local `Origin` it will become.
pub type XcmOriginToCallOrigin = (
    // Superuser converter for the Relay-chain (Parent) location. This will allow it to issue a
    // transaction from the Root origin.
    ParentAsSuperuser<Origin>,
);

parameter_types! {
    /// One XCM operation is 1_000_000_000 weight - almost certainly a conservative estimate.
    pub UnitWeightCost: Weight = 1_000_000_000;
    /// Used in native traders
    /// This might be able to skipped.
    /// We have to use `here()` because of reanchoring logic
    pub ParaTokenPerSecond: (AssetId, u128) = (Concrete(MultiLocation::here()), 1_000_000_000);
    pub const MaxInstructions: u32 = 100;
}

match_types! {
    pub type ParentLocation: impl Contains<MultiLocation> = {
        MultiLocation { parents: 1, interior: Here }
    };
}
match_types! {
    pub type ParentOrSiblings: impl Contains<MultiLocation> = {
        MultiLocation { parents: 1, interior: Here } |
        MultiLocation { parents: 1, interior: X1(_) }
    };
}

pub type Barrier = (
    // Allows local origin messages which call weight_credit >= weight_limit.
    TakeWeightCredit,
    // Parent root gets free execution
    AllowUnpaidExecutionFrom<ParentLocation>,
    // Expected responses are OK.
    // Allows `Pending` or `VersionNotifier` query responses.
    AllowKnownQueryResponses<PolkadotXcm>,
    // Subscriptions for version tracking are OK.
    // Allows execution of `SubscribeVersion` or `UnsubscribeVersion` instruction,
    // from parent or sibling chains.
    AllowSubscriptionsFrom<ParentOrSiblings>,
);

pub struct XcmExecutorConfig;
impl Config for XcmExecutorConfig {
    type Call = Call;
    type XcmSender = XcmRouter;
    // Defines how to Withdraw and Deposit instruction work
    type AssetTransactor = (); // MultiAssetTransactor;
    type OriginConverter = XcmOriginToCallOrigin;
    // Combinations of (Location, Asset) pairs which we trust as reserves.
    type IsReserve = MultiNativeAsset;
    type IsTeleporter = ();
    type LocationInverter = LocationInverter<Ancestry>;
    type Barrier = Barrier;
    type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
    // Trader is the means to purchasing weight credit for XCM execution.
    // We define a trader that will charge parachain's native currency, who's `MultiLocation`
    // is defined in `SelfReserve`.
    type Trader = FixedRateOfFungible<ParaTokenPerSecond, ()>;
    type ResponseHandler = PolkadotXcm;
    type AssetTrap = PolkadotXcm;
    type AssetClaims = PolkadotXcm;
    // This is needed for the version change notifier work
    type SubscriptionService = PolkadotXcm;
}

/// No one is allowed to dispatch XCM sends/executions.
pub type LocalOriginToLocation = ();

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = (
    // Only use UMP to communicate with the relay chain:
    cumulus_primitives_utility::ParentAsUmp<ParachainSystem, PolkadotXcm>,
);

impl pallet_xcm::Config for Runtime {
    const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
    type Origin = Origin;
    type Call = Call;
    type Event = Event;
    type SendXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
    type XcmRouter = XcmRouter;
    type ExecuteXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
    /// This means that no location will pass XcmExecuteFilter, so a dispatched `execute` message will be filtered.
    /// This shouldn't be reachable since `LocalOriginToLocation = ();`, but let's be on the safe side.
    type XcmExecuteFilter = Nothing;
    type XcmExecutor = XcmExecutor<XcmExecutorConfig>;
    type XcmTeleportFilter = Nothing;
    type XcmReserveTransferFilter = Nothing;
    type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
    type LocationInverter = LocationInverter<Ancestry>;
    type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
}

impl cumulus_pallet_dmp_queue::Config for Runtime {
    type Event = Event;
    type XcmExecutor = XcmExecutor<XcmExecutorConfig>;
    type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
}
