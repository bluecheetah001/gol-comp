use crate::{Block, Node};

// macros make formatting nicer

/// this has very little error handling, must be a square with side length of 8
#[macro_export]
macro_rules! test_block {
    {$s:literal} => {
        $crate::Block::from_test_format($s)
    };
}
/// this has very little error handling, must be a square with side length a power of 2 >= 16
#[macro_export]
macro_rules! test_node {
    {$s:literal} => {
        $crate::Node::from_test_format($s)
    };
}
pub use {test_block, test_node};

impl Block {
    /// expects 8 lines of '.'s (dead) and 'o's (alive)
    /// first line may be empty, leading/trailing spaces are ignored
    pub fn from_test_format(s: &str) -> Block {
        let chunk = s
            .split_ascii_whitespace()
            .array_chunks()
            .next()
            .expect("8 rows");
        parse_block(chunk)
    }
}
impl Node {
    pub fn from_test_format(s: &str) -> Node {
        let mut nodes: Vec<Vec<Node>> = s
            .split_ascii_whitespace()
            .array_chunks()
            .map(parse_blocks)
            .array_chunks()
            .map(merge_leafs_row)
            .collect();
        while nodes.len() > 1 {
            nodes = merge_nodes(nodes)
        }
        nodes.pop().unwrap().pop().unwrap()
    }
}

fn merge_nodes(nodes: Vec<Vec<Node>>) -> Vec<Vec<Node>> {
    nodes
        .into_iter()
        .array_chunks()
        .map(merge_nodes_row)
        .collect()
}
fn merge_nodes_row(blocks: [Vec<Node>; 2]) -> Vec<Node> {
    let [n, s] = blocks;
    std::iter::zip(n, s)
        .array_chunks()
        .map(merge_node)
        .collect()
}
fn merge_node(nodes: [(Node, Node); 2]) -> Node {
    let [(nw, sw), (ne, se)] = nodes;
    Node::new(nw, ne, sw, se)
}

fn merge_leafs_row(blocks: [Vec<Block>; 2]) -> Vec<Node> {
    let [n, s] = blocks;
    std::iter::zip(n, s)
        .array_chunks()
        .map(merge_leaf)
        .collect()
}
fn merge_leaf(blocks: [(Block, Block); 2]) -> Node {
    let [(nw, sw), (ne, se)] = blocks;
    Node::new(nw, ne, sw, se)
}

fn parse_blocks(rows: [&str; 8]) -> Vec<Block> {
    let len = rows[0].len() / 8;
    let mut blocks = Vec::with_capacity(len);
    for i in 0..len {
        blocks.push(parse_block(rows.map(|r| &r[8 * i..][..8])));
    }
    blocks
}
fn parse_block(rows: [&str; 8]) -> Block {
    Block::from_rows_array(rows.map(parse_block_row))
}
fn parse_block_row(s: &str) -> u8 {
    let b = s.as_bytes();
    let mut p = 0;
    for i in 0..8 {
        p |= parse_bit(b[i]) << (7 - i);
    }
    p
}
fn parse_bit(b: u8) -> u8 {
    match b {
        b'.' => 0,
        b'o' => 1,
        _ => panic!("Invalid byte {b:02x}"),
    }
}

#[cfg(test)]
mod test {
    use crate::{Block, Node};

    #[test]
    fn block() {
        let parsed = test_block! {"
            oooo....
            oo....oo
            o.o...oo
            ..ooo...
            .......o
            ........
            oooooooo
            .o.o.o.o
        "};
        let actual = Block::from_rows(0xf0_c3_a3_38_01_00_ff_55);
        assert_eq!(actual, parsed);
    }

    #[test]
    fn node_0() {
        let parsed = test_node! {"
            o......o...oo.o.
            o....o.oo..ooo.o
            ..oo.oo.o.oo.ooo
            ....oo.oo.o.oo.o
            .oo.o..o..o.oooo
            oo.o.oo.....o.o.
            o.o.o.o...o.o.o.
            oo.ooooooo.o....
            .o..o..oo...o...
            oo..o..oooo...oo
            .o..oo.ooo...o.o
            .o...o.ooooo..oo
            ..o.ooo..o....oo
            .o.....o.o.oooo.
            o.oo.ooo.oooo.oo
            o....o.oo......o
        "};
        let actual = Node::new(
            Block::from_rows(0x81_85_36_0d_69_d6_aa_df),
            Block::from_rows(0x1a_9d_b7_ad_2f_0a_2a_d0),
            Block::from_rows(0x49_c9_4d_45_2e_41_b7_85),
            Block::from_rows(0x88_e3_c5_f3_43_5e_7b_81),
        );
        assert_eq!(actual, parsed);
    }
}
