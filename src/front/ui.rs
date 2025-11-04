// front/ui.rs

use ratatui::{
    Frame, Terminal, backend::CrosstermBackend, layout::{Alignment, Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style, Stylize}, symbols::DOT, text::Line, widgets::{self, Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap, block::Title}
};

// Import App and related types from the app module
use super::app::*;

// Note: No need for crossterm or std::io here, as they're not used in UI rendering.

pub fn ui(f: &mut Frame, app: &mut App) {
    let size = f.size();
    match app.state {
        AppState::SelectRealm => draw_select_realm(f, app, size, true),
        AppState::PasswordPrompt => {
            draw_select_realm(f, app, size, false);
            draw_password_prompt(f, app, size);
        },
        AppState::NewRealmForm => {
            draw_select_realm(f, app, size, false);
            draw_new_realm_form(f, app, size);
        }
        AppState::Dashboard => draw_dashboard(f, app, size),
    }
}

fn draw_select_realm(f: &mut Frame, app: &mut App, size: Rect, tips : bool) {
    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("Select Realm");
    if tips{
        block = block.title(Title::from(Line::from(vec!["Esc".red(), DOT.into(), "↑↓".red(), DOT.into(), "Select".into(), "↵".red(), DOT.into(), "N".red(), "ew".into()]))
            .position(widgets::block::Position::Bottom).alignment(Alignment::Right));
    }
    let inner_area = block.inner(size);
    f.render_widget(block, size);

    let items: Vec<ListItem> = app
        .realm_list
        .iter()
        .enumerate()
        .map(|(i, ca)| {
            let style = if i == app.realm_selected {
                Style::default().fg(Color::Black).bg(Color::White)
            } else {
                Style::default()
            };
            ListItem::new(ca.as_str()).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::NONE))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    let mut state = ListState::default();
    state.select(Some(app.realm_selected));

    f.render_stateful_widget(list, inner_area, &mut state);
}

fn draw_password_prompt(f: &mut Frame, app: &mut App, size: Rect) {
    let area = popup_rect(25, 5, 40, 20, size);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(app.realm_list[app.realm_selected].as_str());
    f.render_widget(Clear, area); // clear the background
    f.render_widget(&block, area);

    let inner_area = block.inner(area);
    let text = vec![
        Line::from(""),
        Line::from(format!("Password:{}", "*".repeat(app.password_text.len()))),
        Line::from(""),
    ];
    let paragraph = Paragraph::new(text).wrap(Wrap { trim: true });
    f.render_widget(paragraph, inner_area);
}

fn draw_new_realm_form(f: &mut Frame, app: &mut App, size: Rect) {
    let area = popup_rect(50, 20, 60, 70, size); // Dynamic size: 60% width/height, min 50x20
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("New Realm");
    f.render_widget(Clear, area);
    f.render_widget(&block, area);

    let inner_area = block.inner(area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // Realm block
            Constraint::Length(12), // CA block
            Constraint::Length(3), // Buttons
        ])
        .split(inner_area);

    // Realm block
    let realm_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("Realm");
    let realm_inner = realm_block.inner(chunks[0]);
    f.render_widget(realm_block, chunks[0]);

    let realm_lines = vec![
        Line::from(""),
        Line::from(format!("Name: {}", if app.nrf_selected_field == 0 { app.nrf_name.clone() + "_" } else { app.nrf_name.clone() })).style(if app.nrf_selected_field == 0 { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() }),
        Line::from(""),
        Line::from(format!("Password: {}", if app.nrf_selected_field == 1 { "*".repeat(app.nrf_form_password.len()) + "_" } else { "*".repeat(app.nrf_form_password.len()) })).style(if app.nrf_selected_field == 1 { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() }),
        Line::from(""),
    ];
    let realm_paragraph = Paragraph::new(realm_lines).wrap(Wrap { trim: true });
    f.render_widget(realm_paragraph, realm_inner);

    // CA block
    let ca_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("CA");
    let ca_inner = ca_block.inner(chunks[1]);
    f.render_widget(ca_block, chunks[1]);

    let key_sizes = [1024, 2048, 4096];
    let ca_lines = vec![
        Line::from(""),
        Line::from(format!("Common Name: {}", if app.nrf_selected_field == 2 { app.nrf_ca_common_name.clone() + "_" } else { app.nrf_ca_common_name.clone() })).style(if app.nrf_selected_field == 2 { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() }),
        Line::from(""),
        Line::from(format!("Organization: {}", if app.nrf_selected_field == 3 { app.nrf_ca_organization.clone() + "_" } else { app.nrf_ca_organization.clone() })).style(if app.nrf_selected_field == 3 { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() }),
        Line::from(""),
        Line::from(format!("Country: {}", if app.nrf_selected_field == 4 { app.nrf_ca_country.clone() + "_" } else { app.nrf_ca_country.clone() })).style(if app.nrf_selected_field == 4 { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() }),
        Line::from(""),
        Line::from(format!("Key Size: {} {} {}",
            if app.nrf_selected_field == 5 && app.nrf_ca_key_size_index > 0 { "←" } else { " " },
            key_sizes[app.nrf_ca_key_size_index],
            if app.nrf_selected_field == 5 && app.nrf_ca_key_size_index < key_sizes.len().saturating_sub(1) { "→" } else { " " })).style(if app.nrf_selected_field == 5 { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() }),
        Line::from(""),
    ];
    let ca_paragraph = Paragraph::new(ca_lines).wrap(Wrap { trim: true });
    f.render_widget(ca_paragraph, ca_inner);

    // Buttons
    let button_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[2]);

    let create_text = if app.nrf_selected_field == 6 { "[Create]" } else { " Create " };
    let create_paragraph = Paragraph::new(create_text)
        .alignment(Alignment::Center)
        .style(if app.nrf_selected_field == 6 { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() });
    f.render_widget(create_paragraph, button_chunks[1]);

    let cancel_text = if app.nrf_selected_field == 7 { "[Cancel]" } else { " Cancel " };
    let cancel_paragraph     = Paragraph::new(cancel_text)
        .alignment(Alignment::Center)
        .style(if app.nrf_selected_field == 7 { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() });
    f.render_widget(cancel_paragraph, button_chunks[0]);

}

fn draw_dashboard(f: &mut Frame, app: &mut App, size: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(Title::from(app.realm_list[app.realm_selected].as_str()).alignment(Alignment::Center));
    let inner_area = block.inner(size);
    f.render_widget(block, size);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(inner_area);

    // Left panel: split into static menu and cert list
    let left_inner = chunks[0];

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(app.dashboard_static_menu.len() as u16 + 2), // +2 for borders
            Constraint::Min(1),
        ])
        .split(left_inner);

    // Static menu block
    let static_block = Block::default().borders(Borders::ALL).border_type(BorderType::Rounded);
    let static_inner = static_block.inner(left_chunks[0]);
    f.render_widget(static_block, left_chunks[0]);

    let static_items: Vec<ListItem> = app
        .dashboard_static_menu
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == app.dashboard_selected_static && matches!(app.dashboard_focus, DashboardFocus::StaticOption) {
                Style::default().fg(Color::Black).bg(Color::White)
            } else {
                Style::default()
            };
            ListItem::new(item.as_str()).style(style)
        })
        .collect();

    let static_list = List::new(static_items)
        .block(Block::default().borders(Borders::NONE))
        .highlight_style(if matches!(app.dashboard_focus, DashboardFocus::StaticOption) {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        })
        .highlight_symbol(if matches!(app.dashboard_focus, DashboardFocus::StaticOption) { "> " } else { "" });

    let mut static_state = ListState::default();
    if matches!(app.dashboard_focus, DashboardFocus::StaticOption) {
        static_state.select(Some(app.dashboard_selected_static));
    }

    f.render_stateful_widget(static_list, static_inner, &mut static_state);

    // Cert list block
    let cert_block = Block::default().borders(Borders::ALL).border_type(BorderType::Rounded);
    let cert_inner = cert_block.inner(left_chunks[1]);
    f.render_widget(cert_block, left_chunks[1]);

    let cert_items: Vec<ListItem> = app
        .dashboard_cert_list
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == app.dashboard_selected_cert && matches!(app.dashboard_focus, DashboardFocus::CertList) {
                Style::default().fg(Color::Black).bg(Color::White)
            } else {
                Style::default()
            };
            ListItem::new(item.as_str()).style(style)
        })
        .collect();

    let cert_list = List::new(cert_items)
        .block(Block::default().borders(Borders::NONE))
        .highlight_style(if matches!(app.dashboard_focus, DashboardFocus::CertList) {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        })
        .highlight_symbol(if matches!(app.dashboard_focus, DashboardFocus::CertList) { "> " } else { "" });

    let mut cert_state = ListState::default();
    if matches!(app.dashboard_focus, DashboardFocus::CertList) {
        cert_state.select(Some(app.dashboard_selected_cert));
    }

    f.render_stateful_widget(cert_list, cert_inner, &mut cert_state);

    // Right panel: content
    let right_block = Block::default().borders(Borders::ALL).border_type(BorderType::Rounded);
    let right_inner = right_block.inner(chunks[1]);
    f.render_widget(right_block, chunks[1]);

    let paragraph = Paragraph::new("Content")
        .block(Block::default().borders(Borders::NONE))
        .alignment(Alignment::Center);
    f.render_widget(paragraph, right_inner);
}

/// helper function to create a popup rect with minimum size and percentage
pub fn popup_rect(min_width: u16, min_height: u16, percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let desired_width = ((r.width * percent_x) / 100).max(min_width).min(r.width);
    let desired_height = ((r.height * percent_y) / 100).max(min_height).min(r.height);
    let x = r.x + (r.width.saturating_sub(desired_width)) / 2;
    let y = r.y + (r.height.saturating_sub(desired_height)) / 2;
    Rect::new(x, y, desired_width, desired_height)
}