//! 聊天视图 - 类似 Claude Code CLI 风格
use crate::app::{App, MessageRole};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    // 垂直分割：上方消息区，下方输入区
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),      // 消息区 - 最小5行
            Constraint::Length(3),   // 输入区 - 固定3行
        ])
        .split(area);

    render_messages(f, chunks[0], app);
    render_input(f, chunks[1], app);
}

fn render_messages(f: &mut Frame, area: Rect, app: &App) {
    let mut lines = Vec::new();

    // 顶部留白
    lines.push(Line::from(""));
    lines.push(Line::from(""));

    // Logo 标题
    lines.push(Line::from(vec![
        Span::styled(" NexaCode", Style::default().fg(Color::Black).bold()),
    ]));
    lines.push(Line::from(""));

    if app.messages.is_empty() {
        // 欢迎提示
        lines.push(Line::from(vec![
            Span::styled("  What do you want to build?", Style::default().fg(app.theme.secondary())),
        ]));
    } else {
        // 渲染消息历史
        for msg in &app.messages {
            match msg.role {
                MessageRole::User => {
                    lines.push(Line::from(vec![
                        Span::styled("  ◇ ", Style::default().fg(app.theme.secondary())),
                        Span::styled("You", Style::default().fg(app.theme.secondary()).bold()),
                    ]));
                    for line in msg.content.lines() {
                        lines.push(Line::from(format!("    {}", line)));
                    }
                }
                MessageRole::Assistant => {
                    lines.push(Line::from(vec![
                        Span::styled("  ◆ ", Style::default().fg(app.theme.info())),
                        Span::styled("Assistant", Style::default().fg(app.theme.info()).bold()),
                    ]));
                    for line in msg.content.lines() {
                        lines.push(Line::from(format!("    {}", line)));
                    }
                }
                MessageRole::System => {
                    for line in msg.content.lines() {
                        lines.push(Line::from(vec![
                            Span::styled(format!("  [system] {}", line), Style::default().fg(app.theme.secondary())),
                        ]));
                    }
                }
                MessageRole::Tool => {
                    for line in msg.content.lines() {
                        lines.push(Line::from(vec![
                            Span::styled(format!("  [tool] {}", line), Style::default().fg(app.theme.purple())),
                        ]));
                    }
                }
            }
            lines.push(Line::from("")); // 消息之间空行
        }
    }

    let paragraph = Paragraph::new(lines)
        .style(app.theme.base_style());

    f.render_widget(paragraph, area);
}

fn render_input(f: &mut Frame, area: Rect, app: &App) {
    // 输入框带边框
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.border()))
        .style(app.theme.base_style());

    // 构建输入内容，包含光标
    let input_text = if app.input.is_empty() {
        format!("█")
    } else {
        let mut chars: Vec<char> = app.input.chars().collect();
        chars.insert(app.cursor_pos, '█');
        chars.into_iter().collect()
    };

    let input_content = vec![
        Line::from(vec![
            Span::styled("> ", Style::default().fg(app.theme.primary()).bold()),
            Span::styled(input_text, Style::default().fg(app.theme.foreground())),
        ]),
    ];

    let paragraph = Paragraph::new(input_content)
        .block(input_block);

    f.render_widget(paragraph, area);
}
