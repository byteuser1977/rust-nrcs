use consensus::{ConsensusEngine, PoSEngine};
use blockchain_types::*;

#[test]
fn test_pos_chain_validation() {
    let engine = PoSEngine::new(15, 1000, 100);
    let genesis = Block::new(0, [0u8; 32], 0);
    let mut current = genesis.clone();

    // Simulate a short chain
    for height in 1..10 {
        let mut next = Block::new(height, current.compute_hash().unwrap(), 1);
        next.timestamp = current.timestamp + 15; // target spacing
        next.base_target = 1000;

        // Validate the block
        assert!(engine.verify_timestamp(&next, next.timestamp).is_ok());

        current = next;
    }
}

#[test]
fn test_difficulty_persistence_over_chain() {
    let engine = PoSEngine::new(15, 1000, 100);
    let mut blocks = vec![];

    for i in 0..20 {
        let mut block = Block::new(i as u32, if i == 0 { [0u8; 32] } else { blocks[i-1].compute_hash().unwrap() }, 1);
        block.timestamp = 1000 + i as u32 * 15;
        block.base_target = 1000;
        blocks.push(block);
    }

    // Calculate difficulty for next block
    let next_diff = engine.calculate_next_difficulty(&blocks);
    // Should remain stable if timing is on target
    assert!(next_diff > 0);
}

#[test]
fn test_consensus_engine_trait_object() {
    // Ensure trait objects can be created (compilation test)
    let pos_engine: Box<dyn ConsensusEngine> = Box::new(PoSEngine::new(15, 1000, 100));
    let block = Block::new(1, [0u8; 32], 0);
    assert!(pos_engine.verify_difficulty(&block).is_ok());
}
