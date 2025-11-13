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
use crate::tools::{self};
use tools::certs_manager::{Realm, Cert, KeySize, CertType};
use tools::mycrypt::{encrypt_to_file, decrypt_from_file};

#[derive(Clone)]
pub enum AppState {
    ErrorPrompt,
    SelectRealm,
    PasswordPrompt,
    NewRealmForm,
    Dashboard,
}


#[derive(Clone)]
pub struct App {
    pub state: AppState,
    pub scroll: usize,
    pub max_scroll: usize,

    // SelectRealm
    pub realm_list: Vec<String>,
    pub realm_selected: usize,
    
    // PasswordPrompt
    pub password_text: String,
    pub current_realm: Option<Realm>,
    
    // Dashboard
    pub dashboard_static_menu: Vec<String>,
    pub dashboard_selected: usize,
    pub dashboard_on_content: bool, // is focus on content box

    pub dashboard_content_cursor: usize,

    pub dashboard_newcert_type: CertType,
    pub dashboard_newcert_keysize: KeySize,
    pub dashboard_newcert_cn: String,
    pub dashboard_newcert_altdns: Vec<String>,
    pub dashboard_newcert_altip: Vec<String>,

    // Fields for NewRealmForm (nrf)
    pub nrf_cursor: usize, // 0: Name, 1: Password, 2: Common Name, 3: Organization, 4: Country, 5: Key Size, 6: Cancel, 7: Create
    pub nrf_name: String,
    pub nrf_password: String,
    pub nrf_ca_common_name: String,
    pub nrf_ca_organization: String,
    pub nrf_ca_country: String,
    pub nrf_ca_key_size_index: usize, // 0: 1024, 1: 2048, 2: 4096
    pub last_error: String,
    pub error_fallback_state: AppState,
}

impl App {
    pub fn new() -> Self {
        let realm_list: Vec<String> = Realm::list().unwrap();

        let dashboard_static_menu = vec![
            "CA Info".to_string(),
            "Create Cert".to_string(),
            "Import Cert".to_string(),
            "Sign a CSR".to_string(),
        ];

        Self {
            state: AppState::SelectRealm,
            scroll: 0,
            max_scroll: 0,

            realm_list,
            realm_selected: 0,
            
            password_text: String::new(),
            current_realm: None,

            dashboard_static_menu,
            dashboard_selected: 0,
            dashboard_on_content: false,
            dashboard_content_cursor: 0,

    
            dashboard_newcert_type: CertType::Server,
            dashboard_newcert_keysize: KeySize::Size4096,
            dashboard_newcert_cn: String::new(),
            dashboard_newcert_altdns: Vec::new(),
            dashboard_newcert_altip: Vec::new(),

            nrf_cursor: 0,
            nrf_name: String::new(),
            nrf_password: String::new(),
            nrf_ca_common_name: String::new(),
            nrf_ca_organization: String::new(),
            nrf_ca_country: String::new(),
            nrf_ca_key_size_index: 2, // Default to 4096
            
            last_error: String::new(),
            error_fallback_state:AppState::SelectRealm, // Default
            
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
                    AppState::ErrorPrompt => match key.code {
                        KeyCode::Up => self.scroll_up(),
                        KeyCode::Down => self.scroll_down(),
                        KeyCode::Home => self.scroll_fast_up(),
                        KeyCode::End => self.scroll_fast_down(),
                        KeyCode::Esc => self.state = self.error_fallback_state.clone(),
                        _ => {}
                    },
                    AppState::SelectRealm => match key.code {
                        KeyCode::Up => self.realm_previous(),
                        KeyCode::Down => self.realm_next(),
                        KeyCode::Home => self.realm_fast_previous(),
                        KeyCode::End => self.realm_fast_next(),
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
                            if self.nrf_cursor == 5 {
                                self.nrf_change_key_size(false); // Decrease key size
                            }
                        }
                        KeyCode::Right => {
                            if self.nrf_cursor == 5 {
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
                        KeyCode::Down => self.dashboard_down(),
                        KeyCode::Up => self.dashboard_up(),
                        KeyCode::Enter => self.dashboard_select(),
                        KeyCode::Left => self.dashboard_left(),
                        KeyCode::Right => self.dashboard_right(),
                        KeyCode::Esc => self.dashboard_escape(),
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

    // Select Realm
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
    
    pub fn back_to_select_realm(&mut self) {
        self.current_realm = None;
        self.realm_list = Realm::list().unwrap();
        self.state = AppState::SelectRealm
    }

    pub fn realm_select_action(&mut self) {
        self.password_text.clear();
        self.state = AppState::PasswordPrompt;
    }
    
    // Password
    pub fn password_submit(&mut self) {
        let filename = &self.realm_list[self.realm_selected];
        let realm_decode = decrypt_from_file(&filename, &self.password_text);
        match realm_decode {
            Ok(realm) => {
                self.current_realm = Some(realm);
                self.state=AppState::Dashboard
            },
            Err(e) => {
                self.password_text.clear();
                self.switch_to_error(e.to_string(), AppState::PasswordPrompt);
            },
        }
    }


    //Global Scroll
    pub fn scroll_up(&mut self){
        self.scroll = self.scroll.saturating_sub(1)
    }

    pub fn scroll_fast_up(&mut self){
        self.scroll = self.scroll.saturating_sub(10)
    }


    pub fn scroll_down(&mut self){
        if self.scroll < self.max_scroll {
            self.scroll += 1;
        }
    }

    pub fn scroll_fast_down(&mut self){
        if self.scroll < self.max_scroll {
            self.scroll += 10;
        }else {
            self.scroll = self.max_scroll;
        }
    }

    //Error
    pub fn switch_to_error(&mut self, error: String, fallback_state: AppState){
        self.error_fallback_state = fallback_state;
        self.last_error = error;
        
        self.scroll=0;
        self.state = AppState::ErrorPrompt;

    }

    
    // New Realm Form
    pub fn nrf_next_field(&mut self) {
        if self.nrf_cursor < 7 {
            self.nrf_cursor += 1;
        }
    }

    pub fn nrf_previous_field(&mut self) {
        if self.nrf_cursor > 0 {
            self.nrf_cursor = self.nrf_cursor.saturating_sub(1);
        }
    }

    pub fn nrf_activate_field(&mut self) {
        // Activate button and maybe other
        match self.nrf_cursor {
            6 => {
                // Create realm (placeholder - implement based on your backend)
                match self.nrf_create_realm() {
                    Ok(_) => {
                        self.realm_list = Realm::list().unwrap(); // Refresh list
                        self.state = AppState::SelectRealm;
                        self.nrf_clear_form();
                    },
                    Err(e) => {
                        self.switch_to_error(e.to_string(), AppState::NewRealmForm);
                    },
                }
            }
            7 => { // Cancel
                self.nrf_clear_form();
                self.back_to_select_realm();
            }
            _ => {
                self.nrf_next_field()
            }
        }
    }

    pub fn nrf_input_char(&mut self, c: char) {
        let field = match self.nrf_cursor {
            0 => &mut self.nrf_name,
            1 => &mut self.nrf_password,
            2 => &mut self.nrf_ca_common_name,
            3 => &mut self.nrf_ca_organization,
            4 => &mut self.nrf_ca_country,
            _ => return, // Not an input field
        };
        // COUNTRY have to be == 2CHAR
        if self.nrf_cursor == 4 {
            if field.len() < 2 {
                field.push(c.to_ascii_uppercase());
            }
        }else{ // the other
            if field.len() < 255 {
                field.push(c);
            }

        }

    }

    pub fn nrf_backspace(&mut self) {
        let field = match self.nrf_cursor {
            0 => &mut self.nrf_name,
            1 => &mut self.nrf_password,
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
        self.nrf_password.clear();
        self.nrf_ca_common_name.clear();
        self.nrf_ca_organization.clear();
        self.nrf_ca_country.clear();
        self.nrf_ca_key_size_index = 2; // Reset to 4096
        self.nrf_cursor = 0;
    }

    pub fn nrf_get_key_size(&self) -> KeySize {
        match self.nrf_ca_key_size_index {
            0 => KeySize::Size1024,
            1 => KeySize::Size2048,
            2 => KeySize::Size4096,
            _ => KeySize::Size2048,
        }
    }

    pub fn nrf_create_realm(&self) -> Result<(), Box<dyn std::error::Error>> {
        //BACKEND
        // Create the realm using settings, then create the vault if not already existing
        let key_size = self.nrf_get_key_size();

        
        let new_realm= Realm::new(&self.nrf_name, key_size, &self.nrf_ca_common_name, &self.nrf_ca_organization, &self.nrf_ca_country)?;
        
        return encrypt_to_file(&self.nrf_name, &self.nrf_password, &new_realm, false);
    }

    // Dashboard

    pub fn dashboard_up(&mut self) {
        if ! self.dashboard_on_content {
            self.scroll=0; // reset scroll on content when changing in menu
            self.dashboard_selected = self.dashboard_selected.saturating_sub(1);
        }else{
            match self.dashboard_selected {
                0 => self.scroll_up(), // Show CA
                _ => {}, //pass
            }
        }
    }


    pub fn dashboard_down(&mut self) {
        if ! self.dashboard_on_content {
            self.scroll=0; // reset scroll when changing view

            let realm_len = self.current_realm.as_ref().map(|realm| realm.certs.len()).unwrap_or(0);
            let total_len = self.dashboard_static_menu.len() + realm_len;
            if self.dashboard_selected < total_len.saturating_sub(1) {
                self.dashboard_selected += 1;
            }
        }else{
            match self.dashboard_selected {
                0 => self.scroll_down(), // Show CA
                _ => {}, //pass
            }
        }
    }

    pub fn dashboard_right(&mut self) {
        if ! self.dashboard_on_content{
            self.dashboard_on_content = true;
        }
    }

    pub fn dashboard_left(&mut self) {
        if self.dashboard_on_content{
            self.dashboard_on_content = false;
        }
    }

    pub fn dashboard_select(&mut self) {
        if ! self.dashboard_on_content{
            self.dashboard_on_content = true;
        }else{
            // ??
        }
    }

    pub fn dashboard_escape(&mut self) {
        if self.dashboard_on_content{
            self.dashboard_on_content = false;
        }else{
            self.current_realm = None;
            self.back_to_select_realm();
        }
    }
}