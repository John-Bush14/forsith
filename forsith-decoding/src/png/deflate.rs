#[derive(Debug)]
pub struct Block {
    pub last: bool,
    pub compressed: bool,
    pub tree: Option<(HuffmanTree<u16>, HuffmanTree<u16>)>
}

#[derive(Debug)]
enum Entry<S> {
    Symbol(S),
    NestedTree
}

#[derive(Debug)]
pub struct HuffmanTree<S> {
    table: Vec<(Entry<S>, u8)>, // (symbol, code length)
    max_bits: u8,
    nested_tree: Option<Box<HuffmanTree<S>>>,
}
