use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};
use crate::app::TreeNode;
use crate::decoder::{TxView, PlutusNode, AssetView};

#[derive(Debug, Clone)]
pub struct DetailWidget<'a> {
    block: Option<Block<'a>>,
}

impl<'a> DetailWidget<'a> {
    pub fn new() -> Self {
        Self { block: None }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl<'a> Widget for DetailWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = self.block.unwrap_or_else(|| {
            Block::default()
                .title("DETAIL")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue))
        });

        block.render(area, buf);
    }
}

pub fn render_detail_content(
    area: Rect,
    buf: &mut Buffer,
    node: Option<&TreeNode>,
    tx: Option<&TxView>,
    scroll: usize,
) {
    let content_area = area.inner(ratatui::layout::Margin::new(1, 1));
    
    let text = if let (Some(node), Some(tx)) = (node, tx) {
        render_node_detail(node, tx)
    } else {
        Text::from("Select a node to view details")
    };

    let paragraph = Paragraph::new(text)
        .wrap(Wrap { trim: true })
        .scroll((scroll as u16, 0));

    paragraph.render(content_area, buf);
}

fn render_node_detail(node: &TreeNode, tx: &TxView) -> Text<'static> {
    match node {
        TreeNode::InputsHeader { count, expanded } => {
            let lines = vec![
                Line::from(Span::styled(
                    format!(" Inputs ({})", count),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(format!("Status: {}", if *expanded { "Expanded" } else { "Collapsed" })),
                Line::from(""),
                Line::from("Press → to expand/collapse"),
            ];
            Text::from(lines)
        }
        
        TreeNode::Input { index, .. } => {
            if let Some(input) = tx.inputs.get(*index) {
                let mut lines = vec![
                    Line::from(Span::styled(
                        format!(" Input #{}", index),
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    )),
                    Line::from(""),
                    Line::from(Span::styled("Transaction:", Style::default().fg(Color::Gray))),
                    Line::from(input.tx_hash.clone()),
                    Line::from(""),
                    Line::from(Span::styled("Index:", Style::default().fg(Color::Gray))),
                    Line::from(input.index.to_string()),
                    Line::from(""),
                    Line::from(Span::styled("Address:", Style::default().fg(Color::Gray))),
                ];
                
                // Wrap long address
                for chunk in input.address.as_bytes().chunks(64) {
                    lines.push(Line::from(String::from_utf8_lossy(chunk).to_string()));
                }
                
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled("Value:", Style::default().fg(Color::Gray))));
                
                for asset in &input.value {
                    lines.push(Line::from(format!("  • {}", format_asset(asset))));
                }
                
                if let Some(datum) = &input.datum {
                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled("Datum Hash:", Style::default().fg(Color::Gray))));
                    lines.push(Line::from(datum.raw_cbor.clone()));
                }
                
                Text::from(lines)
            } else {
                Text::from("Input not found")
            }
        }
        
        TreeNode::InputDatum { input_index } => {
            if let Some(input) = tx.inputs.get(*input_index) {
                if let Some(datum) = &input.datum {
                    render_datum_detail(&datum.raw_cbor, &datum.decoded)
                } else {
                    Text::from("No datum for this input")
                }
            } else {
                Text::from("Input not found")
            }
        }
        
        TreeNode::OutputsHeader { count, expanded } => {
            let lines = vec![
                Line::from(Span::styled(
                    format!(" Outputs ({})", count),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(format!("Status: {}", if *expanded { "Expanded" } else { "Collapsed" })),
                Line::from(""),
                Line::from("Press → to expand/collapse"),
            ];
            Text::from(lines)
        }
        
        TreeNode::Output { index, .. } => {
            if let Some(output) = tx.outputs.get(*index) {
                let mut lines = vec![
                    Line::from(Span::styled(
                        format!(" Output #{}", index),
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    )),
                    Line::from(""),
                    Line::from(Span::styled("Address:", Style::default().fg(Color::Gray))),
                ];
                
                for chunk in output.address.as_bytes().chunks(64) {
                    lines.push(Line::from(String::from_utf8_lossy(chunk).to_string()));
                }
                
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled("Value:", Style::default().fg(Color::Gray))));
                
                for asset in &output.value {
                    lines.push(Line::from(format!("  • {}", format_asset(asset))));
                }
                
                if let Some(script_ref) = &output.script_ref {
                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled("Script Reference:", Style::default().fg(Color::Gray))));
                    lines.push(Line::from(script_ref.clone()));
                }
                
                if let Some(datum) = &output.datum {
                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled("Datum:", Style::default().fg(Color::Gray))));
                    lines.push(Line::from(format!("Type: {}", datum.decoded.type_name())));
                }
                
                Text::from(lines)
            } else {
                Text::from("Output not found")
            }
        }
        
        TreeNode::OutputDatum { output_index } => {
            if let Some(output) = tx.outputs.get(*output_index) {
                if let Some(datum) = &output.datum {
                    render_datum_detail(&datum.raw_cbor, &datum.decoded)
                } else {
                    Text::from("No datum for this output")
                }
            } else {
                Text::from("Output not found")
            }
        }
        
        TreeNode::RedeemersHeader { count, expanded } => {
            let lines = vec![
                Line::from(Span::styled(
                    format!(" Redeemers ({})", count),
                    Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(format!("Status: {}", if *expanded { "Expanded" } else { "Collapsed" })),
                Line::from(""),
                Line::from("Press → to expand/collapse"),
            ];
            Text::from(lines)
        }
        
        TreeNode::Redeemer { index, .. } => {
            if let Some(redeemer) = tx.redeemers.get(*index) {
                let mut lines = vec![
                    Line::from(Span::styled(
                        format!(" {} Redeemer #{}", redeemer.tag, index),
                        Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
                    )),
                    Line::from(""),
                    Line::from(Span::styled("Tag:", Style::default().fg(Color::Gray))),
                    Line::from(redeemer.tag.clone()),
                    Line::from(""),
                    Line::from(Span::styled("Index:", Style::default().fg(Color::Gray))),
                    Line::from(redeemer.index.to_string()),
                    Line::from(""),
                    Line::from(Span::styled("Execution Units:", Style::default().fg(Color::Gray))),
                    Line::from(format!("  Memory: {}", redeemer.ex_units.0)),
                    Line::from(format!("  Steps:  {}", redeemer.ex_units.1)),
                    Line::from(""),
                    Line::from(Span::styled("Data:", Style::default().fg(Color::Gray))),
                ];
                
                // Add formatted Plutus data
                for line in redeemer.data.to_string_pretty().lines() {
                    lines.push(Line::from(line.to_string()));
                }
                
                Text::from(lines)
            } else {
                Text::from("Redeemer not found")
            }
        }
        
        TreeNode::Metadata { expanded } => {
            let mut lines = vec![
                Line::from(Span::styled(
                    " Metadata",
                    Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
            ];
            
            if let Some(metadata) = &tx.metadata {
                if *expanded {
                    if let Ok(pretty) = serde_json::to_string_pretty(metadata) {
                        for line in pretty.lines() {
                            lines.push(Line::from(line.to_string()));
                        }
                    }
                } else {
                    lines.push(Line::from("Press → to expand"));
                }
            } else {
                lines.push(Line::from("No metadata"));
            }
            
            Text::from(lines)
        }
    }
}

fn render_datum_detail(raw_cbor: &str, decoded: &PlutusNode) -> Text<'static> {
    let mut lines = vec![
        Line::from(Span::styled(
            " Datum",
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled("Raw CBOR:", Style::default().fg(Color::Gray))),
        Line::from(raw_cbor.to_string()),
        Line::from(""),
        Line::from(Span::styled("Type:", Style::default().fg(Color::Gray))),
        Line::from(decoded.type_name().to_string()),
        Line::from(""),
        Line::from(Span::styled("Decoded:", Style::default().fg(Color::Gray))),
    ];
    
    for line in decoded.to_string_pretty().lines() {
        lines.push(Line::from(line.to_string()));
    }
    
    Text::from(lines)
}

fn format_asset(asset: &AssetView) -> String {
    if asset.policy_id == "ada" {
        let ada = asset.amount as f64 / 1_000_000.0;
        format!("₳ {:.6}", ada)
    } else {
        let policy_short = if asset.policy_id.len() > 16 {
            format!("{}...{}", &asset.policy_id[..8], &asset.policy_id[asset.policy_id.len()-8..])
        } else {
            asset.policy_id.clone()
        };
        
        let asset_name = if asset.asset_name.is_empty() {
            "(no name)".to_string()
        } else if asset.asset_name.len() > 32 {
            format!("{}...", &asset.asset_name[..28])
        } else {
            asset.asset_name.clone()
        };
        
        format!("{} {} ({})", asset.amount, asset_name, policy_short)
    }
}