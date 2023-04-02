use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display};
use std::io::{Error as IoError, Write};

use either::Either;

use crate::{Block, DepthQuad, Node, Population};

impl Node {
    pub fn write_to(&self, write: impl Write) -> Result<(), IoError> {
        McWriter::new(write).write(self)
    }
    pub fn write_to_string(&self) -> String {
        String::from_utf8(self.write_to_bytes()).expect("valid string")
    }
    pub fn write_to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        self.write_to(&mut out).expect("valid bytes");
        out
    }

    // pub fn read_from(
    //     mut read: impl Read,
    // ) -> Result<Result<Self, MacrocellError<Vec<u8>>>, IoError> {
    //     // api setup such that far down the road this could avoid reading everything in up front
    //     let mut buf = Vec::new();
    //     read.read_to_end(&mut buf)?;
    //     Ok(Node::read_from_bytes(&buf).map_err(|e| e.into_owned()))
    // }
    pub fn read_from_bytes(bytes: &[u8]) -> Result<Self, MacrocellError<&[u8]>> {
        McReader::new(bytes).read()
    }
    // TODO could make error generic be &str, unsafe conversion is probably possible
    pub fn read_from_string(string: &str) -> Result<Self, MacrocellError<&[u8]>> {
        McReader::new(string.as_bytes()).read()
    }
}

// formatting

struct McWriter<'n, W> {
    write: W,
    nodes: HashMap<&'n Node, usize>,
    blocks: HashMap<Block, usize>,
    last: usize,
}
impl<'n, W: Write> McWriter<'n, W> {
    fn new(write: W) -> Self {
        McWriter {
            write,
            nodes: HashMap::new(),
            blocks: HashMap::new(),
            last: 0,
        }
    }
    fn write(mut self, node: &'n Node) -> Result<(), IoError> {
        self.write_header()?;
        self.write_node(node)
    }
    fn write_header(&mut self) -> Result<(), IoError> {
        writeln!(self.write, "[M2] (metalife 1.0)")?;
        writeln!(self.write, "#R B3/S23")?;
        Ok(())
    }
    fn write_node(&mut self, node: &'n Node) -> Result<(), IoError> {
        let size = node.width_log2();
        match node.depth_quad() {
            DepthQuad::Leaf(leaf) => {
                let nw = self.maybe_write_block(leaf.nw)?;
                let ne = self.maybe_write_block(leaf.ne)?;
                let sw = self.maybe_write_block(leaf.sw)?;
                let se = self.maybe_write_block(leaf.se)?;
                writeln!(self.write, "{size} {nw} {ne} {sw} {se}")?;
            }
            DepthQuad::Inner(_depth, inner) => {
                let nw = self.maybe_write_node(&inner.nw)?;
                let ne = self.maybe_write_node(&inner.ne)?;
                let sw = self.maybe_write_node(&inner.sw)?;
                let se = self.maybe_write_node(&inner.se)?;
                writeln!(self.write, "{size} {nw} {ne} {sw} {se}")?;
            }
        }
        Ok(())
    }
    fn maybe_write_node(&mut self, node: &'n Node) -> Result<usize, IoError> {
        match self.nodes.get(node) {
            Some(index) => Ok(*index),
            None if node.is_empty() => Ok(0),
            _ => {
                self.write_node(node)?;
                self.last += 1;
                self.nodes.insert(&node, self.last);
                Ok(self.last)
            }
        }
    }
    fn maybe_write_block(&mut self, block: Block) -> Result<usize, IoError> {
        match self.blocks.get(&block) {
            Some(index) => Ok(*index),
            None if block.is_empty() => Ok(0),
            _ => {
                // TODO don't need to write trailing empty rows
                // let rows = block.to_rows_array();
                // let lead_rows = rows.iter().rev().skip_while(|row| row == 0).rev();
                for row in block.to_rows_array() {
                    self.write_row(row)?;
                }
                writeln!(self.write)?;

                self.last += 1;
                self.blocks.insert(block, self.last);
                Ok(self.last)
            }
        }
    }
    fn write_row(&mut self, row: u8) -> Result<(), IoError> {
        let mut buf = [b'.'; 9];
        let mut dollar = 0;
        for i in 0..8 {
            let bit = (row >> (7 - i)) & 1;
            if bit == 1 {
                buf[i] = b'*';
                dollar = i + 1;
            }
        }
        buf[dollar] = b'$';
        self.write.write_all(&buf[..dollar + 1])
    }
}

// parsing

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MacrocellErrorKind {
    InvalidHeader,
    InvalidContent,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MacrocellErrorHint {
    InvalidHeader,
    TooManyBlockRows,
    TooManyBlockBits,
    InvalidTwoStateDepth,
    SizeTooLarge,
    InvalidForwardRef,
    InvalidRefDepth,
    InvalidBlockAfterBlock,
    InvalidNumberAfterBlock,
    InvalidBlockAfterNumber,
    InvalidNumberAfterNumber,
    InvalidEolAfterNumber,
    InvalidChar,
}

#[derive(Clone, Copy)]
struct MacrocellErrorData<S> {
    line: LineInfo<S>,
    hint: MacrocellErrorHint,
}
#[derive(Clone)]
pub struct MacrocellError<S>(Box<MacrocellErrorData<S>>);
impl<S> MacrocellError<S> {
    fn new(line: LineInfo<S>, hint: MacrocellErrorHint) -> Self {
        Self(Box::new(MacrocellErrorData { line, hint }))
    }
    pub fn kind(&self) -> MacrocellErrorKind {
        match self.0.hint {
            MacrocellErrorHint::InvalidHeader => MacrocellErrorKind::InvalidHeader,
            _ => MacrocellErrorKind::InvalidContent,
        }
    }
    pub fn line_src(&self) -> &S {
        &self.0.line.line_src
    }
    /// 0 based line index
    pub fn line(&self) -> usize {
        self.0.line.line
    }
    /// 0 based column index
    pub fn column(&self) -> usize {
        self.0.line.column
    }
    fn hint_code(&self) -> &MacrocellErrorHint {
        &self.0.hint
    }
    pub fn hint(&self) -> &'static str {
        match self.hint_code() {
            MacrocellErrorHint::InvalidHeader => "Macrocell files start with [M2]",
            MacrocellErrorHint::TooManyBlockRows => {
                "Can't have anything after the 8th '$' in a leaf node"
            }
            MacrocellErrorHint::TooManyBlockBits => "Too many '.'s and '*'s in a row, max of 8",
            MacrocellErrorHint::InvalidTwoStateDepth => {
                "Only handles two-state Macrocell files, use '.'s, '*'s, and '$' for 8x8 leaf nodes"
            }
            MacrocellErrorHint::SizeTooLarge => "Node is too large to be handled",
            MacrocellErrorHint::InvalidForwardRef => {
                "Child nodes must be declared before parent nodes"
            }
            MacrocellErrorHint::InvalidRefDepth => {
                "Child nodes must have a size exactly 1 less than the parent node"
            }
            MacrocellErrorHint::InvalidBlockAfterBlock => {
                "Leaf nodes must be specified on their own line"
            }
            MacrocellErrorHint::InvalidNumberAfterBlock => "Leaf nodes don't reference other nodes",
            MacrocellErrorHint::InvalidBlockAfterNumber => {
                "Leaf nodes must be specified on their own line"
            }
            MacrocellErrorHint::InvalidNumberAfterNumber => "Need exactly 4 child nodes",
            MacrocellErrorHint::InvalidEolAfterNumber => "Need exactly 4 child nodes",
            MacrocellErrorHint::InvalidChar => "Invalid character",
        }
    }
}
// TODO not sure what the right generic constraint should be here to do this the right way
impl<'src> MacrocellError<&'src [u8]> {
    pub fn into_owned(self) -> MacrocellError<Vec<u8>> {
        MacrocellError(Box::new(MacrocellErrorData {
            line: self.0.line.into_owned(),
            hint: self.0.hint,
        }))
    }
}
impl<S: AsRef<[u8]>> Debug for MacrocellError<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MacrocellError")
            .field("line", &self.line())
            .field("column", &self.column())
            .field(
                "line_src",
                &String::from_utf8_lossy(self.line_src().as_ref()),
            )
            .field("hint", &self.hint_code())
            .finish()
    }
}
impl<S: AsRef<[u8]>> Display for MacrocellError<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let line = self.line() + 1;
        // TODO if given a width then limit output src
        let line_src = String::from_utf8_lossy(self.line_src().as_ref());
        let column = self.column() + 1;
        let mark = "^";
        let hint = self.hint();
        writeln!(
            f,
            "Failed to parse macrocell on line {line}:\n{line_src}\n{mark:>column$}\n{hint}\n"
        )
    }
}
impl<S: AsRef<[u8]>> Error for MacrocellError<S> {}

#[derive(Clone, Copy, Debug)]
struct LineInfo<S> {
    line: usize,
    column: usize,
    line_src: S,
}
impl<'src> LineInfo<&'src [u8]> {
    fn from_src_offset(src: &'src [u8], offset: usize) -> Self {
        let mut cr = false;
        let trim_eol = move |line: &'src [u8]| match line {
            [b'\n'] if cr => {
                cr = false;
                None
            }
            [line @ .., b'\n'] => {
                cr = false;
                Some(line)
            }
            [line @ .., b'\r'] => {
                cr = true;
                Some(line)
            }
            // eof may not have a line terminator
            line => Some(line),
        };
        fn offset_of<'src>(src: &'src [u8], line_src: &'src [u8]) -> usize {
            let src_ptr_range = src.as_ptr_range();
            let line_src_ptr = line_src.as_ptr();
            debug_assert!(
                line_src_ptr >= src_ptr_range.start && line_src_ptr <= src_ptr_range.end,
                "line is not in src"
            );
            unsafe { line_src.as_ptr().sub_ptr(src.as_ptr()) }
        }

        let (line, line_offset, line_src) = src
            .split_inclusive(|&b| b == b'\n' || b == b'\r')
            .filter_map(trim_eol)
            .enumerate()
            .map(|(i, line_src)| (i, offset_of(src, line_src), line_src))
            .filter(|(_i, line_offset, _line_src)| line_offset <= &offset)
            .last()
            .expect("split to always produce an element");
        let column = offset - line_offset;
        LineInfo {
            line,
            column,
            line_src,
        }
    }
    fn into_owned(self) -> LineInfo<Vec<u8>> {
        LineInfo {
            line: self.line,
            column: self.column,
            line_src: self.line_src.to_owned(),
        }
    }
}

type MacrocellResult<'src, T> = Result<T, MacrocellError<&'src [u8]>>;

enum Token {
    Block,
    Number,
    Eol,
    Eof,
}

struct McReader<'src> {
    src: &'src [u8],
    at: usize,
    nodes: Vec<Either<Block, Node>>,
}
impl<'src> McReader<'src> {
    fn new(src: &'src [u8]) -> Self {
        Self {
            src,
            at: 0,
            nodes: Vec::new(),
        }
    }

    fn read(mut self) -> MacrocellResult<'src, Node> {
        self.read_header()?;
        self.read_body()
    }
    fn read_header(&mut self) -> MacrocellResult<'src, ()> {
        if &self.src[self.at..][..4] == b"[M2]" {
            self.at += 4;
            self.consume_line();
            Ok(())
        } else {
            self.fail(MacrocellErrorHint::InvalidHeader)
        }
    }
    fn read_body(&mut self) -> MacrocellResult<'src, Node> {
        loop {
            match self.peak_token()? {
                Token::Block => {
                    let block = self.consume_block_line()?;
                    self.nodes.push(Either::Left(block));
                }
                Token::Number => {
                    let node = self.consume_node_line()?;
                    self.nodes.push(Either::Right(node));
                }
                Token::Eol => {
                    self.consume_line();
                }
                Token::Eof => {
                    return Ok(match self.nodes.pop() {
                        Some(Either::Right(node)) => node,
                        // normalize smaller than node patterns into the smallest node
                        Some(Either::Left(block)) => block.expand().into(),
                        None => Node::empty(0),
                    });
                }
            }
        }
    }

    fn consume_node_line(&mut self) -> MacrocellResult<'src, Node> {
        let pos = self.at;
        let size = self.consume_number(
            Node::MAX_WIDTH_LOG2 as usize,
            MacrocellErrorHint::SizeTooLarge,
        )? as u8;
        if size < Node::MIN_WIDTH_LOG2 {
            return self.fail_at(pos, MacrocellErrorHint::InvalidTwoStateDepth);
        }
        let value = if size == Node::MIN_WIDTH_LOG2 {
            let nw = self.expect_block_ref()?;
            let ne = self.expect_block_ref()?;
            let sw = self.expect_block_ref()?;
            let se = self.expect_block_ref()?;
            Node::new(nw, ne, sw, se)
        } else {
            let child_depth = size - Node::MIN_WIDTH_LOG2 - 1;
            let nw = self.expect_node_ref(child_depth)?;
            let ne = self.expect_node_ref(child_depth)?;
            let sw = self.expect_node_ref(child_depth)?;
            let se = self.expect_node_ref(child_depth)?;
            Node::new(nw, ne, sw, se)
        };

        self.expect_line(
            MacrocellErrorHint::InvalidBlockAfterNumber,
            MacrocellErrorHint::InvalidNumberAfterNumber,
        )?;

        Ok(value)
    }
    fn expect_node_ref(&mut self, expected_depth: u8) -> MacrocellResult<'src, Node> {
        self.expect_ref(
            || Node::empty(expected_depth),
            |r| match r {
                Either::Right(node) if node.depth() == expected_depth => Some(node.clone()),
                _ => None,
            },
        )
    }
    fn expect_block_ref(&mut self) -> MacrocellResult<'src, Block> {
        self.expect_ref(Block::empty, |r| match r {
            Either::Left(block) => Some(*block),
            _ => None,
        })
    }
    fn expect_ref<T>(
        &mut self,
        empty: impl Fn() -> T,
        filter: impl Fn(&Either<Block, Node>) -> Option<T>,
    ) -> MacrocellResult<'src, T> {
        match self.peak_token()? {
            Token::Block => return self.fail(MacrocellErrorHint::InvalidBlockAfterNumber),
            Token::Number => {}
            Token::Eol | Token::Eof => return self.fail(MacrocellErrorHint::InvalidEolAfterNumber),
        }
        let pos = self.at;
        let index = self.consume_number(self.nodes.len(), MacrocellErrorHint::InvalidForwardRef)?;
        if index == 0 {
            Ok(empty())
        } else if let Some(value) = filter(&self.nodes[index - 1]) {
            Ok(value)
        } else {
            self.fail_at(pos, MacrocellErrorHint::InvalidRefDepth)
        }
    }
    fn consume_number(
        &mut self,
        max: usize,
        too_large_hint: MacrocellErrorHint,
    ) -> MacrocellResult<'src, usize> {
        let pos = self.at;
        let mut value = 0_usize;
        while let Some(b @ b'0'..=b'9') = self.peak() {
            self.consume();
            let digit = (b - b'0') as usize;
            let new_value = value.wrapping_mul(10).wrapping_add(digit);
            if new_value > max || new_value < value {
                return self.fail_at(pos, too_large_hint);
            }
            value = new_value;
        }
        Ok(value)
    }

    fn consume_block_line(&mut self) -> MacrocellResult<'src, Block> {
        let mut rows = [0_u8; 8];
        let mut r = 0;
        let mut i = 8;

        loop {
            match self.peak() {
                Some(b'$') => {
                    if r >= 8 {
                        return self.fail(MacrocellErrorHint::TooManyBlockRows);
                    }
                    self.consume();
                    r += 1;
                    i = 8;
                }
                Some(b'.') => {
                    if r >= 8 {
                        return self.fail(MacrocellErrorHint::TooManyBlockRows);
                    }
                    if i == 0 {
                        return self.fail(MacrocellErrorHint::TooManyBlockBits);
                    }
                    self.consume();
                    i -= 1;
                }
                Some(b'*') => {
                    if r >= 8 {
                        return self.fail(MacrocellErrorHint::TooManyBlockRows);
                    }
                    if i == 0 {
                        return self.fail(MacrocellErrorHint::TooManyBlockBits);
                    }
                    self.consume();
                    i -= 1;
                    rows[r] |= 1 << i;
                }
                _ => break,
            }
        }

        self.expect_line(
            MacrocellErrorHint::InvalidBlockAfterBlock,
            MacrocellErrorHint::InvalidNumberAfterBlock,
        )?;

        Ok(Block::from_rows_array(rows))
    }

    fn expect_line(
        &mut self,
        block_hint: MacrocellErrorHint,
        number_hint: MacrocellErrorHint,
    ) -> MacrocellResult<'src, ()> {
        match self.peak_token()? {
            Token::Block => self.fail(block_hint),
            Token::Number => self.fail(number_hint),
            Token::Eol => {
                self.consume_line();
                Ok(())
            }
            Token::Eof => Ok(()),
        }
    }
    fn consume_line(&mut self) {
        loop {
            let b = self.peak();
            self.consume();
            match b {
                None | Some(b'\n' | b'\r') => return,
                _ => {} // ignore comments
            }
        }
    }

    /// consuming any insignificant white space
    fn peak_token(&mut self) -> MacrocellResult<'src, Token> {
        loop {
            match self.peak() {
                Some(b' ' | b'\t') => {
                    self.consume();
                }
                Some(b'.' | b'*' | b'$') => return Ok(Token::Block),
                Some(b'0'..=b'9') => return Ok(Token::Number),
                Some(b'\n' | b'\r' | b'#') => return Ok(Token::Eol),
                None => return Ok(Token::Eof),
                _ => return self.fail(MacrocellErrorHint::InvalidChar),
            }
        }
    }
    fn peak(&self) -> Option<u8> {
        self.src.get(self.at).copied()
    }
    fn consume(&mut self) {
        self.at += 1;
    }

    fn fail<T>(&self, hint: MacrocellErrorHint) -> MacrocellResult<'src, T> {
        self.fail_at(self.at, hint)
    }
    fn fail_at<T>(&self, at: usize, hint: MacrocellErrorHint) -> MacrocellResult<'src, T> {
        Err(MacrocellError::new(
            LineInfo::from_src_offset(self.src, at),
            hint,
        ))
    }
}

// tests

#[cfg(test)]
mod test {
    use unindent::unindent;

    use crate::{Block, Node};

    fn assert_node_fmt(node: Node, fmt: &str) {
        let mut fmt = unindent(fmt);
        fmt.insert_str(0, "[M2] (metalife 1.0)\n#R B3/S23\n");
        assert_eq!(node.write_to_string(), fmt);
        assert_eq!(Node::read_from_string(&fmt).expect("valid input"), node);
    }

    #[test]
    fn empty() {
        // writes size of node even for empty nodes
        // isn't necessary for golly, but round trips without extra info
        assert_node_fmt(Node::empty(0), "4 0 0 0 0\n");
        assert_node_fmt(Node::empty(1), "5 0 0 0 0\n");
        assert_node_fmt(Node::empty(59), "63 0 0 0 0\n");
    }

    #[test]
    fn small() {
        let b0 = Block::from_rows(0x00_00_00_00_00_00_00_00);
        let b1 = Block::from_rows(0x80_00_00_00_00_00_00_00);
        let b2 = Block::from_rows(0x00_00_00_00_00_00_00_01);
        let b3 = Block::from_rows(0x80_40_20_10_08_04_02_01);
        assert_node_fmt(
            Node::new(b1, b1, b1, b1),
            "
                *$$$$$$$$
                4 1 1 1 1
            ",
        );
        assert_node_fmt(
            Node::new(b0, b1, b2, b3),
            "
                *$$$$$$$$
                $$$$$$$.......*$
                *$.*$..*$...*$....*$.....*$......*$.......*$
                4 0 1 2 3
            ",
        );
    }
}
