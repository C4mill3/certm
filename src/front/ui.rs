use ratatui::{
    Frame, Terminal, backend::CrosstermBackend, layout::{Alignment, Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style, Stylize}, symbols::DOT, text::Line, widgets::{self, Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Widget, WidgetRef, Wrap, block::Title}
};

// Import App and related types from the app module
use super::app::*;
use super::mywidgets;

pub fn ui(f: &mut Frame, app: &mut App) {
    let size = f.area();
    match app.state {
        AppState::ErrorPrompt => draw_error_prompt(f, app, size),
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

fn draw_error_prompt(f: &mut Frame, app: &mut App, size: Rect) {
    let area = popup_rect(50, 20, 60, 70, size);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("Error")
        .title(Title::from(Line::from(vec!["↑↓".red(), DOT.into(), "Esc".red()]))
            .position(widgets::block::Position::Bottom)
            .alignment(Alignment::Right));

    f.render_widget(Clear, area); // clear the background
    f.render_widget(&block, area);

    let inner_area = block.inner(area);




    update_scroll(app, &inner_area, &app.last_error.clone());
    let content = mywidgets::ScrollableParagraph::new(app.last_error.clone(), app.scroll, app.max_scroll);
    content.render_ref(inner_area, f.buffer_mut());

}

fn draw_select_realm(f: &mut Frame, app: &mut App, size: Rect, tips : bool) {
    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("Select Realm");
    
    if tips{
        block = block.title(Title::from(Line::from(vec!["↑↓".red(), DOT.into(), "Select".into(), "↵".red(), DOT.into(), "N".red(), "ew".into(), DOT.into(), "Esc".red(),]))
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
        .title(app.realm_list[app.realm_selected].as_str())
        .title(Title::from(Line::from(vec!["Try".into(), "↵".red(), DOT.into(), "Esc".red(),]))
            .position(widgets::block::Position::Bottom).alignment(Alignment::Right));

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
            Constraint::Length(10), // CA block
            Constraint::Length(2), // Buttons
        ])
        .split(inner_area);

    // Realm block
    let realm_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("Realm");
    let realm_inner = realm_block.inner(chunks[0]);
    f.render_widget(realm_block, chunks[0]);

    // Calculate available width for Realm fields
    let available_width = realm_inner.width as usize;

    let name_full = if app.nrf_selected_field == 0 { app.nrf_name.clone() + "_" } else { app.nrf_name.clone() };
    let password_full = if app.nrf_selected_field == 1 { "*".repeat(app.nrf_password.len()) + "_" } else { "*".repeat(app.nrf_password.len()) };

    let realm_lines = vec![
        Line::from(""),
        Line::from(format!("Name: {}", truncate_with_ellipsis(&name_full, available_width, "Name: ".len()) )).style(if app.nrf_selected_field == 0 { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() }),
        Line::from(""),
        Line::from(format!("Password: {}", truncate_with_ellipsis(&password_full, available_width, "Password: ".len()) )).style(if app.nrf_selected_field == 1 { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() }),
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


    let common_name_full = if app.nrf_selected_field == 2 { app.nrf_ca_common_name.clone() + "_" } else { app.nrf_ca_common_name.clone() };
    let organization_full = if app.nrf_selected_field == 3 { app.nrf_ca_organization.clone() + "_" } else { app.nrf_ca_organization.clone() };
    let country_full = if app.nrf_selected_field == 4 { app.nrf_ca_country.clone() + "_" } else { app.nrf_ca_country.clone() };

    let key_sizes = [1024, 2048, 4096];
    let ca_lines = vec![
        Line::from(""),
        Line::from(format!("Common Name: {}", truncate_with_ellipsis(&common_name_full, available_width, "Common Name: ".len()) )).style(if app.nrf_selected_field == 2 { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() }),
        Line::from(""),
        Line::from(format!("Organization: {}", truncate_with_ellipsis(&organization_full, available_width, "Organization: ".len()) )).style(if app.nrf_selected_field == 3 { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() }),
        Line::from(""),
        Line::from(format!("Country: {}", truncate_with_ellipsis(&country_full, available_width, "Country: ".len()) )).style(if app.nrf_selected_field == 4 { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() }),
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
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[2]);

    let create_text = if app.nrf_selected_field == 6 { "[Create]" } else { " Create " };
    let create_paragraph = Paragraph::new(create_text)
        .alignment(Alignment::Center)
        .style(if app.nrf_selected_field == 6 { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() });
    f.render_widget(create_paragraph, button_chunks[0]);

    let cancel_text = if app.nrf_selected_field == 7 { "[Cancel]" } else { " Cancel " };
    let cancel_paragraph     = Paragraph::new(cancel_text)
        .alignment(Alignment::Center)
        .style(if app.nrf_selected_field == 7 { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() });
    f.render_widget(cancel_paragraph, button_chunks[1]);

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
            let style = if i == app.dashboard_selected_static && matches!(app.dashboard_select, DashboardSelect::StaticOption) {
                Style::default().fg(Color::Black).bg(Color::White)
            } else {
                Style::default()
            };
            ListItem::new(item.as_str()).style(style)
        })
        .collect();

    let static_list = List::new(static_items)
        .block(Block::default().borders(Borders::NONE))
        .highlight_style(if matches!(app.dashboard_select, DashboardSelect::StaticOption) {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        })
        .highlight_symbol(if matches!(app.dashboard_select, DashboardSelect::StaticOption) { "> " } else { "" });

    let mut static_state = ListState::default();
    if matches!(app.dashboard_select, DashboardSelect::StaticOption) {
        static_state.select(Some(app.dashboard_selected_static));
    }

    f.render_stateful_widget(static_list, static_inner, &mut static_state);

    // Cert list block
    let cert_block = Block::default().borders(Borders::ALL).border_type(BorderType::Rounded);
    let cert_inner = cert_block.inner(left_chunks[1]);
    f.render_widget(cert_block, left_chunks[1]);

    let cert_items: Vec<ListItem> = app
        .current_realm.as_ref().unwrap().certs
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == app.dashboard_selected_cert && matches!(app.dashboard_select, DashboardSelect::CertList) {
                Style::default().fg(Color::Black).bg(Color::White)
            } else {
                Style::default()
            };
            ListItem::new(item.get_subject_name().unwrap()).style(style)
        })
        .collect();

    let cert_list = List::new(cert_items)
        .block(Block::default().borders(Borders::NONE))
        .highlight_style(if matches!(app.dashboard_select, DashboardSelect::CertList) {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        })
        .highlight_symbol(if matches!(app.dashboard_select, DashboardSelect::CertList) { "> " } else { "" });

    let mut cert_state = ListState::default();
    if matches!(app.dashboard_select, DashboardSelect::CertList) {
        cert_state.select(Some(app.dashboard_selected_cert));
    }

    f.render_stateful_widget(cert_list, cert_inner, &mut cert_state);

    // Right panel: content
    let right_block = Block::default().borders(Borders::ALL).border_type(BorderType::Rounded);
    let right_inner = right_block.inner(chunks[1]);
    f.render_widget(right_block, chunks[1]);

    // call here generate_dashboard_content
    //f.render_widget(generate_dashboard_content(app), right_inner);
    generate_dashboard_content(app, &right_inner).render_ref(right_inner, f.buffer_mut());
}

/// helper function to create a popup rect with minimum size and percentage
pub fn popup_rect(min_width: u16, min_height: u16, percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let desired_width = ((r.width * percent_x) / 100).max(min_width).min(r.width);
    let desired_height = ((r.height * percent_y) / 100).max(min_height).min(r.height);
    let x = r.x + (r.width.saturating_sub(desired_width)) / 2;
    let y = r.y + (r.height.saturating_sub(desired_height)) / 2;
    Rect::new(x, y, desired_width, desired_height)
}

fn truncate_with_ellipsis(s: &str, available_width: usize, text_size: usize) -> String {
    let max_len = available_width.saturating_sub(text_size+1); // +1 buffer
    if s.len() <= max_len {

        s.to_string()

    } else {

        format!("…{}", &s[s.len().saturating_sub(max_len)..]) // …

    }

}

fn generate_dashboard_content(app: &mut App, inner_area: &Rect) ->  Box<dyn WidgetRef> {
    // dynamically fill the content (right box) of dashboard depending of the current selection
    match app.dashboard_select {
        DashboardSelect::StaticOption => {
            match app.dashboard_selected_static {
                0 => {
                    let text = app.current_realm.as_ref().unwrap().ca.get_info_txt().unwrap();
                    update_scroll(app, &inner_area, &text);
                    return Box::new(mywidgets::ScrollableParagraph::new(text, app.scroll, app.max_scroll));
                },
                1 => {
                    return Box::new(Paragraph::new("Static1")
                        .block(Block::default().borders(Borders::NONE))
                        .alignment(Alignment::Center));
                },
                2 => {
                    return Box::new(Paragraph::new("Static2")
                        .block(Block::default().borders(Borders::NONE))
                        .alignment(Alignment::Center));
                },
                3 => {
                    return Box::new(Paragraph::new("Static3")
                        .block(Block::default().borders(Borders::NONE))
                        .alignment(Alignment::Center));
                },
                _ => {
                    return Box::new(Paragraph::new("Static_shouldneverappear")
                        .block(Block::default().borders(Borders::NONE))
                        .alignment(Alignment::Center));
                },
            }
        }
        DashboardSelect::CertList => {
            match app.dashboard_selected_static {
                0 => {
                    return Box::new(Paragraph::new("Cert0")
                        .block(Block::default().borders(Borders::NONE))
                        .alignment(Alignment::Center));
                },
                _ => {
                    return Box::new(Paragraph::new("Cert_If_List_Empty")
                        .block(Block::default().borders(Borders::NONE))
                        .alignment(Alignment::Center));
                }, // appear if list empty
            }
        }
    };
    
}

fn update_scroll(app: &mut App, inner_area: &Rect, text: &String){
    // Calculate scroll size
    
    // Calculate content height for scrollbar
    let content_width = inner_area.width as usize;  // Use full width if no scrollbar
    let content_height = text.lines().map(|line| {
        if line.len() <= content_width {
            1
        } else {
            (line.len() + content_width - 1) / content_width
        }
    }).sum::<usize>();

    let visible_height = inner_area.height as usize;
    app.max_scroll = content_height.saturating_sub(visible_height);  // Set the global max scroll
    if app.scroll > app.max_scroll{ // because of resizing
        app.scroll = app.max_scroll;
    }
}