pub mod tx_tree;
pub mod detail;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::time::Duration;

use crate::app::{App, FetchState, InputMode, TreeNode};
use tx_tree::{TxTreeWidget, TreeState};
use detail::DetailWidget;

pub async fn run(mut app: App) -> Result<()> {
    let mut stdout = std::io::stdout();
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;

    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, &mut app).await;

    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    let tick_rate = Duration::from_millis(100);
    let mut tree_state = TreeState::new();
    
    loop {
        terminal.draw(|f| render(f, app, &mut tree_state))?;
        
        while let Some(event) = app.poll_event() {
            app.handle_event(event);
            if let FetchState::Done(_) = &app.fetch_state {
                // Sync tree state with app after fetch
                tree_state = TreeState {
                    selected_index: app.tree_state.selected_index,
                    expanded: app.tree_state.expanded.clone(),
                    visible_nodes: app.tree_state.visible_nodes.clone(),
                    scroll_offset: 0,
                };
            }
        }
        
        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match app.input_mode {
                        InputMode::Normal => {
                            if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                                break;
                            }
                            
                            // Handle tree navigation
                            match key.code {
                                KeyCode::Up => {
                                    tree_state.select_previous();
                                    app.tree_state.selected_index = tree_state.selected_index;
                                }
                                KeyCode::Down => {
                                    tree_state.select_next();
                                    app.tree_state.selected_index = tree_state.selected_index;
                                }
                                KeyCode::Right | KeyCode::Char(' ') => {
                                    if let Some(node) = tree_state.visible_nodes.get(tree_state.selected_index) {
                                        if matches!(node, 
                                            TreeNode::InputsHeader { .. } | 
                                            TreeNode::OutputsHeader { .. } | 
                                            TreeNode::RedeemersHeader { .. } | 
                                            TreeNode::Metadata { .. }
                                        ) {
                                            let all_nodes = app.tree_state.visible_nodes.clone();
                                            tree_state.toggle_expand(&all_nodes);
                                            app.tree_state.expanded = tree_state.expanded.clone();
                                            app.tree_state.visible_nodes = tree_state.visible_nodes.clone();
                                        }
                                    }
                                }
                                KeyCode::Left => {
                                    if let Some(node) = tree_state.visible_nodes.get(tree_state.selected_index) {
                                        if matches!(node,
                                            TreeNode::InputsHeader { .. } |
                                            TreeNode::OutputsHeader { .. } |
                                            TreeNode::RedeemersHeader { .. } |
                                            TreeNode::Metadata { .. }
                                        ) {
                                            let all_nodes = app.tree_state.visible_nodes.clone();
                                            tree_state.toggle_expand(&all_nodes);
                                            app.tree_state.expanded = tree_state.expanded.clone();
                                            app.tree_state.visible_nodes = tree_state.visible_nodes.clone();
                                        }
                                    }
                                }
                                _ => {}
                            }
                            
                            // Update scroll based on visible area
                            if let Some(height) = terminal.size().ok().map(|s| s.height as usize) {
                                tree_state.update_scroll(height.saturating_sub(6));
                            }
                        }
                        InputMode::Editing => {
                            if key.code == KeyCode::Esc {
                                app.input_mode = InputMode::Normal;
                                continue;
                            }
                        }
                    }
                    app.handle_key(key);
                }
            }
        }
        
        app.tick();
    }
    
    Ok(())
}

fn render(f: &mut Frame, app: &App, tree_state: &mut TreeState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.size());

    render_header(f, app, chunks[0]);
    render_main_content(f, app, tree_state, chunks[1]);
    render_status_bar(f, app, chunks[2]);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area);

    let title = Paragraph::new("🔍 cardano-tx-viz")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    f.render_widget(title, header_chunks[0]);

    let input_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(5), Constraint::Min(0)])
        .split(header_chunks[1]);

    let hash_label = Paragraph::new("hash:")
        .style(Style::default().fg(Color::Gray));
    f.render_widget(hash_label, input_chunks[0]);

    let hash_style = match app.input_mode {
        InputMode::Editing => Style::default().fg(Color::Yellow).bg(Color::DarkGray),
        InputMode::Normal => Style::default().fg(Color::White),
    };

    let display_hash = if app.input_hash.is_empty() && app.input_mode == InputMode::Normal {
        "<press / to enter hash>".to_string()
    } else {
        let mut hash = app.input_hash.clone();
        if app.input_mode == InputMode::Editing {
            if app.cursor_position < hash.len() {
                hash.insert(app.cursor_position, '|');
            } else {
                hash.push('|');
            }
        }
        hash
    };

    let hash_input = Paragraph::new(display_hash)
        .style(hash_style)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(hash_input, input_chunks[1]);
}

fn render_main_content(f: &mut Frame, app: &App, tree_state: &mut TreeState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    // Left panel - Tree
    let tree_block = Block::default()
        .title("TX TREE")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));

    match &app.fetch_state {
        FetchState::Loading => {
            let spinner = app.get_spinner_char();
            let loading_text = format!("{} Loading transaction...", spinner);
            let loading = Paragraph::new(loading_text)
                .block(tree_block)
                .style(Style::default().fg(Color::Yellow));
            f.render_widget(loading, chunks[0]);
        }
        FetchState::Error(err) => {
            let error_text = format!("❌ Error:\n{}", err);
            let error = Paragraph::new(error_text)
                .block(tree_block)
                .style(Style::default().fg(Color::Red));
            f.render_widget(error, chunks[0]);
        }
        FetchState::Idle => {
            let idle = Paragraph::new("Enter a transaction hash to begin\n\nPress / to search")
                .block(tree_block)
                .style(Style::default().fg(Color::Gray));
            f.render_widget(idle, chunks[0]);
        }
        FetchState::Done(_) => {
            let tree = TxTreeWidget::new().block(tree_block);
            f.render_stateful_widget(tree, chunks[0], tree_state);
        }
    }

    // Right panel - Detail
    let detail_block = Block::default()
        .title("DETAIL")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));

    if let FetchState::Done(tx) = &app.fetch_state {
        let node = tree_state.visible_nodes.get(tree_state.selected_index);
        let detail_area = detail_block.inner(chunks[1]);
        detail_block.render(chunks[1], f.buffer_mut());
        
        detail::render_detail_content(
            detail_area,
            f.buffer_mut(),
            node,
            Some(tx),
            app.detail_scroll,
        );
    } else {
        let empty = Paragraph::new("")
            .block(detail_block);
        f.render_widget(empty, chunks[1]);
    }
}

fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let status_text = if let Some(msg) = &app.status_message {
        Line::from(Span::styled(msg.clone(), Style::default().fg(Color::Yellow)))
    } else {
        match app.input_mode {
            InputMode::Editing => {
                Line::from(vec![
                    Span::styled("[Enter]", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::raw(" confirm  "),
                    Span::styled("[Esc]", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                    Span::raw(" cancel  "),
                    Span::styled("[Ctrl+V]", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::raw(" paste"),
                ])
            }
            InputMode::Normal => {
                Line::from(vec![
                    Span::styled("[/]", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::raw(" search  "),
                    Span::styled("[↑/↓]", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::raw(" navigate  "),
                    Span::styled("[→/←]", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::raw(" expand/collapse  "),
                    Span::styled("[c]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw(" copy  "),
                    Span::styled("[p]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw(" policy  "),
                    Span::styled("[r]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw(" raw  "),
                    Span::styled("[q]", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                    Span::raw(" quit"),
                ])
            }
        }
    };

    let status = Paragraph::new(status_text)
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(status, area);
}