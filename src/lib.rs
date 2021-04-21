mod block;
mod blockchain;
mod merkle_tree;

// Implementing the thoughts of Satoshi Nakamoto in https://bitcoin.org/bitcoin.pdf:
/*pub mod blockchain {
    use sha2::Sha256;
    use sha2::Digest;
    //use std::hash::Hash; // unnecessary, SHA requires the AsRef<[u8]> trait instead of the Hash trait
    use std::sync::Mutex; // (appending Blocks to a Blockchain has to be synchronized)
    use hex::FromHex; // https://stackoverflow.com/questions/52987181/how-can-i-convert-a-hex-string-to-a-u8-slice
    use std::str::FromStr;



    // The Nonce of a Block is the things that's incremented until the Block's hash has the
    // required zero bits (as many as specified by the ZEROS constant).









    fn xml_helper_parse_attr(xml : &String, attr_name : &str) -> String {
        String::from("") // ToDo
    }

    fn xml_helper_parse_tag(xml : &String, tag_name : &str) -> String {
        String::from("") // ToDo
    }

}*/
