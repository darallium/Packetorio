use crate::core::packet::Protocol;
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operand {
    Equal,
    LessThan,
    GreaterThan,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortRule {
    pub port: u16,
    pub operand: Operand,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LengthRule {
    pub length: u32,
    pub operand: Operand,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentMatchType {
    Partial,
    Exact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentRule {
    pub content: Vec<u8>,
    pub match_type: ContentMatchType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterRule {
    Ip(IpNetwork),
    Port(PortRule),
    Length(LengthRule),
    Protocol(Protocol),
    Content(ContentRule),
}
