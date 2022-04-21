
        fn disapprove_proposal(p: u32, ) -> Weight;
}

/// Weights for pallet_collective using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_collective::WeightInfo for SubstrateWeight<T> {
        // Storage: Council Members (r:1 w:1)
        // Storage: Council Proposals (r:1 w:0)
        // Storage: Council Voting (r:100 w:100)
        // Storage: Council Prime (r:0 w:1)
        fn set_members(m: u32, _n: u32, p: u32, ) -> Weight {
                (0 as Weight)
                        // Standard Error: 25_000
                        .saturating_add((34_937_000 as Weight).saturating_mul(m as Weight))
                        // Standard Error: 25_000
                        .saturating_add((41_946_000 as Weight).saturating_mul(p as Weight))
                        .saturating_add(T::DbWeight::get().reads(2 as Weight))
                        .saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(p as Weight)))
                        .saturating_add(T::DbWeight::get().writes(2 as Weight))
                        .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(p as Weight)))
        }
        // Storage: Council Members (r:1 w:0)
        fn execute(b: u32, m: u32, ) -> Weight {
                (37_040_000 as Weight)
                        // Standard Error: 0
                        .saturating_add((4_000 as Weight).saturating_mul(b as Weight))
                        // Standard Error: 0
                        .saturating_add((187_000 as Weight).saturating_mul(m as Weight))
                        .saturating_add(T::DbWeight::get().reads(1 as Weight))
        }
        // Storage: Council Members (r:1 w:0)
        // Storage: Council ProposalOf (r:1 w:0)
        fn propose_execute(b: u32, m: u32, ) -> Weight {
                (45_615_000 as Weight)
                        // Standard Error: 1_000
                        .saturating_add((7_000 as Weight).saturating_mul(b as Weight))
                        // Standard Error: 10_000
                        .saturating_add((337_000 as Weight).saturating_mul(m as Weight))
                        .saturating_add(T::DbWeight::get().reads(2 as Weight))
        }
        // Storage: Council Members (r:1 w:0)
        // Storage: Council ProposalOf (r:1 w:1)
        // Storage: Council Proposals (r:1 w:1)
        // Storage: Council ProposalCount (r:1 w:1)
        // Storage: Council Voting (r:0 w:1)
        fn propose_proposed(b: u32, m: u32, p: u32, ) -> Weight {
                (53_342_000 as Weight)
                        // Standard Error: 0
                        .saturating_add((15_000 as Weight).saturating_mul(b as Weight))
                        // Standard Error: 3_000
                        .saturating_add((213_000 as Weight).saturating_mul(m as Weight))
                        // Standard Error: 3_000
                        .saturating_add((520_000 as Weight).saturating_mul(p as Weight))
                        .saturating_add(T::DbWeight::get().reads(4 as Weight))
                        .saturating_add(T::DbWeight::get().writes(4 as Weight))
        }
        // Storage: Council Members (r:1 w:0)
        // Storage: Council Voting (r:1 w:1)
        fn vote(m: u32, ) -> Weight {
                (66_122_000 as Weight)
                        // Standard Error: 4_000
                        .saturating_add((431_000 as Weight).saturating_mul(m as Weight))
                        .saturating_add(T::DbWeight::get().reads(2 as Weight))
                        .saturating_add(T::DbWeight::get().writes(1 as Weight))
        }
        // Storage: Council Voting (r:1 w:1)
        // Storage: Council Members (r:1 w:0)
        // Storage: Council Proposals (r:1 w:1)
        // Storage: Council ProposalOf (r:0 w:1)
        fn close_early_disapproved(m: u32, p: u32, ) -> Weight {
                (65_110_000 as Weight)
                        // Standard Error: 3_000
                        .saturating_add((358_000 as Weight).saturating_mul(m as Weight))
                        // Standard Error: 3_000
                        .saturating_add((433_000 as Weight).saturating_mul(p as Weight))
                        .saturating_add(T::DbWeight::get().reads(3 as Weight))
                        .saturating_add(T::DbWeight::get().writes(3 as Weight))
        }
        // Storage: Council Voting (r:1 w:1)
        // Storage: Council Members (r:1 w:0)
        // Storage: Council ProposalOf (r:1 w:1)
        // Storage: Council Proposals (r:1 w:1)
        fn close_early_approved(b: u32, m: u32, p: u32, ) -> Weight {
                (76_399_000 as Weight)
                        // Standard Error: 0
                        .saturating_add((12_000 as Weight).saturating_mul(b as Weight))
                        // Standard Error: 5_000
                        .saturating_add((378_000 as Weight).saturating_mul(m as Weight))
                        // Standard Error: 5_000
                        .saturating_add((519_000 as Weight).saturating_mul(p as Weight))
                        .saturating_add(T::DbWeight::get().reads(4 as Weight))
                        .saturating_add(T::DbWeight::get().writes(3 as Weight))
        }
        // Storage: Council Voting (r:1 w:1)
        // Storage: Council Members (r:1 w:0)
        // Storage: Council Prime (r:1 w:0)
        // Storage: Council Proposals (r:1 w:1)
        // Storage: Council ProposalOf (r:0 w:1)
        fn close_disapproved(m: u32, p: u32, ) -> Weight {
                (71_329_000 as Weight)
                        // Standard Error: 3_000
                        .saturating_add((369_000 as Weight).saturating_mul(m as Weight))
                        // Standard Error: 3_000
                        .saturating_add((444_000 as Weight).saturating_mul(p as Weight))
                        .saturating_add(T::DbWeight::get().reads(4 as Weight))
                        .saturating_add(T::DbWeight::get().writes(3 as Weight))
        }
        // Storage: Council Voting (r:1 w:1)
        // Storage: Council Members (r:1 w:0)
        // Storage: Council Prime (r:1 w:0)
        // Storage: Council ProposalOf (r:1 w:1)
        // Storage: Council Proposals (r:1 w:1)
        fn close_approved(b: u32, m: u32, p: u32, ) -> Weight {
                (80_242_000 as Weight)
                        // Standard Error: 0
                        .saturating_add((12_000 as Weight).saturating_mul(b as Weight))
                        // Standard Error: 3_000
                        .saturating_add((399_000 as Weight).saturating_mul(m as Weight))
                        // Standard Error: 3_000
                        .saturating_add((528_000 as Weight).saturating_mul(p as Weight))
                        .saturating_add(T::DbWeight::get().reads(5 as Weight))
                        .saturating_add(T::DbWeight::get().writes(3 as Weight))
        }
        // Storage: Council Proposals (r:1 w:1)
        // Storage: Council Voting (r:0 w:1)
        // Storage: Council ProposalOf (r:0 w:1)
        fn disapprove_proposal(p: u32, ) -> Weight {
                (40_464_000 as Weight)
                        // Standard Error: 2_000
                        .saturating_add((495_000 as Weight).saturating_mul(p as Weight))
                        .saturating_add(T::DbWeight::get().reads(1 as Weight))
                        .saturating_add(T::DbWeight::get().writes(3 as Weight))
        }
}

// For backwards compatibility and tests
impl WeightInfo for () {
        // Storage: Council Members (r:1 w:1)
        // Storage: Council Proposals (r:1 w:0)
        // Storage: Council Voting (r:100 w:100)
        // Storage: Council Prime (r:0 w:1)
        fn set_members(m: u32, _n: u32, p: u32, ) -> Weight {
                (0 as Weight)
                        // Standard Error: 25_000
                        .saturating_add((34_937_000 as Weight).saturating_mul(m as Weight))
                        // Standard Error: 25_000
                        .saturating_add((41_946_000 as Weight).saturating_mul(p as Weight))
                        .saturating_add(RocksDbWeight::get().reads(2 as Weight))
                        .saturating_add(RocksDbWeight::get().reads((1 as Weight).saturating_mul(p as Weight)))
                        .saturating_add(RocksDbWeight::get().writes(2 as Weight))
                        .saturating_add(RocksDbWeight::get().writes((1 as Weight).saturating_mul(p as Weight)))
        }
        // Storage: Council Members (r:1 w:0)
        fn execute(b: u32, m: u32, ) -> Weight {
                (37_040_000 as Weight)
                        // Standard Error: 0
                        .saturating_add((4_000 as Weight).saturating_mul(b as Weight))
                        // Standard Error: 0
                        .saturating_add((187_000 as Weight).saturating_mul(m as Weight))
                        .saturating_add(RocksDbWeight::get().reads(1 as Weight))
        }
        // Storage: Council Members (r:1 w:0)
        // Storage: Council ProposalOf (r:1 w:0)
        fn propose_execute(b: u32, m: u32, ) -> Weight {
                (45_615_000 as Weight)
                        // Standard Error: 1_000
                        .saturating_add((7_000 as Weight).saturating_mul(b as Weight))
                        // Standard Error: 10_000
                        .saturating_add((337_000 as Weight).saturating_mul(m as Weight))
                        .saturating_add(RocksDbWeight::get().reads(2 as Weight))
        }
        // Storage: Council Members (r:1 w:0)
        // Storage: Council ProposalOf (r:1 w:1)
        // Storage: Council Proposals (r:1 w:1)
        // Storage: Council ProposalCount (r:1 w:1)
        // Storage: Council Voting (r:0 w:1)
        fn propose_proposed(b: u32, m: u32, p: u32, ) -> Weight {
                (53_342_000 as Weight)
                        // Standard Error: 0
                        .saturating_add((15_000 as Weight).saturating_mul(b as Weight))
                        // Standard Error: 3_000
                        .saturating_add((213_000 as Weight).saturating_mul(m as Weight))
                        // Standard Error: 3_000
                        .saturating_add((520_000 as Weight).saturating_mul(p as Weight))
                        .saturating_add(RocksDbWeight::get().reads(4 as Weight))
                        .saturating_add(RocksDbWeight::get().writes(4 as Weight))
        }
        // Storage: Council Members (r:1 w:0)
        // Storage: Council Voting (r:1 w:1)
        fn vote(m: u32, ) -> Weight {
                (66_122_000 as Weight)
                        // Standard Error: 4_000
                        .saturating_add((431_000 as Weight).saturating_mul(m as Weight))
                        .saturating_add(RocksDbWeight::get().reads(2 as Weight))
                        .saturating_add(RocksDbWeight::get().writes(1 as Weight))
        }
        // Storage: Council Voting (r:1 w:1)
        // Storage: Council Members (r:1 w:0)
        // Storage: Council Proposals (r:1 w:1)
        // Storage: Council ProposalOf (r:0 w:1)
        fn close_early_disapproved(m: u32, p: u32, ) -> Weight {
                (65_110_000 as Weight)
                        // Standard Error: 3_000
                        .saturating_add((358_000 as Weight).saturating_mul(m as Weight))
                        // Standard Error: 3_000
                        .saturating_add((433_000 as Weight).saturating_mul(p as Weight))
                        .saturating_add(RocksDbWeight::get().reads(3 as Weight))
                        .saturating_add(RocksDbWeight::get().writes(3 as Weight))
        }
        // Storage: Council Voting (r:1 w:1)
        // Storage: Council Members (r:1 w:0)
        // Storage: Council ProposalOf (r:1 w:1)
        // Storage: Council Proposals (r:1 w:1)
        fn close_early_approved(b: u32, m: u32, p: u32, ) -> Weight {
                (76_399_000 as Weight)
                        // Standard Error: 0
                        .saturating_add((12_000 as Weight).saturating_mul(b as Weight))
                        // Standard Error: 5_000
                        .saturating_add((378_000 as Weight).saturating_mul(m as Weight))
                        // Standard Error: 5_000
                        .saturating_add((519_000 as Weight).saturating_mul(p as Weight))
                        .saturating_add(RocksDbWeight::get().reads(4 as Weight))
                        .saturating_add(RocksDbWeight::get().writes(3 as Weight))
        }
        // Storage: Council Voting (r:1 w:1)
        // Storage: Council Members (r:1 w:0)
        // Storage: Council Prime (r:1 w:0)
        // Storage: Council Proposals (r:1 w:1)
        // Storage: Council ProposalOf (r:0 w:1)
        fn close_disapproved(m: u32, p: u32, ) -> Weight {
                (71_329_000 as Weight)
                        // Standard Error: 3_000
                        .saturating_add((369_000 as Weight).saturating_mul(m as Weight))
                        // Standard Error: 3_000
                        .saturating_add((444_000 as Weight).saturating_mul(p as Weight))
                        .saturating_add(RocksDbWeight::get().reads(4 as Weight))
                        .saturating_add(RocksDbWeight::get().writes(3 as Weight))
        }
        // Storage: Council Voting (r:1 w:1)
        // Storage: Council Members (r:1 w:0)
        // Storage: Council Prime (r:1 w:0)
        // Storage: Council ProposalOf (r:1 w:1)
        // Storage: Council Proposals (r:1 w:1)
        fn close_approved(b: u32, m: u32, p: u32, ) -> Weight {
                (80_242_000 as Weight)
                        // Standard Error: 0
                        .saturating_add((12_000 as Weight).saturating_mul(b as Weight))
                        // Standard Error: 3_000
                        .saturating_add((399_000 as Weight).saturating_mul(m as Weight))
                        // Standard Error: 3_000
                        .saturating_add((528_000 as Weight).saturating_mul(p as Weight))
                        .saturating_add(RocksDbWeight::get().reads(5 as Weight))
                        .saturating_add(RocksDbWeight::get().writes(3 as Weight))
        }
        // Storage: Council Proposals (r:1 w:1)
        // Storage: Council Voting (r:0 w:1)
        // Storage: Council ProposalOf (r:0 w:1)
        fn disapprove_proposal(p: u32, ) -> Weight {
                (40_464_000 as Weight)
                        // Standard Error: 2_000
                        .saturating_add((495_000 as Weight).saturating_mul(p as Weight))
                        .saturating_add(RocksDbWeight::get().reads(1 as Weight))
                        .saturating_add(RocksDbWeight::get().writes(3 as Weight))
        }
}
