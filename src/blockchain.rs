use crate::block::Block;
use std::sync::Mutex;
use crate::merkle_tree::MerkleTree;

/// A Blockchain chaining Blocks, each of the Blocks storing multiple values of type T.
#[derive(Debug)]
pub struct Blockchain<T : AsRef<[u8]> + Clone> {
    /// The list of all blocks stored in this blockchain.
    blocks : Vec<Block<T>>,
    /// Only 1 thread shall be able to append a Block to a Blockchain at a given time
    /// (to undefined undefined behaviour). Note that this mutex only locks the appending and
    /// obviously not the time-intensive mining process that happens beforehand!
    /// (see difference between append_block() and append_data())
    append_mutex : Mutex<()>
}

impl<T : AsRef<[u8]> + Clone> Blockchain<T> {

    /// Creates a new `Blockchain`.
    pub fn new() -> Blockchain<T> {
        Blockchain {
            blocks: Vec::new(),
            append_mutex : Mutex::new(())
        }
    }

    /// Returns the total number of Blocks in this Blockchain.
    pub fn length(&self) -> usize {
        self.blocks.len()
    }

    /// Returns the hash of the last/latest block in this Blockchain
    /// or the INITIAL_HASH when this Blockchain is still empty.
    pub fn hash_of_last_block(&self) -> SHAHash {
        match self.blocks.last() {
            Some(last_block) => last_block.calculate_hash(),
            None => INITIAL_HASH,
        }
    }

    /// Verify the correctness of this Blockchain:
    /// - Verifies whether all the Block Hashes are correct AND valid (i.e. start with ZEROS 0's)
    /// - Verifies whether all the Merkle Root Hashes are correct.
    ///
    /// Or to put it differently:
    /// - Checks whether all the 'Previous Hashes' of the Blocks actually are the hash of the
    ///   blocks that comes directly before it.
    /// - Calls .verify() on each of the blocks in this Blockchain (this includes checking all
    ///   of the Merkle Trees for validity!)
    pub fn verify(&self) -> bool {
        let mut previous_hash = INITIAL_HASH;
        for block in &self.blocks {
            let valid_link_to_prev_block = block.prev_hash == previous_hash;
            let valid_block = block.verify();
            if !valid_link_to_prev_block || !valid_block {
                // Inconsistency found!
                return false;
            }
            // TODO: I don't think it's necessary to recalculate the hashes
            // previous_hash = block.calculate_hash(); // update the previous_hash
        }
        // No inconsistencies found in the Blockchain!
        true
    }

    /// Checks whether the given Block has a correct nonce and prev_hash.
    /// If so, appends the given Block to this Blockchain and returns true.
    /// Returns false when the given Block was incorrect and was not appended.
    ///
    /// This function is primarily used for appending Blocks that others publicly announced to
    /// your private copy of the Blockchain.
    /// In order to append your own data, you have to find out the nonce using trial-and-error
    /// first - the append_data() function does that for you.
    pub fn append_block(&mut self, block : Block<T>) -> bool {
        let _guard = self.append_mutex.lock().unwrap(); // <synchronize>
        let valid_block = block.verify();
        let valid_link_to_prev_block = block.prev_hash == self.hash_of_last_block();
        if valid_block && valid_link_to_prev_block {
            self.blocks.push(block);
            true
        } else {
            // Invalid blockchain
            // TODO: Instead of a return value maybe just panic, if the blockchain is invalid?
            false
        }
        // </synchronize> The lock is released automatically here because the MutexGuard goes out of scope!
    }

    /// Takes the data given as a MerkleTree and "mines" a new Block for it, then appends it to
    /// this Blockchain. When the "mining" (nonce calculation) finished but another Block was
    /// appended in the meantime (most likely via an append_block() call), the whole process has
    /// to start over. This means that calling this function can take very long - potentially
    /// forever!
    ///
    /// This function also returns a copy of the "mined" Block so you can announce it to the network!!
    /// (The communication with others on the network is NOT part of this library!!)
    pub fn append_data(&mut self, mtree : MerkleTree<T>) -> Block<T> {
        let mut new_block = Block::new(self.hash_of_last_block(), mtree);
        new_block.calculate_nonce();
        self.append_block(new_block.clone());
        new_block

        // ToDo: concurrency!
        //    1) restart calculating a nonce with a/the new prev_hash when either
        //       a) another thread was faster and already appended data in the meantime,
        //       or, much more likely,
        //       b) somebody else publicly announced a new block which you/another thread of you
        //          simply added using a quick append_block() call
    }
}