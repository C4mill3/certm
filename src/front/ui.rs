use ratatui::{
    Frame, layout::{Alignment, Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style, Stylize}, symbols::DOT, text::{Line, Span}, widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Widget, WidgetRef, Wrap}
};

// Import App and related types from the app module
use super::app::*;
use super::mywidgets;

pub fn ui_wrong_size(f: &mut Frame, current_width :u16, current_height: u16) {
    let size = f.area();
    let area = popup_rect(5, 5, 100, 100, size);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("Error")
        .title_bottom(Line::from(vec!["Esc".red()]).right_aligned());

    f.render_widget(Clear, area); // clear the background
    f.render_widget(&block, area);

    let inner_area = block.inner(area);

    let content = Paragraph::new(format!("Minimal Size:\n 52x22\nCurrently:\n{}x{}", current_width, current_height));
    content.render(inner_area, f.buffer_mut());

}

pub fn ui_render(f: &mut Frame, app: &mut App) {
    let size = f.area();
    match app.state {
        AppState::ErrorPrompt => draw_error_prompt(f, app, size),
        AppState::SelectRealm => draw_select_realm(f, app, size, true),
        AppState::PasswordPrompt => {
            match app.password_intent {
                PasswordIntent::EnterRealm | PasswordIntent::DeleteRealm => {draw_select_realm(f, app, size, false)},
                PasswordIntent::DeleteCert | PasswordIntent::CreateCert |
                PasswordIntent::ImportCert => {draw_dashboard(f, app, size, false)}

            }
            
            draw_password_prompt(f, app, size);
        },
        AppState::NewRealmForm => {
            draw_select_realm(f, app, size, false);
            draw_new_realm_form(f, app, size);
        }
        AppState::NewRealmFromCA => {
            draw_select_realm(f, app, size, false);
            draw_new_realm_from_ca(f, app, size);
        }
        AppState::Dashboard => draw_dashboard(f, app, size, true),
        AppState::ExportCert => {
            draw_dashboard(f, app, size, false);
            draw_export_cert(f, app, size);
        },
    }
}

fn draw_error_prompt(f: &mut Frame, app: &mut App, size: Rect) {
    let area = popup_rect(50, 20, 60, 70, size);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("Error")
        .title_bottom(Line::from(vec!["↑↓".red(), DOT.into(), "Esc".red()]).right_aligned());

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
        block = block.title_bottom(Line::from(vec!["↑↓".red(), DOT.into(),
            "Select".into(), "↵".red(), DOT.into(),
            "N".red(), "ew".into(), DOT.into(),
            "I".red(), "mport".into(), DOT.into(),
            "D".red(), "elete".into(), DOT.into(),
            "Esc".red(),]).right_aligned());
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
    let title = match app.password_intent {
        PasswordIntent::EnterRealm => {format!("Enter in {}", app.realm_list[app.realm_selected].as_str())},
        PasswordIntent::DeleteRealm => {format!("Delete Realm {}", app.realm_list[app.realm_selected].as_str())},
        PasswordIntent::CreateCert => {"Generate New Cert".to_string()},
        PasswordIntent::ImportCert => {"Import Cert".to_string()},
        PasswordIntent::DeleteCert => {"Delete Cert".to_string()},
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title_bottom(Line::from(vec!["Try".into(), "↵".red(), DOT.into(), "Esc".red(),]).right_aligned())
        .title(title);

    f.render_widget(Clear, area); // clear the background
    f.render_widget(&block, area);

    let inner_area = block.inner(area);
    let text = vec![
        Line::from(""),
        Line::from(format!("Realm Password:{}", "*".repeat(app.password_text.len()))),
        Line::from(""),
    ];
    let paragraph = Paragraph::new(text).wrap(Wrap { trim: true });
    f.render_widget(paragraph, inner_area);
}

fn draw_new_realm_form(f: &mut Frame, app: &mut App, size: Rect) {
    let area = popup_rect(50, 20, 70, 80, size); // Dynamic size: 60% width/height, min 50x20
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("New Realm")
        .title_bottom(Line::from(vec!["↑↓".red(), DOT.into(), "Select".into(), "↵".red(), DOT.into(), "Esc".red(),]).right_aligned());
    f.render_widget(Clear, area);
    f.render_widget(&block, area);

    let inner_area = block.inner(area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Realm block
            Constraint::Length(11), // CA block
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

    let name_full = if app.nrf_cursor == 0 { app.nrf_name.clone() + "_" } else { app.nrf_name.clone() };
    let password_full = if app.nrf_cursor == 1 { "*".repeat(app.nrf_password.len()) + "_" } else { "*".repeat(app.nrf_password.len()) };

    let realm_lines = vec![
        Line::from(format_with_ellipsis("Name: ", &name_full, available_width)).style(if app.nrf_cursor == 0 { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() }),
        Line::from(""),
        Line::from(format_with_ellipsis("Password: ", &password_full, available_width)).style(if app.nrf_cursor == 1 { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() }),
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


    let common_name_full = if app.nrf_cursor == 2 { app.nrf_ca_common_name.clone() + "_" } else { app.nrf_ca_common_name.clone() };
    let organization_full = if app.nrf_cursor == 3 { app.nrf_ca_organization.clone() + "_" } else { app.nrf_ca_organization.clone() };
    let country_full = if app.nrf_cursor == 4 { app.nrf_ca_country.clone() + "_" } else { app.nrf_ca_country.clone() };
    
    let formatted_date = format_date(&app.nrf_valid_until.clone());
    let valid_until_full = if app.nrf_cursor == 5 { formatted_date + "_" } else { formatted_date };

    let key_sizes = [1024, 2048, 4096];
    let ca_lines = vec![
        Line::from(format_with_ellipsis("Common Name: ", &common_name_full, available_width)).style(if app.nrf_cursor == 2 { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() }),
        Line::from(""),
        Line::from(format_with_ellipsis("Organization: ", &organization_full, available_width)).style(if app.nrf_cursor == 3 { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() }),
        Line::from(""),
        Line::from(format_with_ellipsis("Country: ", &country_full, available_width)).style(if app.nrf_cursor == 4 { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() }),
        Line::from(""),
        Line::from(format_with_ellipsis("Valid Until (DD/MM/YYYY): ", &valid_until_full, available_width)).style(if app.nrf_cursor == 5 { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() }),
        Line::from(""),
        Line::from(format!("Key Size: {} {} {}",
            if app.nrf_cursor == 6 && app.nrf_ca_key_size_index > 0 { "←" } else { " " },
            key_sizes[app.nrf_ca_key_size_index],
            if app.nrf_cursor == 6 && app.nrf_ca_key_size_index < key_sizes.len().saturating_sub(1) { "→" } else { " " })).style(if app.nrf_cursor == 6 { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() }),
    ];
    let ca_paragraph = Paragraph::new(ca_lines).wrap(Wrap { trim: true });
    f.render_widget(ca_paragraph, ca_inner);

    // Buttons
    let button_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[2]);

    let create_text = if app.nrf_cursor == 7 { "[Create]" } else { " Create " };
    let create_paragraph = Paragraph::new(create_text)
        .alignment(Alignment::Center)
        .style(if app.nrf_cursor == 7 { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() });
    f.render_widget(create_paragraph, button_chunks[0]);

    let cancel_text = if app.nrf_cursor == 8 { "[Cancel]" } else { " Cancel " };
    let cancel_paragraph     = Paragraph::new(cancel_text)
        .alignment(Alignment::Center)
        .style(if app.nrf_cursor == 8 { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() });
    f.render_widget(cancel_paragraph, button_chunks[1]);

}

fn draw_new_realm_from_ca(f: &mut Frame, app: &mut App, size: Rect) {
    let area = popup_rect(50, 15, 70, 60, size); // Dynamic size: 60% width/height, min 50x20
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("New Realm From CA")
        .title_bottom(Line::from(vec!["↑↓".red(), DOT.into(), "Select".into(), "↵".red(), DOT.into(), "Esc".red(),]).right_aligned());

    f.render_widget(Clear, area);
    f.render_widget(&block, area);

    let inner_area = block.inner(area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Realm block
            Constraint::Length(5), // CA block
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

    let name_full = if app.nrf_cursor == 0 { app.nrf_name.clone() + "_" } else { app.nrf_name.clone() };
    let password_full = if app.nrf_cursor == 1 { "*".repeat(app.nrf_password.len()) + "_" } else { "*".repeat(app.nrf_password.len()) };

    let realm_lines = vec![
        Line::from(format_with_ellipsis("Name: ", &name_full, available_width)).style(if app.nrf_cursor == 0 { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() }),
        Line::from(""),
        Line::from(format_with_ellipsis("Password: ", &password_full, available_width)).style(if app.nrf_cursor == 1 { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() }),
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


    let pem_path_full = if app.nrf_cursor == 2 { app.nrf_ca_pem_path.clone() + "_" } else { app.nrf_ca_pem_path.clone() };
    let key_path_full = if app.nrf_cursor == 3 { app.nrf_ca_key_path.clone() + "_" } else { app.nrf_ca_key_path.clone() };
    

    let ca_lines = vec![
        Line::from(format_with_ellipsis("CA pem file path: ", &pem_path_full, available_width)).style(if app.nrf_cursor == 2 { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() }),
        Line::from(""),
        Line::from(format_with_ellipsis("CA key file path: ", &key_path_full, available_width)).style(if app.nrf_cursor == 3 { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() }),
    ];
    let ca_paragraph = Paragraph::new(ca_lines).wrap(Wrap { trim: true });
    f.render_widget(ca_paragraph, ca_inner);

    // Buttons
    let button_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[2]);

    let create_text = if app.nrf_cursor == 4 { "[Create]" } else { " Create " };
    let create_paragraph = Paragraph::new(create_text)
        .alignment(Alignment::Center)
        .style(if app.nrf_cursor == 4 { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() });
    f.render_widget(create_paragraph, button_chunks[0]);

    let cancel_text = if app.nrf_cursor == 5 { "[Cancel]" } else { " Cancel " };
    let cancel_paragraph     = Paragraph::new(cancel_text)
        .alignment(Alignment::Center)
        .style(if app.nrf_cursor == 5 { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() });
    f.render_widget(cancel_paragraph, button_chunks[1]);

}

fn draw_dashboard(f: &mut Frame, app: &mut App, size: Rect, tips : bool) {
    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(Line::from(app.realm_list[app.realm_selected].as_str()).centered());
    if tips{
        block = block.title_bottom(Line::from(generate_dashboard_tips(app.dashboard_selected, app.dashboard_on_content)).right_aligned());
    }
    let inner_area = block.inner(size);
    f.render_widget(block, size);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(inner_area);

    // Left panel: split into static menu and cert list
    let left_inner = chunks[0];

    let static_len = app.dashboard_static_menu.len();

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(static_len as u16 + 2), // +2 for borders
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
            let style = if i == app.dashboard_selected && !app.dashboard_on_content {
                Style::default().fg(Color::Black).bg(Color::White)
            } else {
                Style::default()
            };
            ListItem::new(item.as_str()).style(style)
        })
        .collect();

    let static_list = List::new(static_items)
        .block(Block::default().borders(Borders::NONE))
        .highlight_style(if app.dashboard_selected < static_len { Style::default().add_modifier(Modifier::BOLD) }else { Style::default() })
        .highlight_symbol(if app.dashboard_selected < static_len { "> " }else { "  " });

    let mut static_state = ListState::default();
    static_state.select(Some(app.dashboard_selected));

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
            let style = if i + static_len == app.dashboard_selected
                                && !app.dashboard_on_content {
                Style::default().fg(Color::Black).bg(Color::White)
            } else {
                Style::default()
            };
            ListItem::new(item.get_subject_name().unwrap()).style(style)
        })
        .collect();

    let cert_list = List::new(cert_items)
        .block(Block::default().borders(Borders::NONE))
        .highlight_style(if app.dashboard_selected >= static_len { Style::default().add_modifier(Modifier::BOLD) }else { Style::default() })
        .highlight_symbol(if app.dashboard_selected >= static_len { "> " }else { "  " });

    let mut cert_state = ListState::default();
    cert_state.select(Some(app.dashboard_selected.saturating_sub(static_len)));

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

fn generate_dashboard_content(app: &mut App, inner_area: &Rect) ->  Box<dyn WidgetRef> {
    // dynamically fill the content (right box) of dashboard depending of the current selection
    match app.dashboard_selected {
        0 => {
            let text = app.current_realm.as_ref().unwrap().ca.get_info_txt().unwrap();
            update_scroll(app, &inner_area, &text);
            return Box::new(mywidgets::ScrollableParagraph::new(text, app.scroll, app.max_scroll));
        },
        1 => {
            return Box::new(mywidgets::NewCertForm::new(app.dashboard_on_content, app.dashboard_content_cursor, app.dashboard_newcert_type, app.dashboard_newcert_keysize, &app.dashboard_newcert_cn, &app.dashboard_newcert_altdns, &app.dashboard_newcert_altip, &app.dashboard_newcert_validuntil));
        },
        2 => {
            return Box::new(mywidgets::ImportCertForm::new(app.dashboard_on_content, app.dashboard_content_cursor, &app.dashboard_newcert_pem_path, &app.dashboard_newcert_key_path));
        },
        _ => { // x > 3 -> Cert
            let cert_id = app.get_selected_cert();
            let text = app.current_realm.as_ref().unwrap().certs[cert_id].get_info_txt().unwrap();
            update_scroll(app, &inner_area, &text);
            return Box::new(mywidgets::ScrollableParagraph::new(text, app.scroll, app.max_scroll));
        },
    }
    
}

fn generate_dashboard_tips(dashboard_selected: usize, on_content: bool) -> Vec<Span<'static>> {
    // dynamically write the tips for the dashboard AppState
    let mut tips: Vec<Span<'_>> = vec![
            "↑↓".red(), DOT.into(),
            "Select".into(), "↵".red(), DOT.into(),
            "Esc".red(),];
    match dashboard_selected {
        0 => {
            if on_content{
                tips.splice(4..4,vec![DOT.into(), "E".red(), "xport".into(),]);
            }
            return tips;
        },
        1 => {
            return tips;
        },
        2 => {
            return tips;
        },
        _ => { // x > 3 -> Cert
            if on_content{
                tips.splice(4..4,vec![DOT.into(), "E".red(), "xport".into(), DOT.into(), "D".red(), "elete".into(),]);
            }
            return tips;
        },
    }
    
}

fn draw_export_cert(f: &mut Frame, app: &mut App, size: Rect) {
    let area = popup_rect(48, 7, 40, 20, size);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title_bottom(Line::from(vec!["Select".into(), "↵".red(), DOT.into(), "Esc".red(),]).right_aligned())
        .title("Export");

    f.render_widget(Clear, area); // clear the background
    f.render_widget(&block, area);

    // Calculate available width for Realm fields
    let available_width = block.inner(area).width as usize;

    let path_full = if app.dashboard_content_cursor == 0 { app.export_cert_path.clone() + "_" } else { app.export_cert_path.clone() };
    let export_private_full = if app.export_cert_private {"✓"} else{"𐄂"};

    let export_button_full = if app.dashboard_content_cursor == 2 {"[Export]"} else {"Export"};

    let inner_area = block.inner(area);
    let text = vec![
        Line::from(format_with_ellipsis("Folder: ", &path_full, available_width)).style(if app.dashboard_content_cursor == 0 { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() }),
        Line::from(""),
        Line::from(format!("Export Private Key: {}", export_private_full)).style(if app.dashboard_content_cursor == 1 { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() }),
        Line::from(""),
        Line::from(export_button_full).style(if app.dashboard_content_cursor == 2 { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() }).alignment(Alignment::Center),
    ];
    let paragraph = Paragraph::new(text).wrap(Wrap { trim: true });
    f.render_widget(paragraph, inner_area);
}

pub fn popup_rect(min_width: u16, min_height: u16, percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let desired_width = ((r.width * percent_x) / 100).max(min_width).min(r.width);
    let desired_height = ((r.height * percent_y) / 100).max(min_height).min(r.height);
    let x = r.x + (r.width.saturating_sub(desired_width)) / 2;
    let y = r.y + (r.height.saturating_sub(desired_height)) / 2;
    Rect::new(x, y, desired_width, desired_height)
}


pub fn format_with_ellipsis(prompt: &str, input: &str, available_width: usize) -> String {
    let text_size = prompt.len();
    let max_len = available_width.saturating_sub(text_size+1); // +1 buffer
    if input.len() <= max_len {

        return format!("{}{}", prompt, input.to_string());

    } else {

        return format!("{}…{}", &prompt, &input[input.len().saturating_sub(max_len)..]); // …

    }
}

pub fn update_scroll(app: &mut App, inner_area: &Rect, text: &String){
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

pub fn format_date(input: &str) -> String {
    let mut formatted = String::new();
    
    // Append characters to formatted string with appropriate slashes
    for (i, c) in input.chars().enumerate() {
        formatted.push(c);
        // Add slashes accordingly
        if i == 1 || i == 3 {
            formatted.push('/');
        }
    }
    
    formatted
}