use sha2::Sha256;
use sha2::Digest;
use std::str::FromStr;

/// In order to be able to reduce the size of the Blockchain / to forget old
/// no longer necessary to remember data, Blocks store their data in a Merkle Tree.
///
/// The Merkle Tree can be shrunk, either in part or completely (leaving only its root hash),
/// but deleted data can be restored later at any point in time, even from an unreliable source!
///
/// In a currency Blockchain, the type T would be something representing a transaction.
///
/// For graphics of Merkle Trees, see the Bitcoin paper (https://bitcoin.org/bitcoin.pdf), pp.4+5
#[derive(Clone, Debug)]
pub enum MerkleTree<T : AsRef<[u8]> + Clone> {
    // A node with a left and a right child and the hash of them.
    Node {
        /// The hash of both children `left` and `right` of this node.
        hash: SHAHash,
        /// The left child of this node.
        left: Box<MerkleTree<T>>,
        /// The right child of this node.
        right: Box<MerkleTree<T>>
    },

    /// A leaf is a node without children, but with data
    Leaf {
        /// The hash of the data of this leaf.
        hash: SHAHash,
        /// - A Leaf with the Option being 'Some' is an actual Leaf of the Merkle Tree storing some data T.
        /// - A Leaf with the Option being 'None' could be 2 things:
        /// a) an actual Leaf of the full Merkle Tree with just the data T missing OR
        /// b) the root of a Merkle Subtree that has been chopped off (when reinserting that,
        ///    the Option shall never become 'Some' but the Leaf shall rather be replaced with a Node)
        data: Option<T>
    }
}

impl<T: AsRef<[u8]> + Clone> MerkleTree<T> {

    /// Creates a new Merkle Tree with the data from the given Vector.
    /// If your data is in some other data structure, please collect() it into a Vec first.
    ///
    /// Please note that no data may be added later on and that the data also cannot be changed.
    /// Data can however be forgotten to save space and be restored later.
    ///
    /// Important: this function panics when called on an empty Vec!
    pub fn new(data: &[T]) -> MerkleTree<T> {
        /*match vector.len() {
            0 => panic!("Cannot create a MerkleTree from an empty Vec!"),
            1 => MerkleTree::Leaf(SHAHash::from(Sha256::new().chain(vector[0]).finalize()), Some(vector[0])),
            2 => {
                let left_leaf = MerkleTree::Leaf(SHAHash::from(Sha256::new().chain(vector[0]).finalize()), Some(vector[0]));
                let right_leaf = MerkleTree::Leaf(SHAHash::from(Sha256::new().chain(vector[1]).finalize()), Some(vector[1]));
                MerkleTree::Node(SHAHash::from(Sha256::new().chain(left_leaf.get_root_hash()).chain(right_leaf.get_root_hash()).finalize()),
                                 Box::new(left_leaf),
                                 Box::new(right_leaf))
            }
            3 => {
                let leaf1 = MerkleTree::Leaf(SHAHash::from(Sha256::new().chain(vector[0]).finalize()), Some(vector[0]));
                let leaf2 = MerkleTree::Leaf(SHAHash::from(Sha256::new().chain(vector[1]).finalize()), Some(vector[1]));
                let leaf3 = MerkleTree::Leaf(SHAHash::from(Sha256::new().chain(vector[2]).finalize()), Some(vector[2]));
                let left_subtree = MerkleTree::Node(SHAHash::from(Sha256::new().chain(leaf1.get_root_hash()).chain(leaf2.get_root_hash()).finalize()),
                                                    Box::new(leaf1),
                                                    Box::new(leaf2));
                MerkleTree::Node(SHAHash::from(Sha256::new().chain(left_subtree.get_root_hash()).chain(leaf3.get_root_hash()).finalize()),
                                 Box::new(left_subtree),
                                 Box::new(leaf3))
            }
            4 => {
                let leaf1 = MerkleTree::Leaf(SHAHash::from(Sha256::new().chain(vector[0]).finalize()), Some(vector[0]));
                let leaf2 = MerkleTree::Leaf(SHAHash::from(Sha256::new().chain(vector[1]).finalize()), Some(vector[1]));
                let leaf3 = MerkleTree::Leaf(SHAHash::from(Sha256::new().chain(vector[2]).finalize()), Some(vector[2]));
                let leaf4 = MerkleTree::Leaf(SHAHash::from(Sha256::new().chain(vector[2]).finalize()), Some(vector[2]));
                let left_subtree = MerkleTree::Node(SHAHash::from(Sha256::new().chain(leaf1.get_root_hash()).chain(leaf2.get_root_hash()).finalize()),
                                                    Box::new(leaf1),
                                                    Box::new(leaf2));
                let right_subtree = MerkleTree::Node(SHAHash::from(Sha256::new().chain(leaf3.get_root_hash()).chain(leaf4.get_root_hash()).finalize()),
                                                     Box::new(leaf3),
                                                     Box::new(leaf4));
                MerkleTree::Node(SHAHash::from(Sha256::new().chain(left_subtree.get_root_hash()).chain(right_subtree.get_root_hash()).finalize()),
                                 Box::new(left_subtree),
                                 Box::new(right_subtree))
            }
            _ => {
                panic!("ToDo: create a new MerkleTree with more than 4 leaves!"); // ToDo
            }
        }*/

        match data.len() {
            0 => panic!("Cannot create a MerkleTree from an data array!"),
            1 => {
                MerkleTree::Leaf {
                    hash: Sha256::new().digest(data[0].clone()),
                    data: data[0].clone()
                }
            },
            _ => {
                // Split the data in two equally sized subtrees
                let mut parts = data.chunks(data.len()/2);
                // Create subtrees for the two parts
                let left_subtree = Self::new(parts.next().unwrap());
                let right_subtree = Self::new(parts.next().unwrap());
                // Return a node with these two parts as children
                MerkleTree::Node {
                    hash: Sha256::new()
                        .chain(left_subtree.get_root_hash())
                        .chain(right_subtree.get_root_hash()).finalize(),
                    left: Box::new(left_subtree),
                    right: Box::new(right_subtree)
                }
            }
        }
    }

    /// Returns the hash of this MerkleTree.
    pub fn get_root_hash(&self) -> SHAHash {
        match self {
            MerkleTree::Node{hash, ..} => hash,
            MerkleTree::Leaf{hash, ..} => hash
        }
    }

    /// Checks whether this Merkle Tree is valid, i.e. all hashes are correct.
    pub fn verify(&self) -> bool {
        match self {
            MerkleTree::Leaf{data: None, ..} => {
                // No data stored at all -> no hashes to match
                true
            },
            MerkleTree::Leaf{hash, data: Some(t)} => {
                // Check if the stored hash matches the one recalculated using data
                *hash == SHAHash::from(Sha256::new().digest(t))
            },
            MerkleTree::Node{hash, left, right} => {
                // Check if the stored hash matches the one recalculated using the two subtrees
                let valid_root = *hash == SHAHash::from(
                    Sha256::new()
                        .chain(left.get_root_hash())
                        .chain(right.get_root_hash()).finalize()
                );
                // Check if both subtrees are valid in themself
                let valid_children = left.verify() && right.verify(); // -> recursion
                return valid_root && valid_children
            }
        }
    }

    // ----- Retrieve data from this MerkleTree: -----

    /// Returns all the data that's currently stored in this Merkle Tree.
    /// This may NOT be all the data when some of it was already forgotten!
    pub fn get_currently_stored_data(&self) -> Vec<T> {
        match self {
            MerkleTree::Leaf{data: None, ..} => {
                // No data available
                vec![]
            },
            MerkleTree::Leaf{data: Some(data), ..} => {
                // Self is a leaf, so only export its data
                Vec::from(data)
            },
            MerkleTree::Node{left, right, ..} => {
                // Collect the data from the left and right subtrees
                left.get_currently_stored_data()
                    .append(&mut right.get_currently_stored_data())
            }
        }
    }

    // ----- Exporting & Importing a MerkleTree as XML: -----
    // (the whole point of storing the)

    /// Export this Merkle Tree in an XML format. The XML can be stored somewhere else and
    /// this Merkle Tree shrunk by calling shrink_to_minimum() to save memory.
    pub fn export_xml(&self) -> String {
        // TODO: You may need the trait `LowerHex` for self
        match self {
            MerkleTree::Leaf{hash, data: None /* TODO May need None() here? */} => format!("<leaf hash=\"{:x?}\" />", hash),
            MerkleTree::Leaf{hash, data: Some(data)} => format!("<leaf hash=\"{:x?}\" data=\"{}\" />", hash, data.to_string()),
            MerkleTree::Node{hash, left, right} =>
                format!("<node hash=\"{:x?}\"><left>{}</left><right>{}</right></node>", hash, left.export_xml(), right.export_xml())
        }
    }

    /// Please note that the imported MerkleTree is NOT verified!!!
    /// You have to call verify() afterwards - especially when the XML is coming from an
    /// unreliable source!!!
    ///
    /// Returns None when the input XML was invalid.
    /*pub fn import_xml(xml : String) -> Option<MerkleTree<T>> {
        return if xml.starts_with("<leaf hash=") {
            // Import leaf
            // TODO: Type annotations needed?
            let hash: SHAHash = SHAHash::from_hex(xml_helper_parse_attr(&xml, "hash")).ok()?;
            if xml.contains("data=") {
                let data : T = T::from_str(xml_helper_parse_attr(&xml, "data")
                    .as_str())
                    .ok()?;
                Some(MerkleTree::Leaf{
                    hash,
                    data: Some(data)
                })
            } else {
                Some(MerkleTree::Leaf{
                    hash,
                    data: None
                })
            }
        } else if xml.starts_with("<node hash=") {
            // Import node
            let hash: SHAHash = SHAHash::from_hex(xml_helper_parse_attr(&xml, "hash")).ok()?;
            // TODO: Annotated types required?
            let left: MerkleTree<T> = MerkleTree::import_xml(xml_helper_parse_tag(&xml, "left"))?;
            let right: MerkleTree<T> = MerkleTree::import_xml(xml_helper_parse_tag(&xml, "right"))?;
            Some(MerkleTree::Node{
                hash,
                left: Box::new(left),
                right: Box::new(right)
            })
        } else {
            // Illegal xml
            // TODO: Maybe panic?
            None
        }
    }*/

    pub fn contains_hash(&self, search_hash: &SHAHash) -> bool {
        match self {
            MerkleTree::Leaf{hash, ..} => hash == search_hash,
            MerkleTree::Node{hash, left, right} => {
                hash == search_hash
                    || left.contains_hash(search_hash)
                    || right.contains_hash(search_hash)
            }
        }
    }

    // ----- Grow/Restore: -----

    /// Tries to restore the given element back into this Merkle Tree.
    /// Returns true if the element was restored successfully or if it was already present.
    /// Returns false if the hash of the given element was not found in this Merkle Tree.
    /// If so, you probably have to use restore_subtree() instead.
    pub fn restore_element(&mut self, element: &T) -> bool { // ToDo: avoid Copy
        // Calculate the hash of the given element
        let element_hash = SHAHash::from(Sha256::new()
            .chain(element.clone())
            .finalize());
        match self {
            MerkleTree::Leaf{hash, ..} if hash == element_hash => {
                *self = MerkleTree::Leaf{
                    hash,
                    data: Some(element.clone())
                };
                true
            },
            MerkleTree::Node{left, right, ..} => {
                // Try to restore the element in the left, then in the right subtree
                left.restore_element(element) || right.restore_element(element)
            },
            _ => {
                // No matching node or leaf found for restoring
                false
            }
        }
    }

    /// Tries to insert the given subtree into this Merkle Tree.
    /// Returns false when the root hash of the given subtree was not found in this Merkle Tree.
    ///
    /// Please note that this operation can lead to data being added as well as
    /// data being removed!
    ///
    /// Please also note that the given MerkleTree is NOT checked for validity!
    /// That has to be done beforehand if it's coming from an unreliable source!
    pub fn insert_subtree(&mut self, subtree: MerkleTree<T>) -> bool {
        if self.get_root_hash() == subtree.get_root_hash() {
            // Replace the current tree with the given subtree
            *self = subtree;
            return true;
        }

        return match self {
            MerkleTree::Leaf{..} => {
                // The given subtree does *not* match with the hash of self. If the subtree
                // is a MerkleTree::Leaf, then the hashes should have matched. If the subtree
                // is a MerkleTree::Node, then the subtree cannot be inserted here as a
                // MerkleTree::Leaf, because of conflicting types.
                false
            },
            MerkleTree::Node{left, right, ..} => {
                if left.get_root_hash() == subtree.get_root_hash() {
                    *left = Box::new(subtree);
                    true
                } else if right.get_root_hash() == subtree.get_root_hash() {
                    *right = Box::new(subtree);
                    true
                } else if left.contains_hash(subtree.get_root_hash()) {
                    left.insert_subtree(subtree)
                } else if right.contains_hash(subtree.get_root_hash()) {
                    right.insert_subtree(subtree)
                } else {
                    // Could not insert subtree as left or right subtree or in them.
                    false
                }
            }
        }
    }

    /// Tries to restore the given subtree back into this Merkle Tree.
    /// Returns true if the subtree was restored successfully, even if it was already present (or just in parts).
    ///
    /// This function is smarter than insert_subtree(): Instead of simply replacing the current
    /// subtree with the one given as the parameter (which may actually lead to LOSING data
    /// instead of restoring it when the given subtree contains less data than the one that's
    /// currently already stored) this function is lossless - even when the given subtree
    /// contains less data than the one that's currently stored in this Merkle Tree, no data
    /// is lost!
    ///
    /// Returns false if the root hash of the Merkle Tree given was not found in this Merkle Tree.
    /// If so, you probably have to restore a bigger subtree!
    ///
    /// Please note that the given MerkleTree is NOT checked for validity!
    /// That has to be done beforehand if it's coming from an unreliable source!
    pub fn restore_subtree(&mut self, subtree : MerkleTree<T>) -> bool {
        false // ToDo
    }

    // ----- Shrink: -----

    /// Shrinks this MerkleTree to its minimum size, leaving only its root hash.
    pub fn shrink_to_minimum(&mut self) {
        *self = MerkleTree::Leaf{
            hash: self.get_root_hash(),
            data: None
        };
    }

    /// Looks for an element in this Merkle Tree that's equal to the given element and 'forgets'
    /// it, i.e. deletes it from this Merkle Tree (but keeps the hash).
    /// The element can be restored later by calling restore_element().
    /// However, when larger parts of this Merkle Tree were thrown away, a restore_subtree()
    /// might be necessary.
    ///
    /// Returns false when no element equal to the given element was found in this Merkle Tree or
    /// when it already is forgotten (i.e. only its hash still being there).
    ///
    /// When there are multiple element in this Merkle Tree equal to the given one
    /// (which actually shouldn't be the case for most sensible Blockchain applications)
    /// only the leftmost one is forgotten/deleted and true is returned.
    pub fn forget_leaf(&mut self, element: &T) -> bool where T : PartialEq + Copy { // ToDo: avoid Copy
        match self {
            MerkleTree::Leaf{data: None, ..} => {
                // This MerkleTree is already forgotten
                false
            },
            MerkleTree::Leaf{hash, data: Some(data)} if data == element => {
                *self = MerkleTree::Leaf{hash, data: None};
                true
            },
            MerkleTree::Node{left, right, ..} => {
                left.forget_leaf(element) || right.forget_leaf(element) // || = lazy OR!
            },
            _ => {
                // No matching MerkleTree with equal hash found
                false
            }
        }
    }

    /// Deletes all the data stored in the leaves of this Merkle Tree but leaves the entire
    /// tree structure and all the hashes intact.
    /// This is a less severe operation than shrink_to_minimum, enabling all of the forgotten
    /// elements to be restored individually using restore_element(), in any order.
    ///
    /// This operation makes sense particularly when the datatype T takes up significantly more
    /// storage than a 256-bit hash.
    pub fn forget_all_leaves(&mut self) {
        // Use recursion:
        match self {
            MerkleTree::Leaf(hash, _) => {
                *self = MerkleTree::Leaf{hash, data: None}; // Forget Leaf (data)!
            },
            MerkleTree::Node(_, left, right) => {
                // Forget all leaves of the left and of the right subtree:
                left.forget_all_leaves();
                right.forget_all_leaves();
            },
        }
    }

    /// Deletes the subtree of this Merkle Tree that has the given hash as its root hash.
    /// The root hash itself is kept!
    /// Returns false when this Merkle Tree (currently) does not have a subtree with that hash.
    ///
    /// Calling mtree.forget_subtree(mtree.get_root_hash()) is equivalent to calling
    /// mtree.shrink_to_minimum().
    pub fn forget_subtree(&mut self, hash : SHAHash) -> bool {
        if hash == self.get_root_hash() {
            self.shrink_to_minimum();
            true
        } else {
            match self {
                MerkleTree::Leaf(_,_) => false, // hash not found!
                MerkleTree::Node(_, left, right) =>
                    {
                        left.forget_subtree(hash) || right.forget_subtree(hash)
                    }
            }
        }
    }
}