use std::num::NonZeroU32;
use std::rc::Rc;

struct State(u32);

struct Leaf {
    nw: State,
    ne: State,
    sw: State,
    se: State,
}
struct Inner {
    depth: NonZeroU32,
    nw: Rc<Node>,
    ne: Rc<Node>,
    sw: Rc<Node>,
    se: Rc<Node>,
    f: Rc<Node>,
}
enum Node {
    Leaf(Leaf),
    Inner(Inner),
}
impl Node {
    fn depth(&self) -> u32 {
        match &self {
            Node::Leaf(_) => 0,
            Node::Inner(Inner { depth, .. }) => depth.get(),
        }
    }
}



struct NodeArena{
    
}