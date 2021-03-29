#[cfg(test)]
mod tests {
    use rust_blockchain::blockchain::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[derive(Copy, Clone, Debug, PartialEq)]
    struct Transaction<'a> {
        index : u32,
        sender : &'a str,
        receiver : &'a str,
        amount : f64,
        signature : [u8; 32]
    }

    impl AsRef<[u8]> for Transaction<'_> {
        fn as_ref(&self) -> &[u8] {
            unimplemented!()
        }
    }

    static TRANSACTION_1: Transaction = Transaction {
        index : 1,
        sender : "Alice",
        receiver : "Bob",
        amount : 100.0,
        signature: [34u8; 32]
    };

    #[test]
    fn test_merkle_tree() {
        let tree1 : MerkleTree<Transaction> = MerkleTree::new(vec![TRANSACTION_1]);
        assert!(tree1.verify());
        assert_eq!(vec![TRANSACTION_1], tree1.get_currently_stored_data());
    }

    #[test]
    fn test_block() {
        let previous_hash : SHAHash = [11u8; 32];
        let data : MerkleTree<Transaction> = MerkleTree::new(vec![TRANSACTION_1]);
        let mut test_block : Block<Transaction> = Block::new(previous_hash, data);
        assert!(!test_block.verify());
        test_block.calculate_nonce();
        assert!(test_block.verify());
    }

    #[test]
    fn test_blockchain() {

    }

}
