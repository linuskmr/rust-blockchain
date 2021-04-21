use sha2::Sha256;
use sha2::Digest;
use crate::merkle_tree::MerkleTree;

/// The number of zeros the hash of a Block has to start with in order for it to be accepted.
///
/// The higher this number, the more difficult it becomes to continue the Blockchain.
/// The probability of a hash starting with that many zeros is 1 in 2^ZEROS.
///
/// As we use SHA-256 as the hashing algorithm, this number can be anywhere between
/// 0 (no effort at all) and 255 (essentially impossible) -> therefore stored as a u8
const ZEROS : u8 = 5;


/// This is the hash that's stored as the 'previous hash' (prev_hash) for
/// the very first / initial Block of a Blockchain
static INITIAL_HASH : SHAHash = [0u8; 32];

/// A single Block in the Blockchain.
/// May store all the data of this Block or just parts of it, but at least the root hash of the
/// Merkle Tree so that the data can be restored at any time, even from an unreliable source.
#[derive(Clone, Debug)]
pub struct Block<T : AsRef<[u8]> + Clone> {
    /// The hash of the Block that came before this Block.
    pub(crate) prev_hash: SHAHash,
    /// Random data such that the overall hash of this Block starts with ZEROS 0's.
    nonce: Nonce,
    // The actual data of a Block (or just parts of it, but the root hash at minimum)
    // is stored in a Merkle Tree.
    merkle_tree: MerkleTree<T>
}

impl<T : AsRef<[u8]> + Clone> Block<T> {

    /// Creates a new Block
    /// - that comes after the block with the given Hash
    /// - with the data from the given MerkleTree
    ///
    /// The Nonce of the new Block has yet to be calculated by calling calculate_nonce()
    /// afterwards ("mining") !!!
    pub fn new(previous_hash : SHAHash, data : MerkleTree<T>) -> Block<T> {
        Block {
            prev_hash : previous_hash,
            nonce : 0, // has yet to be calculated!
            merkle_tree: data
        }
    }

    /// Calculates the Nonce for this Block such that this Block's hash (calculate_hash())
    /// starts with ZEROS 0's.
    /// The higher the ZEROS constant, the more difficult/time-intensive this operation becomes.
    ///
    /// This is essentially the "mining" process.
    pub fn calculate_nonce(&mut self) -> Nonce {
        // Increment this Block's nonce until it's correct, i.e. this Block's hash starts with ZERO 0's:
        while !self.verify_nonce() {
            self.nonce += 1;
        }
        self.nonce
    }

    /// Returns the hash of this Block.
    /// When calculate_nonce() has been called on this Block beforehand,
    /// the hash will start with ZEROS 0's.
    pub fn calculate_hash(&self) -> SHAHash {
        Sha256::new()
            .chain(self.prev_hash)
            .chain(self.nonce.to_be_bytes())
            .chain(self.merkle_tree.get_root_hash()) // Very important to just use the root hash!
            .finalize()
    }

    /// Checks whether the nonce of this Block was chosen correctly, i.e. whether the hash of
    /// this block starts with ZEROS 0's.
    pub fn verify_nonce(&self) -> bool {
        let hash = self.calculate_hash();

        // Expected
        const NO_OF_NULL_BYTES : usize = (ZEROS / 8) as usize;
        const NO_OF_0_BITS: usize = (ZEROS % 8) as usize;
        assert!(NO_OF_0_BITS <= 8);

        // Check if there are NO_OF_NULL_BYTES bytes with the value 0.
        if hash.iter().take(NO_OF_NULL_BYTES).any(|&byte| byte != 0) {
            return false;
        }

        if NO_OF_0_BITS == 0 {
            // Only whole bytes should be 0, but no single bits in the last mixed byte.
            return true;
        }

        // Go through the mixed byte after the zero bytes and check that the first NO_OF_0_BITS are zero
        let last_mixed_byte = hash[NO_OF_NULL_BYTES];
        let mut pattern = 0b1000_0000u8;
        for _ in 0..NO_OF_0_BITS {
            if last_mixed_byte & pattern != 0 {
                // Found a non-zero bit
                return false;
            }
            pattern >>= 1;
        }
        true
    }

    /// Checks whether the Merkle Tree of this Block is valid.
    pub fn verify_merkle_tree(&self) -> bool {
        self.merkle_tree.verify()
    }

    /// Checks whether this Block is valid (as seen on its own not in its context as part of
    /// a Blockchain).
    /// Combined check of both verify_nonce() and verify_merkle_tree().
    pub fn verify(&self) -> bool {
        self.verify_nonce() && self.verify_merkle_tree()
    }

    /// Removes the storage of all the data in this Block to clean up space/memory.
    /// The data can however be restored later at any point in time using restore_merkle_tree().
    pub fn clear_merkle_tree(&mut self) {
        self.merkle_tree.shrink_to_minimum();
    }

    /// Tries to restore the data of this Block using a MerkleTree coming from an outside
    /// (unreliable) source.
    /// Returns true if the data was restored successfully, i.e. the MerkleTree was correct and
    /// its root hash was equal to the root hash stored in this Block's Header.
    /// Returns false if no data was restored, i.e. the MerkleTree given was somehow invalid.
    pub fn restore_merkle_tree(&mut self, mtree : MerkleTree<T>) -> bool {
        if mtree.verify() && mtree.get_root_hash() == self.merkle_tree.get_root_hash() {
            self.merkle_tree = mtree;
            true
        } else {
            false
        }
    }
}