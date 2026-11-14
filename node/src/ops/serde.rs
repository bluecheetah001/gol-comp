use serde::de::Visitor;
use serde::{Deserialize, Serialize};

use crate::Node;

impl Serialize for Node {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.write_to_string())
    }
}
impl<'de> Deserialize<'de> for Node {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct NodeVisitor;
        impl Visitor<'_> for NodeVisitor {
            type Value = Node;

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Node::read_from_string(v).map_err(E::custom)
            }

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "Macrocell string")
            }
        }
        deserializer.deserialize_str(NodeVisitor)
    }
}
