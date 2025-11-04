use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use sha2::digest::crypto_common::Key;
use std::io;
use ratatui::{
    Frame, Terminal, backend::CrosstermBackend, layout::{Alignment, Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style, Stylize}, symbols::{DOT, half_block::UPPER}, text::Line, widgets::{self, Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap, block::Title}
};

//backend
use crate::tools;
use tools::certs_manager::{Realm, Cert};


#[derive(Clone)]
pub enum AppState {
    SelectRealm,
    //NewRealmForm,
    PasswordPrompt,
    Dashboard,
}

#[derive(Clone)]
pub enum Focus {
    StaticOption,
    CertList,
    Content,
}

#[derive(Clone)]
pub struct App {
    pub state: AppState,
    pub realm_list: Vec<String>,
    pub selected_realm: usize,
    pub password: String,
    pub success: bool,
    pub static_menu: Vec<String>,
    pub cert_list: Vec<String>,
    pub selected_static: usize,
    pub selected_cert: usize,
    pub focus: Focus,
}

impl App {
    pub fn new() -> Self {
        let realm_list: Vec<String> = Realm::list().unwrap();

        let static_menu = vec![
            "CA Info".to_string(),
            "Create Cert".to_string(),
            "Import Cert".to_string(),
        ];
        let cert_list = vec![
            "cert1".to_string(),
            "cert2".to_string(),
            "cert3".to_string(),
            "cert4".to_string(),
            "cert5".to_string(),
            "cert6".to_string(),
            "cert7".to_string(),
            "cert8".to_string(),
            "cert9".to_string(),
            "cert10".to_string(),
        ];
        Self {
            state: AppState::SelectRealm,
            realm_list,
            selected_realm: 0,
            password: String::new(),
            success: false,
            static_menu,
            cert_list,
            selected_static: 0,
            selected_cert: 0,
            focus: Focus::StaticOption,
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        // setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let mut running = true;
        while running {
            terminal.draw(|f| ui(f, self))?;
            
            if let Event::Key(key) = event::read()? {
                match self.state {
                    AppState::SelectRealm => match key.code {
                        KeyCode::Down => self.next_realm(),
                        KeyCode::Home => self.fast_previous_realm(),
                        KeyCode::End => self.fast_next_realm(),
                        KeyCode::Up => self.previous_realm(),
                        KeyCode::Enter => self.select_realm(),
                        KeyCode::Esc => running = false,
                        _ => {}
                    },
                    AppState::PasswordPrompt => match key.code {
                        KeyCode::Enter => self.submit_password(),
                        KeyCode::Esc => self.back_to_select_realm(),
                        KeyCode::Backspace => {
                            self.password.pop();
                        }
                        KeyCode::Char(c) => {
                            self.password.push(c);
                        }
                        _ => {}
                    },
                    AppState::Dashboard => match key.code {
                        KeyCode::Tab => self.next_focus_dashboard(),
                        KeyCode::BackTab => self.previous_focus_dashboard(),
                        KeyCode::Down => self.next_item_dashboard(),
                        KeyCode::Up => self.previous_item_dashboard(),
                        KeyCode::Esc => {self.realm_list = Realm::list().unwrap(); self.state = AppState::SelectRealm}, // refresh list, then switch
                        _ => {}
                    },
                }
            }
        }
        
        // restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }

    pub fn next_realm(&mut self) {
        if self.selected_realm < self.realm_list.len().saturating_sub(1) {
            self.selected_realm += 1;
        }
    }

    pub fn fast_next_realm(&mut self) {
        if self.selected_realm+10 < self.realm_list.len().saturating_sub(1) {
            self.selected_realm+=10;
        }else {
            self.selected_realm = self.realm_list.len().saturating_sub(1);
        }
    }

    pub fn previous_realm(&mut self) { 
        if self.selected_realm > 0 {
            self.selected_realm = self.selected_realm.saturating_sub(1);
        }
    }

    pub fn fast_previous_realm(&mut self) { 
        if self.selected_realm > 0 {
            self.selected_realm = self.selected_realm.saturating_sub(10);
        }
    }

    pub fn select_realm(&mut self) {
        self.state = AppState::PasswordPrompt;
        self.password.clear();
    }

    pub fn back_to_select_realm(&mut self) {
        self.state = AppState::SelectRealm;
    }

    pub fn submit_password(&mut self) {
        self.success = true;
        self.state = AppState::Dashboard;
    }

    pub fn next_focus_dashboard(&mut self) {
        self.focus = match self.focus {
            Focus::StaticOption => Focus::CertList,
            Focus::CertList => Focus::Content,
            Focus::Content => Focus::StaticOption,
        };
    }

    pub fn previous_focus_dashboard(&mut self) {
        self.focus = match self.focus {
            Focus::StaticOption => Focus::Content,
            Focus::CertList => Focus::StaticOption,
            Focus::Content => Focus::CertList,
        };
    }

    pub fn next_item_dashboard(&mut self) {
        match self.focus {
            Focus::StaticOption => {
                if self.selected_static < self.static_menu.len().saturating_sub(1) {
                    self.selected_static += 1;
                }
            }
            Focus::CertList => {
                if self.selected_cert < self.cert_list.len().saturating_sub(1) {
                    self.selected_cert += 1;
                }
            }
            Focus::Content => {}
        }
    }

    pub fn previous_item_dashboard(&mut self) {
        match self.focus {
            Focus::StaticOption => {
                if self.selected_static > 0 {
                    self.selected_static = self.selected_static.saturating_sub(1);
                }
            }
            Focus::CertList => {
                if self.selected_cert > 0 {
                    self.selected_cert = self.selected_cert.saturating_sub(1);
                }
            }
            Focus::Content => {}
        }
    }
}

pub fn ui(f: &mut Frame, app: &mut App) {
    let size = f.size();
    match app.state {
        AppState::SelectRealm => draw_select_realm(f, app, size, true),
        AppState::PasswordPrompt => {
            draw_select_realm(f, app, size, false);
            draw_password_prompt(f, app, size);
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
            let style = if i == app.selected_realm {
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
    state.select(Some(app.selected_realm));

    f.render_stateful_widget(list, inner_area, &mut state);
}

fn draw_password_prompt(f: &mut Frame, app: &mut App, size: Rect) {
    let area = popup_rect(25, 5, 40, 20, size);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(app.realm_list[app.selected_realm].as_str());
    f.render_widget(Clear, area); // clear the background
    f.render_widget(&block, area);

    let inner_area = block.inner(area);
    let text = vec![
        Line::from(""),
        Line::from(format!("Password:{}", "*".repeat(app.password.len()))),
        Line::from(""),
    ];
    let paragraph = Paragraph::new(text).wrap(Wrap { trim: true });
    f.render_widget(paragraph, inner_area);
}

fn draw_dashboard(f: &mut Frame, app: &mut App, size: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(Title::from(app.realm_list[app.selected_realm].as_str()).alignment(Alignment::Center));
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
            Constraint::Length(app.static_menu.len() as u16 + 2), // +2 for borders
            Constraint::Min(1),
        ])
        .split(left_inner);

    // Static menu block
    let static_block = Block::default().borders(Borders::ALL).border_type(BorderType::Rounded);
    let static_inner = static_block.inner(left_chunks[0]);
    f.render_widget(static_block, left_chunks[0]);

    let static_items: Vec<ListItem> = app
        .static_menu
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == app.selected_static && matches!(app.focus, Focus::StaticOption) {
                Style::default().fg(Color::Black).bg(Color::White)
            } else {
                Style::default()
            };
            ListItem::new(item.as_str()).style(style)
        })
        .collect();

    let static_list = List::new(static_items)
        .block(Block::default().borders(Borders::NONE))
        .highlight_style(if matches!(app.focus, Focus::StaticOption) {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        })
        .highlight_symbol(if matches!(app.focus, Focus::StaticOption) { "> " } else { "" });

    let mut static_state = ListState::default();
    if matches!(app.focus, Focus::StaticOption) {
        static_state.select(Some(app.selected_static));
    }

    f.render_stateful_widget(static_list, static_inner, &mut static_state);

    // Cert list block
    let cert_block = Block::default().borders(Borders::ALL).border_type(BorderType::Rounded);
    let cert_inner = cert_block.inner(left_chunks[1]);
    f.render_widget(cert_block, left_chunks[1]);

    let cert_items: Vec<ListItem> = app
        .cert_list
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == app.selected_cert && matches!(app.focus, Focus::CertList) {
                Style::default().fg(Color::Black).bg(Color::White)
            } else {
                Style::default()
            };
            ListItem::new(item.as_str()).style(style)
        })
        .collect();

    let cert_list = List::new(cert_items)
        .block(Block::default().borders(Borders::NONE))
        .highlight_style(if matches!(app.focus, Focus::CertList) {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        })
        .highlight_symbol(if matches!(app.focus, Focus::CertList) { "> " } else { "" });

    let mut cert_state = ListState::default();
    if matches!(app.focus, Focus::CertList) {
        cert_state.select(Some(app.selected_cert));
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
fn popup_rect(min_width: u16, min_height: u16, percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let desired_width = ((r.width * percent_x) / 100).max(min_width).min(r.width);
    let desired_height = ((r.height * percent_y) / 100).max(min_height).min(r.height);
    let x = r.x + (r.width.saturating_sub(desired_width)) / 2;
    let y = r.y + (r.height.saturating_sub(desired_height)) / 2;
    Rect::new(x, y, desired_width, desired_height)
}