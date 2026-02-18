//! BullMQ TUI - Web Application (WASM)
//!
//! This crate provides a WASM-based web frontend using Ratzilla
//! for rendering the TUI in the browser.

mod ws_client;

use bullmq_tui_core::{render, update, Effect, Message, Model};
use ratzilla::event::{KeyCode, KeyEvent};
use ratzilla::ratatui::Terminal;
use ratzilla::{DomBackend, WebRenderer};
use std::cell::RefCell;
use std::io;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

/// Initialize and run the TUI in the browser
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    // Set up panic hook for better error messages
    console_error_panic_hook::set_once();

    // Get proxy URL from environment or use default
    let proxy_url = web_sys::window()
        .and_then(|w| w.location().origin().ok())
        .map(|origin| format!("{}/ws", origin.replace("http", "ws")))
        .unwrap_or_else(|| "ws://localhost:3001/ws".to_string());

    // Create the model
    let model = Rc::new(RefCell::new(Model::new(proxy_url)));

    // Create the DOM backend and terminal
    let backend = DomBackend::new().map_err(|e| JsValue::from_str(&e.to_string()))?;
    let terminal = Terminal::new(backend).map_err(|e: io::Error| JsValue::from_str(&e.to_string()))?;

    // Set up keyboard event handling
    let model_for_input = model.clone();
    terminal.on_key_event(move |event| {
        let msg = convert_key_event(&event, &model_for_input.borrow());
        if let Some(msg) = msg {
            let mut model = model_for_input.borrow_mut();
            let result = update(&mut model, msg);

            // Execute effects (simplified for WASM)
            for effect in result.effects {
                execute_effect_wasm(&effect);
            }
        }
    });

    // Render loop
    let model_for_render = model.clone();
    terminal.draw_web(move |frame| {
        render(&model_for_render.borrow(), frame);
    });

    Ok(())
}

/// Convert a Ratzilla key event to a TUI message
fn convert_key_event(event: &KeyEvent, model: &Model) -> Option<Message> {
    use bullmq_tui_core::View;

    // Global keybindings
    if event.ctrl && event.code == KeyCode::Char('c') {
        return Some(Message::Quit);
    }

    // View-specific keybindings
    match model.view {
        View::QueueList => match event.code {
            KeyCode::Char('q') | KeyCode::Esc => Some(Message::Quit),
            KeyCode::Char('?') => Some(Message::ShowHelp),
            KeyCode::Char('j') | KeyCode::Down => Some(Message::SelectNextQueue),
            KeyCode::Char('k') | KeyCode::Up => Some(Message::SelectPrevQueue),
            KeyCode::Enter => Some(Message::OpenQueue),
            KeyCode::Char('r') => Some(Message::RefreshQueues),
            _ => None,
        },
        View::JobList => {
            // Handle Shift+Tab for previous tab
            if event.shift && event.code == KeyCode::Tab {
                return Some(Message::PrevTab);
            }
            match event.code {
                KeyCode::Char('q') | KeyCode::Esc => Some(Message::Back),
                KeyCode::Char('?') => Some(Message::ShowHelp),
                KeyCode::Char('j') | KeyCode::Down => Some(Message::SelectNextJob),
                KeyCode::Char('k') | KeyCode::Up => Some(Message::SelectPrevJob),
                KeyCode::Tab => Some(Message::NextTab),
                KeyCode::Enter => Some(Message::OpenJobDetail),
                KeyCode::Char('r') => Some(Message::RetryJob),
                KeyCode::Char('d') => Some(Message::RemoveJob),
                _ => None,
            }
        }
        View::JobDetail => match event.code {
            KeyCode::Char('q') | KeyCode::Esc => Some(Message::Back),
            KeyCode::Char('?') => Some(Message::ShowHelp),
            KeyCode::Char('j') | KeyCode::Down => Some(Message::ScrollDown),
            KeyCode::Char('k') | KeyCode::Up => Some(Message::ScrollUp),
            _ => None,
        },
        View::Help => match event.code {
            KeyCode::Char('q') | KeyCode::Esc => Some(Message::HideOverlay),
            _ => None,
        },
        View::Confirm => match event.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => Some(Message::Confirm),
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Some(Message::Cancel),
            _ => None,
        },
        _ => None,
    }
}

/// Execute an effect in WASM context (simplified)
fn execute_effect_wasm(effect: &Effect) {
    // In WASM, we need to use the WebSocket client to communicate with the proxy
    // This is a simplified placeholder - full implementation would spawn tasks
    match effect {
        Effect::Quit => {
            // In browser, we can't really quit - just show a message
            web_sys::console::log_1(&"Quit requested".into());
        }
        Effect::LoadQueues => {
            web_sys::console::log_1(&"Loading queues...".into());
            // Would send WebSocket request here
        }
        Effect::LoadJobs { queue, state, .. } => {
            web_sys::console::log_1(&format!("Loading jobs for {} in state {:?}", queue, state).into());
            // Would send WebSocket request here
        }
        _ => {}
    }
}
