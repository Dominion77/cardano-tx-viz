use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxView {
    pub hash: String,
    pub inputs: Vec<InputView>,
    pub outputs: Vec<OutputView>,
    pub redeemers: Vec<RedeemerView>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputView {
    pub tx_hash: String,
    pub index: u32,
    pub address: String,
    pub value: Vec<AssetView>,
    pub datum: Option<DatumView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputView {
    pub address: String,
    pub value: Vec<AssetView>,
    pub datum: Option<DatumView>,
    pub script_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatumView {
    pub raw_cbor: String,
    pub decoded: PlutusNode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedeemerView {
    pub tag: String,
    pub index: u32,
    pub data: PlutusNode,
    pub ex_units: (u64, u64),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssetView {
    pub policy_id: String,
    pub asset_name: String,
    pub amount: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PlutusNode {
    Constr(u64, Vec<PlutusNode>),
    Map(Vec<(PlutusNode, PlutusNode)>),
    List(Vec<PlutusNode>),
    Int(i128),
    Bytes(String), // hex-encoded
    Text(String),
}

impl PlutusNode {
    pub fn type_name(&self) -> &'static str {
        match self {
            PlutusNode::Constr(_, _) => "Constructor",
            PlutusNode::Map(_) => "Map",
            PlutusNode::List(_) => "List",
            PlutusNode::Int(_) => "Integer",
            PlutusNode::Bytes(_) => "Bytes",
            PlutusNode::Text(_) => "Text",
        }
    }

    pub fn to_string_pretty(&self) -> String {
        self.to_string_indent(0)
    }

    fn to_string_indent(&self, indent: usize) -> String {
        let indent_str = "  ".repeat(indent);
        match self {
            PlutusNode::Constr(tag, fields) => {
                let mut result = format!("{}Constr {} [", indent_str, tag);
                if fields.is_empty() {
                    result.push(']');
                } else {
                    result.push('\n');
                    for (i, field) in fields.iter().enumerate() {
                        result.push_str(&field.to_string_indent(indent + 1));
                        if i < fields.len() - 1 {
                            result.push_str(",\n");
                        }
                    }
                    result.push_str(&format!("\n{}]", indent_str));
                }
                result
            }
            PlutusNode::Map(entries) => {
                let mut result = format!("{}Map {{", indent_str);
                if entries.is_empty() {
                    result.push('}');
                } else {
                    result.push('\n');
                    for (i, (k, v)) in entries.iter().enumerate() {
                        result.push_str(&format!(
                            "{}  {}: {}",
                            indent_str,
                            k.to_string_compact(),
                            v.to_string_compact()
                        ));
                        if i < entries.len() - 1 {
                            result.push_str(",\n");
                        }
                    }
                    result.push_str(&format!("\n{}}}", indent_str));
                }
                result
            }
            PlutusNode::List(items) => {
                let mut result = format!("{}List [", indent_str);
                if items.is_empty() {
                    result.push(']');
                } else {
                    result.push('\n');
                    for (i, item) in items.iter().enumerate() {
                        result.push_str(&item.to_string_indent(indent + 1));
                        if i < items.len() - 1 {
                            result.push_str(",\n");
                        }
                    }
                    result.push_str(&format!("\n{}]", indent_str));
                }
                result
            }
            PlutusNode::Int(n) => format!("{}Int({})", indent_str, n),
            PlutusNode::Bytes(hex) => format!("{}Bytes(\"{}\")", indent_str, hex),
            PlutusNode::Text(s) => format!("{}Text(\"{}\")", indent_str, s),
        }
    }

    pub fn to_string_compact(&self) -> String {
        match self {
            PlutusNode::Constr(tag, fields) => {
                if fields.is_empty() {
                    format!("Constr {} []", tag)
                } else {
                    let fields_str: Vec<String> =
                        fields.iter().map(|f| f.to_string_compact()).collect();
                    format!("Constr {} [{}]", tag, fields_str.join(", "))
                }
            }
            PlutusNode::Map(entries) => {
                if entries.is_empty() {
                    "Map {}".to_string()
                } else {
                    let entries_str: Vec<String> = entries
                        .iter()
                        .map(|(k, v)| {
                            format!("{}: {}", k.to_string_compact(), v.to_string_compact())
                        })
                        .collect();
                    format!("Map {{ {} }}", entries_str.join(", "))
                }
            }
            PlutusNode::List(items) => {
                if items.is_empty() {
                    "List []".to_string()
                } else {
                    let items_str: Vec<String> =
                        items.iter().map(|i| i.to_string_compact()).collect();
                    format!("List [{}]", items_str.join(", "))
                }
            }
            PlutusNode::Int(n) => format!("Int({})", n),
            PlutusNode::Bytes(hex) => format!("Bytes(\"{}\")", hex),
            PlutusNode::Text(s) => format!("Text(\"{}\")", s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plutus_node_pretty_print() {
        let node = PlutusNode::Constr(
            0,
            vec![
                PlutusNode::Int(1000000),
                PlutusNode::Bytes("deadbeef".to_string()),
            ],
        );
        let pretty = node.to_string_pretty();
        assert!(pretty.contains("Constr 0 ["));
        assert!(pretty.contains("Int(1000000)"));
        assert!(pretty.contains("Bytes(\"deadbeef\")"));
    }

    #[test]
    fn test_plutus_node_compact_print() {
        let node = PlutusNode::List(vec![
            PlutusNode::Int(1),
            PlutusNode::Int(2),
            PlutusNode::Int(3),
        ]);
        assert_eq!(node.to_string_compact(), "List [Int(1), Int(2), Int(3)]");
    }
}
