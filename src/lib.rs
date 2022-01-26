pub mod codes_markers;
pub mod decoder;
pub mod huffman_tree;
pub mod mcu;

use crate::codes_markers::*;
use crate::huffman_tree::HuffmanTree;
use crate::mcu::MCU;

#[cfg(test)]
mod tests {
    use crate::huffman_tree::HuffmanTree;

    #[test]
    fn test_huffman_tree() {
        let mut ht = HuffmanTree::new();
        let leng = vec![0, 2, 3, 1, 1, 1];
        let vals = vec![3, 4, 2, 5, 6, 1, 0, 7];

        ht.build(&leng, &vals);

        ht.print();

        assert!(1 == 0);
    }
}
