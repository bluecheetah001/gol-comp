use crate::{Block, Node, Quad};

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
    let [(nw, ne), (sw, se)] = nodes;
    Node::new_inner(Quad { nw, ne, sw, se })
}

fn merge_leafs_row(blocks: [Vec<Block>; 2]) -> Vec<Node> {
    let [n, s] = blocks;
    std::iter::zip(n, s)
        .array_chunks()
        .map(merge_leaf)
        .collect()
}
fn merge_leaf(blocks: [(Block, Block); 2]) -> Node {
    let [(nw, ne), (sw, se)] = blocks;
    Node::new_leaf(Quad { nw, ne, sw, se })
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
