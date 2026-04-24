//! Chat view - Claude Code CLI style interface

use crate::tui::Theme;
use crate::{MessageRole, Mode, State};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

pub fn render(f: &mut Frame, area: Rect, state: &State) {
    // Check if we're in model selection mode
    if !state.model_selections.is_empty() {
        render_model_selection(f, area, state);
        return;
    }

    // Calculate space for command suggestions
    let suggestion_height = if !state.command_suggestions.is_empty() {
        // Show up to 10 suggestions, each taking 1 line
        (state.command_suggestions.len().min(10) + 1) as u16
    } else {
        0
    };

    // Vertical split: messages on top, suggestions (if any), input at bottom
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(if suggestion_height > 0 {
            vec![
                Constraint::Min(5),                        // Message area
                Constraint::Length(suggestion_height),     // Suggestions
                Constraint::Length(3),                     // Input area
            ]
        } else {
            vec![
                Constraint::Min(5),      // Message area - minimum 5 lines
                Constraint::Length(3),   // Input area - fixed 3 lines
            ]
        })
        .split(area);

    let theme = Theme::from(state.theme);
    
    if suggestion_height > 0 {
        render_messages(f, chunks[0], state, &theme);
        render_suggestions(f, chunks[1], state, &theme);
        render_input(f, chunks[2], state, &theme);
    } else {
        render_messages(f, chunks[0], state, &theme);
        render_input(f, chunks[1], state, &theme);
    }

    // Render help overlay if showing
    if state.show_help {
        render_help(f, f.size(), state, &theme);
    }
}

fn render_messages(f: &mut Frame, area: Rect, state: &State, theme: &Theme) {
    let mut lines = Vec::new();

    // Top padding
    lines.push(Line::from(""));
    lines.push(Line::from(""));

    // Logo title
    lines.push(Line::from(vec![
        Span::styled("  NexaCode", Style::default().fg(Color::Black).bold()),
    ]));

    // Mode indicator
    let mode_style = match state.mode {
        Mode::Normal => Style::default().fg(theme.info()),
        Mode::Input => Style::default().fg(theme.primary()),
        Mode::Command => Style::default().fg(theme.warning()),
        Mode::Search => Style::default().fg(theme.purple()),
    };
    lines.push(Line::from(vec![
        Span::styled("  -- ", Style::default().fg(theme.secondary())),
        Span::styled(format!("{}", state.mode), mode_style),
        Span::styled(" --", Style::default().fg(theme.secondary())),
    ]));

    lines.push(Line::from(""));

    if state.messages.is_empty() {
        // Welcome prompt
        lines.push(Line::from(vec![
            Span::styled("  What do you want to build?", Style::default().fg(theme.secondary())),
        ]));
    } else {
        // Render message history with search highlighting
        for (msg_idx, msg) in state.messages.iter().enumerate() {
            match msg.role {
                MessageRole::User => {
                    lines.push(Line::from(vec![
                        Span::styled("  ◇ ", Style::default().fg(theme.secondary())),
                        Span::styled("You", Style::default().fg(theme.secondary()).bold()),
                    ]));
                    render_message_content(&mut lines, msg, msg_idx, state, theme);
                }
                MessageRole::Assistant => {
                    lines.push(Line::from(vec![
                        Span::styled("  ◆ ", Style::default().fg(theme.info())),
                        Span::styled("Assistant", Style::default().fg(theme.info()).bold()),
                    ]));
                    render_message_content(&mut lines, msg, msg_idx, state, theme);
                }
                MessageRole::System => {
                    lines.push(Line::from(vec![
                        Span::styled("  ⚙ System", Style::default().fg(theme.warning()).bold()),
                    ]));
                    render_message_content(&mut lines, msg, msg_idx, state, theme);
                }
                MessageRole::Tool => {
                    lines.push(Line::from(vec![
                        Span::styled("  🔧 Tool", Style::default().fg(theme.purple()).bold()),
                    ]));
                    render_message_content(&mut lines, msg, msg_idx, state, theme);
                }
            }
            lines.push(Line::from("")); // Empty line between messages
        }
    }

    // Status message
    if let Some(status) = &state.status_message {
        lines.push(Line::from(""));
        let status_style = if state.status_is_error {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(theme.info())
        };
        lines.push(Line::from(vec![
            Span::styled("  ⚡ ", status_style),
            Span::styled(status, status_style),
        ]));
    }

    // Search results indicator
    if let Some(query) = &state.search_query {
        let match_info: String = if state.search_results.is_empty() {
            "No matches".to_string()
        } else {
            let current = state.current_match_index.map(|i| i + 1).unwrap_or(0);
            format!("{}/{}", current, state.search_results.len())
        };
        lines.push(Line::from(vec![
            Span::styled("  🔍 \"", Style::default().fg(theme.purple())),
            Span::styled(query.clone(), Style::default().fg(theme.purple()).bold()),
            Span::styled("\" ", Style::default().fg(theme.purple())),
            Span::styled(match_info, Style::default().fg(theme.info())),
        ]));
    }

    let paragraph = Paragraph::new(lines)
        .style(theme.base_style())
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

/// Render message content with search highlighting
fn render_message_content<'a>(
    lines: &mut Vec<Line<'a>>,
    msg: &'a nexacode_core::Message,
    msg_idx: usize,
    state: &State,
    theme: &Theme,
) {
    // Find all search matches in this message
    let matches_in_msg: Vec<_> = state
        .search_results
        .iter()
        .filter(|m| m.message_index == msg_idx)
        .collect();

    for line in msg.content.lines() {
        if matches_in_msg.is_empty() || state.search_query.is_none() {
            // No highlighting needed
            lines.push(Line::from(format!("    {}", line)));
        } else {
            // Apply search highlighting
            let highlighted = highlight_search_matches(line, &matches_in_msg, theme);
            lines.push(Line::from(highlighted));
        }
    }
}

/// Highlight search matches in a line
fn highlight_search_matches<'a>(line: &'a str, _matches: &[&crate::SearchMatch], theme: &Theme) -> Vec<Span<'a>> {
    let mut spans = Vec::new();
    let _last_end = 0;

    let prefix = "    ";
    spans.push(Span::raw(prefix));

    // If there are matches, highlight them
    // This is a simplified version - proper implementation would track line offsets
    if !_matches.is_empty() {
        // Just highlight the entire line with a subtle background for now
        spans.push(Span::styled(
            line,
            Style::default()
                .fg(theme.foreground())
                .bg(Color::Yellow)
                .add_modifier(Modifier::DIM),
        ));
    } else {
        spans.push(Span::raw(line));
    }

    spans
}

fn render_input(f: &mut Frame, area: Rect, state: &State, theme: &Theme) {
    // Determine prompt based on mode
    let (prompt, prompt_style) = match state.mode {
        Mode::Normal => (" ", Style::default().fg(theme.secondary())),
        Mode::Input => (">", Style::default().fg(Color::Black).bold()),
        Mode::Command => (":", Style::default().fg(theme.warning()).bold()),
        Mode::Search => ("/", Style::default().fg(theme.purple()).bold()),
    };

    // Input box with border
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border()))
        .style(theme.base_style());

    // Build input content with cursor
    let input_text = if state.input.is_empty() {
        format!("█")
    } else {
        // Insert cursor at correct position
        let byte_pos = state.cursor_pos;
        let char_pos = state.input[..byte_pos.min(state.input.len())]
            .chars()
            .count();
        let chars: Vec<char> = state.input.chars().collect();
        let mut chars_with_cursor = chars.clone();
        chars_with_cursor.insert(char_pos, '█');
        chars_with_cursor.into_iter().collect()
    };

    let input_content = vec![
        Line::from(vec![
            Span::styled(format!("{} ", prompt), prompt_style),
            Span::styled(input_text, Style::default().fg(theme.foreground())),
        ]),
    ];

    let paragraph = Paragraph::new(input_content)
        .block(input_block);

    f.render_widget(paragraph, area);
}

/// Render command suggestions
fn render_suggestions(f: &mut Frame, area: Rect, state: &State, theme: &Theme) {
    if state.command_suggestions.is_empty() {
        return;
    }

    let mut lines = Vec::new();
    
    // Header
    lines.push(Line::from(vec![
        Span::styled("  Commands (↑↓ to select, Tab to choose):", Style::default().fg(theme.secondary()).bold()),
    ]));

    // Suggestions
    for (idx, suggestion) in state.command_suggestions.iter().take(10).enumerate() {
        let cmd_name = suggestion.trim_start_matches('/');
        let description = get_command_description(cmd_name);
        let is_selected = state.command_suggestion_index == Some(idx);
        
        let (cmd_style, desc_style) = if is_selected {
            (
                Style::default().fg(Color::Black).bg(theme.primary()).bold(),
                Style::default().fg(Color::Black).bg(theme.primary()),
            )
        } else {
            (
                Style::default().fg(theme.primary()).bold(),
                Style::default().fg(theme.foreground()),
            )
        };
        
        let prefix = if is_selected { "  ▸ " } else { "    " };
        
        lines.push(Line::from(vec![
            Span::styled(prefix, Style::default()),
            Span::styled(suggestion, cmd_style),
            Span::styled(" - ", Style::default().fg(theme.secondary())),
            Span::styled(description, desc_style),
        ]));
    }

    let paragraph = Paragraph::new(lines)
        .style(theme.base_style())
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

/// Get a short description for a command
fn get_command_description(cmd: &str) -> &'static str {
    match cmd {
        "model" => "Switch or view current model",
        "models" => "List all available models",
        "config" => "Show current configuration",
        "new" => "Start a new conversation",
        "sessions" => "List all saved sessions",
        "load" => "Load a previous session",
        "save" => "Save current session",
        "export" => "Export session to file",
        "undo" => "Undo last message pair",
        "redo" => "Redo undone message",
        "rollback" => "Rollback to message n",
        "clear" => "Clear current conversation",
        "help" => "Show help information",
        "version" => "Show version info",
        "quit" | "exit" => "Exit NexaCode",
        "theme" => "Switch theme",
        _ => "",
    }
}

fn render_help(f: &mut Frame, area: Rect, _state: &State, theme: &Theme) {
    // Create a centered help panel
    let help_area = centered_rect(60, 70, area);

    let help_block = Block::default()
        .title(" Help (press h or ? to close) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary()))
        .style(theme.base_style());

    let help_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  MODES", Style::default().fg(theme.info()).bold()),
        ]),
        Line::from("    i, a, Space, Enter  →  Input mode"),
        Line::from("    :                  →  Command mode"),
        Line::from("    /                  →  Search mode"),
        Line::from("    Esc                →  Normal mode"),
        Line::from(""),
        Line::from(vec![
            Span::styled("  NAVIGATION", Style::default().fg(theme.info()).bold()),
        ]),
        Line::from("    j/↓    Scroll down"),
        Line::from("    k/↑    Scroll up"),
        Line::from("    g      Scroll to top"),
        Line::from("    G      Scroll to bottom"),
        Line::from(""),
        Line::from(vec![
            Span::styled("  SEARCH", Style::default().fg(theme.info()).bold()),
        ]),
        Line::from("    /query  Search messages"),
        Line::from("    n       Next match"),
        Line::from("    N       Previous match"),
        Line::from(""),
        Line::from(vec![
            Span::styled("  COMMANDS", Style::default().fg(theme.info()).bold()),
        ]),
        Line::from("    :q, :quit     Quit"),
        Line::from("    :h, :help     Show this help"),
        Line::from("    :clear        Clear messages"),
        Line::from("    :theme        Toggle theme"),
        Line::from("    :new [name]   New session"),
        Line::from("    :save         Save session"),
        Line::from("    :sessions     List sessions"),
        Line::from(""),
        Line::from(vec![
            Span::styled("  EDITING (Input mode)", Style::default().fg(theme.info()).bold()),
        ]),
        Line::from("    Ctrl+A    Move to start"),
        Line::from("    Ctrl+E    Move to end"),
        Line::from("    Ctrl+W    Delete word backward"),
        Line::from("    Ctrl+U    Clear input"),
        Line::from("    ↑/↓       Input history"),
        Line::from(""),
        Line::from(vec![
            Span::styled("  OTHER", Style::default().fg(theme.info()).bold()),
        ]),
        Line::from("    t, Ctrl+T  Toggle theme"),
        Line::from("    u          Undo"),
        Line::from("    Ctrl+R     Redo"),
        Line::from(""),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(help_block)
        .style(theme.base_style());

    f.render_widget(paragraph, help_area);
}

/// Helper function to create a centered rect
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Render model selection overlay
fn render_model_selection(f: &mut Frame, area: Rect, state: &State) {
    let theme = Theme::from(state.theme);
    
    // Load current config to highlight current model
    let config = nexacode_core::Config::load();
    let current_model = config.current_model();
    let current_provider = config.current_provider();
    
    // Create a centered selection panel
    let selection_area = centered_rect(60, 60, area);

    let selection_block = Block::default()
        .title(" Select Model (↑↓ to navigate, Enter to select, Esc to cancel) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary()))
        .style(theme.base_style());

    let mut lines = Vec::new();
    lines.push(Line::from(""));

    for (idx, model) in state.model_selections.iter().enumerate() {
        let is_selected = state.model_selection_index == Some(idx);
        let is_current = model.id == current_model && model.provider == current_provider;
        
        let style = if is_selected {
            Style::default().fg(Color::Black).bg(theme.primary()).bold()
        } else if is_current {
            Style::default().fg(theme.primary()).bold()
        } else {
            Style::default().fg(theme.foreground())
        };
        
        let prefix = if is_selected {
            "  ▸ "
        } else if is_current {
            "  ✓ "
        } else {
            "    "
        };
        
        let suffix = if is_current { " [current]" } else { "" };
        
        lines.push(Line::from(vec![
            Span::styled(prefix, Style::default()),
            Span::styled(format!("{}{}", model, suffix), style),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  Press Enter to switch to selected model", 
                    Style::default().fg(theme.secondary())),
    ]));

    let paragraph = Paragraph::new(lines)
        .block(selection_block)
        .style(theme.base_style());

    f.render_widget(paragraph, selection_area);
}
