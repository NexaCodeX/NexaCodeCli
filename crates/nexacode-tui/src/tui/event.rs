//! 事件处理
use crate::app::App;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::time::Duration;

pub async fn handle_event(app: &mut App) -> anyhow::Result<bool> {
    if !event::poll(Duration::from_millis(100))? {
        return Ok(false);
    }

    match event::read()? {
        Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
            handle_key_event(app, key_event)
        }
        _ => Ok(false),
    }
}

fn handle_key_event(app: &mut App, key_event: KeyEvent) -> anyhow::Result<bool> {
    match (key_event.modifiers, key_event.code) {
        // Ctrl+C 退出
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
            app.quit();
            Ok(true)
        }
        // Enter 提交输入
        (KeyModifiers::NONE, KeyCode::Enter) => {
            if let Some(_content) = app.submit_input() {
                // TODO: 这里后续接入 LLM 进行处理
                // 模拟 AI 回复
                app.add_assistant_message("I received your message. This is a placeholder response.\nIn the future, this will be connected to an LLM.");
            }
            Ok(true)
        }
        // t / Ctrl+T 切换主题
        (KeyModifiers::NONE, KeyCode::Char('t')) | (KeyModifiers::CONTROL, KeyCode::Char('t')) => {
            app.toggle_theme();
            Ok(true)
        }
        // 字符输入
        (KeyModifiers::NONE, KeyCode::Char(c)) => {
            app.insert_char(c);
            Ok(true)
        }
        // 退格删除
        (KeyModifiers::NONE, KeyCode::Backspace) => {
            app.delete_char();
            Ok(true)
        }
        // 左移光标
        (KeyModifiers::NONE, KeyCode::Left) => {
            app.move_cursor_left();
            Ok(true)
        }
        // 右移光标
        (KeyModifiers::NONE, KeyCode::Right) => {
            app.move_cursor_right();
            Ok(true)
        }
        // Home 光标到开头
        (KeyModifiers::NONE, KeyCode::Home) => {
            app.move_cursor_start();
            Ok(true)
        }
        // End 光标到结尾
        (KeyModifiers::NONE, KeyCode::End) => {
            app.move_cursor_end();
            Ok(true)
        }
        // 上滚消息
        (KeyModifiers::NONE, KeyCode::Up) => {
            app.scroll_up();
            Ok(true)
        }
        // 下滚消息
        (KeyModifiers::NONE, KeyCode::Down) => {
            app.scroll_down();
            Ok(true)
        }
        _ => Ok(false),
    }
}
