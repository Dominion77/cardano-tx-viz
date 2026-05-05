use crate::app::TreeNode;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, StatefulWidget, Widget},
};

#[derive(Debug, Clone)]
pub struct TxTreeWidget<'a> {
    block: Option<Block<'a>>,
}

impl<'a> Default for TxTreeWidget<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> TxTreeWidget<'a> {
    pub fn new() -> Self {
        Self { block: None }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl<'a> StatefulWidget for TxTreeWidget<'a> {
    type State = TreeState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let block = self.block.unwrap_or_else(|| {
            Block::default()
                .title("TX TREE")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue))
        });

        let inner_area = block.inner(area);
        block.render(area, buf);

        if state.visible_nodes.is_empty() {
            return;
        }

        // Calculate visible range based on scroll
        let list_height = inner_area.height as usize;
        let start = state.scroll_offset;
        let end = (start + list_height).min(state.visible_nodes.len());

        // Render visible nodes
        for (idx, node_idx) in (start..end).enumerate() {
            if node_idx >= state.visible_nodes.len() {
                break;
            }

            let node = &state.visible_nodes[node_idx];
            let y = inner_area.y + idx as u16;

            if y >= area.bottom() {
                break;
            }

            let depth = node.depth();
            let indent = "  ".repeat(depth);
            let display_text = node.display_text();

            let style = if node_idx == state.selected_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::DarkGray)
            } else {
                match node {
                    TreeNode::InputsHeader { .. }
                    | TreeNode::OutputsHeader { .. }
                    | TreeNode::RedeemersHeader { .. }
                    | TreeNode::Metadata { .. } => Style::default().fg(Color::Cyan),
                    TreeNode::Input { .. }
                    | TreeNode::Output { .. }
                    | TreeNode::Redeemer { .. } => Style::default().fg(Color::White),
                    TreeNode::InputDatum { .. } | TreeNode::OutputDatum { .. } => {
                        Style::default().fg(Color::Green)
                    }
                }
            };

            let line = Line::from(Span::styled(format!("{}{}", indent, display_text), style));

            buf.set_line(inner_area.x, y, &line, inner_area.width);
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TreeState {
    pub selected_index: usize,
    pub expanded: Vec<bool>,
    pub visible_nodes: Vec<TreeNode>,
    pub scroll_offset: usize,
}

impl TreeState {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            expanded: Vec::new(),
            visible_nodes: Vec::new(),
            scroll_offset: 0,
        }
    }

    pub fn rebuild_from_nodes(&mut self, all_nodes: Vec<TreeNode>, expanded: Vec<bool>) {
        self.expanded = expanded;
        self.visible_nodes = self.filter_visible_nodes(&all_nodes);
    }

    fn filter_visible_nodes(&self, all_nodes: &[TreeNode]) -> Vec<TreeNode> {
        let mut visible = Vec::new();
        let mut skip_until_next_sibling = false;
        let mut current_depth = 0;

        for (idx, node) in all_nodes.iter().enumerate() {
            if skip_until_next_sibling {
                if node.depth() <= current_depth {
                    skip_until_next_sibling = false;
                } else {
                    continue;
                }
            }

            visible.push(node.clone());

            let should_show_children = idx < self.expanded.len() && self.expanded[idx];

            if !should_show_children {
                skip_until_next_sibling = true;
                current_depth = node.depth();
            }
        }

        visible
    }

    pub fn toggle_expand(&mut self, all_nodes: &[TreeNode]) {
        if self.selected_index < self.expanded.len() {
            self.expanded[self.selected_index] = !self.expanded[self.selected_index];
            self.visible_nodes = self.filter_visible_nodes(all_nodes);
        }
    }

    pub fn select_next(&mut self) {
        if self.selected_index + 1 < self.visible_nodes.len() {
            self.selected_index += 1;
            self.ensure_selected_visible();
        }
    }

    pub fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.ensure_selected_visible();
        }
    }

    fn ensure_selected_visible(&mut self) {
        // Adjust scroll offset to keep selected item visible
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        }
        // Note: height-based adjustment happens during render
    }

    pub fn update_scroll(&mut self, visible_height: usize) {
        if self.selected_index >= self.scroll_offset + visible_height {
            self.scroll_offset = self.selected_index.saturating_sub(visible_height - 1);
        } else if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        }
    }
}
