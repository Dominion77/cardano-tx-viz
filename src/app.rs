use anyhow::{Context, Result};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use std::time::{Duration, Instant};

use crate::decoder::{TxParser, TxView};
use crate::fetcher::{FetcherConfig, Network, TxFetcher};

#[derive(Debug, Clone)]
pub enum FetchState {
    Idle,
    Loading,
    Done(TxView),
    Error(String),
}

#[derive(Debug)]
pub enum AppEvent {
    FetchComplete(Result<TxView>),
    DatumResolved { hash: String, data: Vec<u8> },
    Error(String),
    Tick,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
}

#[derive(Debug, Clone)]
pub struct TreeState {
    pub selected_index: usize,
    pub expanded: Vec<bool>,
    pub visible_nodes: Vec<TreeNode>,
}

#[derive(Debug, Clone)]
pub enum TreeNode {
    InputsHeader { expanded: bool, count: usize },
    Input { index: usize, tx_hash: String, address: String },
    InputDatum { input_index: usize },
    OutputsHeader { expanded: bool, count: usize },
    Output { index: usize, address: String },
    OutputDatum { output_index: usize },
    RedeemersHeader { expanded: bool, count: usize },
    Redeemer { index: usize, tag: String },
    Metadata { expanded: bool },
}

impl TreeNode {
    pub fn display_text(&self) -> String {
        match self {
            TreeNode::InputsHeader { expanded, count } => {
                format!("{} Inputs ({})", if *expanded { "▼" } else { "▶" }, count)
            }
            TreeNode::Input { index, tx_hash, .. } => {
                format!("  Input #{}: {}", index, &tx_hash[..8.min(tx_hash.len())])
            }
            TreeNode::InputDatum { input_index } => {
                format!("    Datum for Input #{}", input_index)
            }
            TreeNode::OutputsHeader { expanded, count } => {
                format!("{} Outputs ({})", if *expanded { "▼" } else { "▶" }, count)
            }
            TreeNode::Output { index, address } => {
                let addr_preview = if address.len() > 20 {
                    format!("{}...{}", &address[..10], &address[address.len()-10..])
                } else {
                    address.clone()
                };
                format!("  Output #{}: {}", index, addr_preview)
            }
            TreeNode::OutputDatum { output_index } => {
                format!("    Datum for Output #{}", output_index)
            }
            TreeNode::RedeemersHeader { expanded, count } => {
                format!("{} Redeemers ({})", if *expanded { "▼" } else { "▶" }, count)
            }
            TreeNode::Redeemer { index, tag } => {
                format!("  {} Redeemer #{}", tag, index)
            }
            TreeNode::Metadata { expanded } => {
                format!("{} Metadata", if *expanded { "▼" } else { "▶" })
            }
        }
    }

    pub fn depth(&self) -> usize {
        match self {
            TreeNode::InputsHeader { .. } => 0,
            TreeNode::Input { .. } => 1,
            TreeNode::InputDatum { .. } => 2,
            TreeNode::OutputsHeader { .. } => 0,
            TreeNode::Output { .. } => 1,
            TreeNode::OutputDatum { .. } => 2,
            TreeNode::RedeemersHeader { .. } => 0,
            TreeNode::Redeemer { .. } => 1,
            TreeNode::Metadata { .. } => 0,
        }
    }
}

pub struct App {
    pub fetch_state: FetchState,
    pub input_hash: String,
    pub input_mode: InputMode,
    pub cursor_position: usize,
    pub network: Network,
    pub fetcher: Box<dyn TxFetcher>,
    pub tx_parser: TxParser,
    pub tree_state: TreeState,
    pub detail_scroll: usize,
    pub status_message: Option<String>,
    pub spinner_frame: usize,
    last_tick: Instant,
    event_tx: mpsc::UnboundedSender<AppEvent>,
    event_rx: mpsc::UnboundedReceiver<AppEvent>,
}

impl App {
    pub fn new(network: Network, fetcher_config: FetcherConfig) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        
        Self {
            fetch_state: FetchState::Idle,
            input_hash: String::new(),
            input_mode: InputMode::Normal,
            cursor_position: 0,
            network,
            fetcher: fetcher_config.create_fetcher(),
            tx_parser: TxParser::new(),
            tree_state: TreeState {
                selected_index: 0,
                expanded: Vec::new(),
                visible_nodes: Vec::new(),
            },
            detail_scroll: 0,
            status_message: None,
            spinner_frame: 0,
            last_tick: Instant::now(),
            event_tx,
            event_rx,
        }
    }

    pub fn start_fetch(&mut self, hash: String) {
        if hash.is_empty() {
            self.status_message = Some("Transaction hash cannot be empty".to_string());
            return;
        }
        
        self.fetch_state = FetchState::Loading;
        self.input_hash = hash.clone();
        self.status_message = Some(format!("Fetching transaction {}...", hash));
        
        let fetcher = self.fetcher.clone_box();
        let event_tx = self.event_tx.clone();
        
        tokio::spawn(async move {
            let result = fetch_transaction(fetcher, &hash).await;
            let _ = event_tx.send(AppEvent::FetchComplete(result));
        });
        
        info!("Started fetch for transaction: {}", hash);
    }

    pub fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::FetchComplete(result) => {
                match result {
                    Ok(tx_view) => {
                        info!("Successfully fetched transaction: {}", tx_view.hash);
                        self.build_tree_from_tx(&tx_view);
                        self.fetch_state = FetchState::Done(tx_view);
                        self.status_message = Some(format!("✓ Transaction {} loaded", self.input_hash));
                        self.input_mode = InputMode::Normal;
                    }
                    Err(e) => {
                        error!("Failed to fetch transaction: {}", e);
                        self.fetch_state = FetchState::Error(e.to_string());
                        self.status_message = Some(format!("✗ Error: {}", e));
                    }
                }
            }
            AppEvent::DatumResolved { hash, data } => {
                debug!("Resolved datum hash: {}", hash);
                self.tx_parser.resolve_datum_hash(&hash, data);
                // Refresh tree if we have a current transaction
                if let FetchState::Done(tx) = &self.fetch_state {
                    let tx_clone = tx.clone();
                    self.build_tree_from_tx(&tx_clone);
                }
            }
            AppEvent::Error(msg) => {
                error!("App error: {}", msg);
                self.status_message = Some(format!("✗ {}", msg));
            }
            AppEvent::Tick => {
                self.spinner_frame = (self.spinner_frame + 1) % 10;
                self.last_tick = Instant::now();
            }
        }
    }

    fn build_tree_from_tx(&mut self, tx: &TxView) {
        let mut nodes = Vec::new();
        let mut expanded = Vec::new();
        
        // Inputs header
        nodes.push(TreeNode::InputsHeader { 
            expanded: true, 
            count: tx.inputs.len() 
        });
        expanded.push(true);
        
        // Inputs
        for (idx, input) in tx.inputs.iter().enumerate() {
            nodes.push(TreeNode::Input { 
                index: idx, 
                tx_hash: input.tx_hash.clone(), 
                address: input.address.clone() 
            });
            expanded.push(false);
            
            if input.datum.is_some() {
                nodes.push(TreeNode::InputDatum { input_index: idx });
                expanded.push(false);
            }
        }
        
        // Outputs header
        nodes.push(TreeNode::OutputsHeader { 
            expanded: true, 
            count: tx.outputs.len() 
        });
        expanded.push(true);
        
        // Outputs
        for (idx, output) in tx.outputs.iter().enumerate() {
            nodes.push(TreeNode::Output { 
                index: idx, 
                address: output.address.clone() 
            });
            expanded.push(false);
            
            if output.datum.is_some() {
                nodes.push(TreeNode::OutputDatum { output_index: idx });
                expanded.push(false);
            }
        }
        
        // Redeemers header
        if !tx.redeemers.is_empty() {
            nodes.push(TreeNode::RedeemersHeader { 
                expanded: true, 
                count: tx.redeemers.len() 
            });
            expanded.push(true);
            
            for (idx, redeemer) in tx.redeemers.iter().enumerate() {
                nodes.push(TreeNode::Redeemer { 
                    index: idx, 
                    tag: redeemer.tag.clone() 
                });
                expanded.push(false);
            }
        }
        
        // Metadata
        if tx.metadata.is_some() {
            nodes.push(TreeNode::Metadata { expanded: false });
            expanded.push(false);
        }
        
        self.tree_state.visible_nodes = nodes;
        self.tree_state.expanded = expanded;
        self.tree_state.selected_index = 0;
    }

    pub fn poll_event(&mut self) -> Option<AppEvent> {
        self.event_rx.try_recv().ok()
    }

    pub fn tick(&mut self) {
        if self.last_tick.elapsed() >= Duration::from_millis(100) {
            let _ = self.event_tx.send(AppEvent::Tick);
        }
    }

    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::{KeyCode, KeyModifiers};
        
        match self.input_mode {
            InputMode::Normal => self.handle_normal_key(key),
            InputMode::Editing => self.handle_editing_key(key),
        }
    }

    fn handle_normal_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::{KeyCode, KeyModifiers};
        
        match key.code {
            KeyCode::Char('/') | KeyCode::Char('i') => {
                self.input_mode = InputMode::Editing;
                self.input_hash.clear();
                self.cursor_position = 0;
                self.status_message = Some("Enter transaction hash...".to_string());
            }
            KeyCode::Enter => {
                if !self.input_hash.is_empty() {
                    self.start_fetch(self.input_hash.clone());
                }
            }
            KeyCode::Up => {
                if self.tree_state.selected_index > 0 {
                    self.tree_state.selected_index -= 1;
                }
            }
            KeyCode::Down => {
                if self.tree_state.selected_index + 1 < self.tree_state.visible_nodes.len() {
                    self.tree_state.selected_index += 1;
                }
            }
            KeyCode::Right | KeyCode::Char(' ') => {
                if let Some(node) = self.tree_state.visible_nodes.get(self.tree_state.selected_index) {
                    let should_expand = match node {
                        TreeNode::InputsHeader { .. } => true,
                        TreeNode::OutputsHeader { .. } => true,
                        TreeNode::RedeemersHeader { .. } => true,
                        TreeNode::Metadata { .. } => true,
                        _ => false,
                    };
                    
                    if should_expand {
                        self.tree_state.expanded[self.tree_state.selected_index] = 
                            !self.tree_state.expanded[self.tree_state.selected_index];
                        // Update node's expanded state
                        match &mut self.tree_state.visible_nodes[self.tree_state.selected_index] {
                            TreeNode::InputsHeader { expanded, .. } => *expanded = !*expanded,
                            TreeNode::OutputsHeader { expanded, .. } => *expanded = !*expanded,
                            TreeNode::RedeemersHeader { expanded, .. } => *expanded = !*expanded,
                            TreeNode::Metadata { expanded } => *expanded = !*expanded,
                            _ => {}
                        }
                    }
                }
            }
            KeyCode::Left => {
                if let Some(node) = self.tree_state.visible_nodes.get(self.tree_state.selected_index) {
                    let should_collapse = match node {
                        TreeNode::InputsHeader { expanded: true, .. } => true,
                        TreeNode::OutputsHeader { expanded: true, .. } => true,
                        TreeNode::RedeemersHeader { expanded: true, .. } => true,
                        TreeNode::Metadata { expanded: true } => true,
                        _ => false,
                    };
                    
                    if should_collapse {
                        self.tree_state.expanded[self.tree_state.selected_index] = false;
                        match &mut self.tree_state.visible_nodes[self.tree_state.selected_index] {
                            TreeNode::InputsHeader { expanded, .. } => *expanded = false,
                            TreeNode::OutputsHeader { expanded, .. } => *expanded = false,
                            TreeNode::RedeemersHeader { expanded, .. } => *expanded = false,
                            TreeNode::Metadata { expanded } => *expanded = false,
                            _ => {}
                        }
                    }
                }
            }
            KeyCode::Char('c') => {
                self.copy_selected_to_clipboard();
            }
            KeyCode::Char('p') => {
                self.copy_policy_id_to_clipboard();
            }
            KeyCode::Char('r') => {
                self.copy_raw_hex_to_clipboard();
            }
            KeyCode::PageUp => {
                if self.detail_scroll > 0 {
                    self.detail_scroll = self.detail_scroll.saturating_sub(5);
                }
            }
            KeyCode::PageDown => {
                self.detail_scroll += 5;
            }
            _ => {}
        }
    }

    fn handle_editing_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::{KeyCode, KeyModifiers};
        
        match key.code {
            KeyCode::Enter => {
                self.input_mode = InputMode::Normal;
                if !self.input_hash.is_empty() {
                    self.start_fetch(self.input_hash.clone());
                }
            }
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.status_message = None;
            }
            KeyCode::Char(c) => {
                if c.is_ascii_hexdigit() && self.input_hash.len() < 64 {
                    self.input_hash.insert(self.cursor_position, c);
                    self.cursor_position += 1;
                }
            }
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                    self.input_hash.remove(self.cursor_position);
                }
            }
            KeyCode::Delete => {
                if self.cursor_position < self.input_hash.len() {
                    self.input_hash.remove(self.cursor_position);
                }
            }
            KeyCode::Left => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
            }
            KeyCode::Right => {
                if self.cursor_position < self.input_hash.len() {
                    self.cursor_position += 1;
                }
            }
            KeyCode::Home => {
                self.cursor_position = 0;
            }
            KeyCode::End => {
                self.cursor_position = self.input_hash.len();
            }
            KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                match crate::clipboard::get_clipboard_text() {
                    Ok(text) => {
                        // Filter to hex characters only
                        let hex_only: String = text
                            .chars()
                            .filter(|c| c.is_ascii_hexdigit())
                            .take(64 - self.input_hash.len())
                            .collect();
                        
                        if !hex_only.is_empty() {
                            self.input_hash.push_str(&hex_only);
                            self.cursor_position = self.input_hash.len();
                            self.status_message = Some(format!("✓ Pasted {} characters", hex_only.len()));
                        }
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Failed to paste: {}", e));
                    }
                }
            }
            _ => {}
        }
    }

    fn copy_selected_to_clipboard(&self) {
        let result = if let Some(node) = self.tree_state.visible_nodes.get(self.tree_state.selected_index) {
            match node {
                TreeNode::Input { index, .. } => {
                    if let Some(tx) = self.get_current_tx() {
                        if let Some(input) = tx.inputs.get(*index) {
                            let content = format!(
                                "Input #{}\nTransaction: {}\nIndex: {}\nAddress: {}",
                                index, input.tx_hash, input.index, input.address
                            );
                            crate::clipboard::copy_to_clipboard(&content)
                        } else {
                            crate::clipboard::copy_to_clipboard(&node.display_text())
                        }
                    } else {
                        crate::clipboard::copy_to_clipboard(&node.display_text())
                    }
                }
                TreeNode::Output { index, .. } => {
                    if let Some(tx) = self.get_current_tx() {
                        if let Some(output) = tx.outputs.get(*index) {
                            let content = format!(
                                "Output #{}\nAddress: {}\nValue: {}",
                                index, 
                                output.address,
                                output.value.iter()
                                    .map(|a| format!("{} {}", a.amount, a.asset_name))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            );
                            crate::clipboard::copy_to_clipboard(&content)
                        } else {
                            crate::clipboard::copy_to_clipboard(&node.display_text())
                        }
                    } else {
                        crate::clipboard::copy_to_clipboard(&node.display_text())
                    }
                }
                TreeNode::InputDatum { input_index } => {
                    if let Some(tx) = self.get_current_tx() {
                        if let Some(input) = tx.inputs.get(*input_index) {
                            if let Some(datum) = &input.datum {
                                crate::clipboard::copy_plutus_data(&datum.decoded)
                            } else {
                                Ok(())
                            }
                        } else {
                            Ok(())
                        }
                    } else {
                        Ok(())
                    }
                }
                TreeNode::OutputDatum { output_index } => {
                    if let Some(tx) = self.get_current_tx() {
                        if let Some(output) = tx.outputs.get(*output_index) {
                            if let Some(datum) = &output.datum {
                                crate::clipboard::copy_plutus_data(&datum.decoded)
                            } else {
                                Ok(())
                            }
                        } else {
                            Ok(())
                        }
                    } else {
                        Ok(())
                    }
                }
                TreeNode::Redeemer { index, .. } => {
                    if let Some(tx) = self.get_current_tx() {
                        if let Some(redeemer) = tx.redeemers.get(*index) {
                            crate::clipboard::copy_plutus_data(&redeemer.data)
                        } else {
                            Ok(())
                        }
                    } else {
                        Ok(())
                    }
                }
                TreeNode::Metadata { .. } => {
                    if let Some(tx) = self.get_current_tx() {
                        if let Some(metadata) = &tx.metadata {
                            if let Ok(json) = serde_json::to_string_pretty(metadata) {
                                crate::clipboard::copy_to_clipboard(&json)
                            } else {
                                Ok(())
                            }
                        } else {
                            Ok(())
                        }
                    } else {
                        Ok(())
                    }
                }
                _ => crate::clipboard::copy_to_clipboard(&node.display_text()),
            }
        } else {
            Ok(())
        };

        if let Err(e) = result {
            self.status_message = Some(format!("Failed to copy: {}", e));
        } else {
            self.status_message = Some("✓ Copied to clipboard".to_string());
        }
    }

    fn copy_policy_id_to_clipboard(&self) {
        let result = if let FetchState::Done(tx) = &self.fetch_state {
            if let Some(node) = self.tree_state.visible_nodes.get(self.tree_state.selected_index) {
                match node {
                    TreeNode::Output { index, .. } => {
                        if let Some(output) = tx.outputs.get(*index) {
                            if let Some(asset) = output.value.iter().find(|a| a.policy_id != "ada") {
                                crate::clipboard::copy_policy_id(&asset.policy_id)
                            } else {
                                // Copy ADA as fallback
                                crate::clipboard::copy_to_clipboard("ada")
                            }
                        } else {
                            Ok(())
                        }
                    }
                    TreeNode::Input { index, .. } => {
                        if let Some(input) = tx.inputs.get(*index) {
                            if let Some(asset) = input.value.iter().find(|a| a.policy_id != "ada") {
                                crate::clipboard::copy_policy_id(&asset.policy_id)
                            } else {
                                crate::clipboard::copy_to_clipboard("ada")
                            }
                        } else {
                            Ok(())
                        }
                    }
                    _ => Ok(()),
                }
            } else {
                Ok(())
            }
        } else {
            Ok(())
        };

        if let Err(e) = result {
            self.status_message = Some(format!("Failed to copy policy ID: {}", e));
        } else {
            self.status_message = Some("✓ Policy ID copied to clipboard".to_string());
        }
    }

    fn copy_raw_hex_to_clipboard(&self) {
        let result = if let FetchState::Done(tx) = &self.fetch_state {
            if let Some(node) = self.tree_state.visible_nodes.get(self.tree_state.selected_index) {
                match node {
                    TreeNode::OutputDatum { output_index } => {
                        if let Some(output) = tx.outputs.get(*output_index) {
                            if let Some(datum) = &output.datum {
                                crate::clipboard::copy_raw_cbor(&datum.raw_cbor)
                            } else {
                                Ok(())
                            }
                        } else {
                            Ok(())
                        }
                    }
                    TreeNode::InputDatum { input_index } => {
                        if let Some(input) = tx.inputs.get(*input_index) {
                            if let Some(datum) = &input.datum {
                                crate::clipboard::copy_raw_cbor(&datum.raw_cbor)
                            } else {
                                Ok(())
                            }
                        } else {
                            Ok(())
                        }
                    }
                    TreeNode::Redeemer { index, .. } => {
                        if let Some(redeemer) = tx.redeemers.get(*index) {
                            // For redeemers, copy the decoded data as pretty-printed
                            crate::clipboard::copy_plutus_data(&redeemer.data)
                        } else {
                            Ok(())
                        }
                    }
                    _ => Ok(()),
                }
            } else {
                Ok(())
            }
        } else {
            Ok(())
        };

        if let Err(e) = result {
            self.status_message = Some(format!("Failed to copy raw data: {}", e));
        } else {
            self.status_message = Some("✓ Raw data copied to clipboard".to_string());
        }
    }

    pub fn get_current_tx(&self) -> Option<&TxView> {
        match &self.fetch_state {
            FetchState::Done(tx) => Some(tx),
            _ => None,
        }
    }

    pub fn get_spinner_char(&self) -> char {
        const SPINNER_CHARS: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        SPINNER_CHARS[self.spinner_frame]
    }
}

async fn fetch_transaction(fetcher: Box<dyn TxFetcher>, hash: &str) -> Result<TxView> {
    let raw_tx = fetcher.fetch(hash).await
        .context("Failed to fetch transaction")?;
    
    let mut parser = TxParser::new();
    let tx_view = parser.parse_transaction(&raw_tx.cbor)
        .context("Failed to parse transaction")?;
    
    Ok(tx_view)
}

impl Default for App {
    fn default() -> Self {
        Self::new(
            Network::Mainnet,
            FetcherConfig::Koios { network: Network::Mainnet }
        )
    }
}