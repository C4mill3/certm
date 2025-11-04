// front/app.rs

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    Frame, Terminal, backend::CrosstermBackend, layout::{Alignment, Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style, Stylize}, symbols::{DOT, half_block::UPPER}, text::Line, widgets::{self, Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap, block::Title}
};

use std::io;

// Backend
use crate::tools;
use tools::certs_manager::{Realm, Cert};

#[derive(Clone)]
pub enum AppState {
    SelectRealm,
    PasswordPrompt,
    NewRealmForm,
    Dashboard,
}

#[derive(Clone)]
pub enum DashboardFocus {
    StaticOption,
    CertList,
    Content,
}

#[derive(Clone)]
pub struct App {
    pub state: AppState,

    // SelectRealm
    pub realm_list: Vec<String>,
    pub realm_selected: usize,
    
    // PasswordPrompt
    pub password_text: String,
    pub password_success: bool,
    
    // Dashboard
    pub dashboard_static_menu: Vec<String>,
    pub dashboard_cert_list: Vec<String>,
    pub dashboard_selected_static: usize,
    pub dashboard_selected_cert: usize,
    pub dashboard_focus: DashboardFocus,

    // Fields for NewRealmForm (nrf)
    pub nrf_selected_field: usize, // 0: Name, 1: Password, 2: Common Name, 3: Organization, 4: Country, 5: Key Size, 6: Cancel, 7: Create
    pub nrf_name: String,
    pub nrf_form_password: String,
    pub nrf_ca_common_name: String,
    pub nrf_ca_organization: String,
    pub nrf_ca_country: String,
    pub nrf_ca_key_size_index: usize, // 0: 1024, 1: 2048, 2: 4096
}

impl App {
    pub fn new() -> Self {
        let realm_list: Vec<String> = Realm::list().unwrap();

        let dashboard_static_menu = vec![
            "CA Info".to_string(),
            "Create Cert".to_string(),
            "Import Cert".to_string(),
        ];
        let dashboard_cert_list = vec![
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
            realm_selected: 0,
            
            password_text: String::new(),
            password_success: false,

            dashboard_static_menu,
            dashboard_cert_list,
            dashboard_selected_static: 0,
            dashboard_selected_cert: 0,
            dashboard_focus: DashboardFocus::StaticOption,

            nrf_selected_field: 0,
            nrf_name: String::new(),
            nrf_form_password: String::new(),
            nrf_ca_common_name: String::new(),
            nrf_ca_organization: String::new(),
            nrf_ca_country: String::new(),
            nrf_ca_key_size_index: 2, // Default to 4096
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
            terminal.draw(|f| super::ui(f, self))?;
            
            if let Event::Key(key) = event::read()? {
                match self.state {
                    AppState::SelectRealm => match key.code {
                        KeyCode::Down => self.realm_next(),
                        KeyCode::Home => self.realm_fast_previous(),
                        KeyCode::End => self.realm_fast_next(),
                        KeyCode::Up => self.realm_previous(),
                        KeyCode::Enter => self.realm_select_action(),
                        KeyCode::Char('n') | KeyCode::Char('N') => self.state = AppState::NewRealmForm,
                        KeyCode::Esc => running = false,
                        _ => {}
                    },
                    AppState::PasswordPrompt => match key.code {
                        KeyCode::Enter => self.password_submit(),
                        KeyCode::Esc => self.back_to_select_realm(),
                        KeyCode::Backspace => {
                            self.password_text.pop();
                        }
                        KeyCode::Char(c) => {
                            self.password_text.push(c);
                        }
                        _ => {}
                    },
                    AppState::NewRealmForm => match key.code {
                        KeyCode::Up => self.nrf_previous_field(),
                        KeyCode::Down => self.nrf_next_field(),
                        KeyCode::Left => {
                            if self.nrf_selected_field == 5 {
                                self.nrf_change_key_size(false); // Decrease key size
                            }
                        }
                        KeyCode::Right => {
                            if self.nrf_selected_field == 5 {
                                self.nrf_change_key_size(true); // Increase key size
                            }
                        }
                        KeyCode::Enter => self.nrf_activate_field(),
                        KeyCode::Backspace => self.nrf_backspace(),
                        KeyCode::Char(c) => self.nrf_input_char(c),
                        KeyCode::Esc => self.back_to_select_realm(),
                        _ => {}
                    },
                    AppState::Dashboard => match key.code {
                        KeyCode::Tab => self.dashboard_next_focus(),
                        KeyCode::BackTab => self.dashboard_previous_focus(),
                        KeyCode::Down => self.dashboard_next_item(),
                        KeyCode::Up => self.dashboard_previous_item(),
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

    pub fn realm_next(&mut self) {
        if self.realm_selected < self.realm_list.len().saturating_sub(1) {
            self.realm_selected += 1;
        }
    }

    pub fn realm_fast_next(&mut self) {
        if self.realm_selected+10 < self.realm_list.len().saturating_sub(1) {
            self.realm_selected+=10;
        }else {
            self.realm_selected = self.realm_list.len().saturating_sub(1);
        }
    }

    pub fn realm_previous(&mut self) { 
        if self.realm_selected > 0 {
            self.realm_selected = self.realm_selected.saturating_sub(1);
        }
    }

    pub fn realm_fast_previous(&mut self) { 
        if self.realm_selected > 0 {
            self.realm_selected = self.realm_selected.saturating_sub(10);
        }
    }

    pub fn realm_select_action(&mut self) {
        self.state = AppState::PasswordPrompt;
        self.password_text.clear();
    }

    pub fn back_to_select_realm(&mut self) {
        self.state = AppState::SelectRealm;
    }

    pub fn password_submit(&mut self) {
        self.password_success = true;
        self.state = AppState::Dashboard;
    }

    // FORM NEW REALM

    pub fn nrf_next_field(&mut self) {
        if self.nrf_selected_field < 7 {
            self.nrf_selected_field += 1;
        }
    }

    pub fn nrf_previous_field(&mut self) {
        if self.nrf_selected_field > 0 {
            self.nrf_selected_field = self.nrf_selected_field.saturating_sub(1);
        }
    }

    pub fn nrf_activate_field(&mut self) {
        // Activate button and maybe other
        match self.nrf_selected_field {
            6 => {
                // Create realm (placeholder - implement based on your backend)
                if self.nrf_create_realm() {
                    self.realm_list = Realm::list().unwrap(); // Refresh list
                    self.state = AppState::SelectRealm;
                    self.nrf_clear_form();
                }
            }
            7 => { // Cancel
                self.nrf_clear_form();
                self.back_to_select_realm();
            }
            _ => {} // Do nothing for input fields
        }
    }

    pub fn nrf_input_char(&mut self, c: char) {
        let field = match self.nrf_selected_field {
            0 => &mut self.nrf_name,
            1 => &mut self.nrf_form_password,
            2 => &mut self.nrf_ca_common_name,
            3 => &mut self.nrf_ca_organization,
            4 => &mut self.nrf_ca_country,
            _ => return, // Not an input field
        };
        if field.len() < 255 {
            field.push(c);
        }
    }

    pub fn nrf_backspace(&mut self) {
        let field = match self.nrf_selected_field {
            0 => &mut self.nrf_name,
            1 => &mut self.nrf_form_password,
            2 => &mut self.nrf_ca_common_name,
            3 => &mut self.nrf_ca_organization,
            4 => &mut self.nrf_ca_country,
            _ => return, // Not an input field
        };
        field.pop();
    }

    pub fn nrf_change_key_size(&mut self, increase: bool) {
        if increase && self.nrf_ca_key_size_index < 2 {
            self.nrf_ca_key_size_index += 1;
        } else if !increase && self.nrf_ca_key_size_index > 0 {
            self.nrf_ca_key_size_index -= 1;
        }
    }

    pub fn nrf_clear_form(&mut self) {
        self.nrf_name.clear();
        self.nrf_form_password.clear();
        self.nrf_ca_common_name.clear();
        self.nrf_ca_organization.clear();
        self.nrf_ca_country.clear();
        self.nrf_ca_key_size_index = 2; // Reset to 4096
        self.nrf_selected_field = 0;
    }

    // Placeholder for creating a realm - implement this based on your backend
    
    pub fn nrf_get_key_size(&self) -> u16 {
        match self.nrf_ca_key_size_index {
            0 => 1024,
            1 => 2048,
            2 => 4096,
            _ => 2048,
        }
    }

    pub fn nrf_create_realm(&self) -> bool {
        let _ = self.nrf_get_key_size();
        // Example: Call your backend function here
        // Realm::create(&self.name, &self.form_password, &self.common_name, &self.organization, &self.country, self.get_key_size()).is_ok()
        true // Placeholder success
    }


    // DASHBOARD
    pub fn dashboard_next_focus(&mut self) {
        self.dashboard_focus = match self.dashboard_focus {
            DashboardFocus::StaticOption => DashboardFocus::CertList,
            DashboardFocus::CertList => DashboardFocus::Content,
            DashboardFocus::Content => DashboardFocus::StaticOption,
        };
    }

    pub fn dashboard_previous_focus(&mut self) {
        self.dashboard_focus = match self.dashboard_focus {
            DashboardFocus::StaticOption => DashboardFocus::Content,
            DashboardFocus::CertList => DashboardFocus::StaticOption,
            DashboardFocus::Content => DashboardFocus::CertList,
        };
    }

    pub fn dashboard_next_item(&mut self) {
        match self.dashboard_focus {
            DashboardFocus::StaticOption => {
                if self.dashboard_selected_static < self.dashboard_static_menu.len().saturating_sub(1) {
                    self.dashboard_selected_static += 1;
                }
            }
            DashboardFocus::CertList => {
                if self.dashboard_selected_cert < self.dashboard_cert_list.len().saturating_sub(1) {
                    self.dashboard_selected_cert += 1;
                }
            }
            DashboardFocus::Content => {}
        }
    }

    pub fn dashboard_previous_item(&mut self) {
        match self.dashboard_focus {
            DashboardFocus::StaticOption => {
                if self.dashboard_selected_static > 0 {
                    self.dashboard_selected_static = self.dashboard_selected_static.saturating_sub(1);
                }
            }
            DashboardFocus::CertList => {
                if self.dashboard_selected_cert > 0 {
                    self.dashboard_selected_cert = self.dashboard_selected_cert.saturating_sub(1);
                }
            }
            DashboardFocus::Content => {}
        }
    }
}