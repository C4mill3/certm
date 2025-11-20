use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, size, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    Frame, Terminal, backend::CrosstermBackend, layout::{Alignment, Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style, Stylize}, symbols::{DOT, half_block::UPPER}, text::Line, widgets::{self, Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap, block::Title}
};

use std::{io};

// Backend
use crate::tools;
use tools::utility::{get_working_directory, path_exist, resolve_path, write_to_file};
use tools::certs_manager::{Realm, Cert, KeySize, CertType};
use tools::mycrypt::{encrypt_to_file, decrypt_from_file, delete_encrypted_file};

#[derive(Clone)]
pub enum AppState {
    ErrorPrompt,
    SelectRealm,
    PasswordPrompt,
    NewRealmForm,
    Dashboard,
    ExportCert,
}

#[derive(Clone)]
pub enum PasswordIntent {
    EnterRealm,
    DeleteRealm,
    CreateCert,
    DeleteCert,
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
    pub password_intent: PasswordIntent,
    pub current_realm: Option<Realm>,
    
    // Dashboard
    pub dashboard_static_menu: Vec<String>,
    pub dashboard_selected: usize,
    pub dashboard_on_content: bool, // is focus on content box
    pub dashboard_content_cursor: usize,

    pub dashboard_newcert_cn: String, // Common Name
    pub dashboard_newcert_altip: String, // Subject AltName IP
    pub dashboard_newcert_altdns: String, // Subject AltName Domain Name
    pub dashboard_newcert_validuntil: String, // Valid Until
    pub dashboard_newcert_type: usize, // 0: Server, 1: Client, 2: ServerAndClient
    pub dashboard_newcert_keysize: usize, // 0: 1024, 1: 2048, 2: 4096

    // Fields for NewRealmForm (nrf)
    pub nrf_cursor: usize, // 0: Name, 1: Password, 2: Common Name, 3: Organization, 4: Country, 5: Valid Until, 6: Key Size, 7: Cancel, 8: Create
    pub nrf_name: String,
    pub nrf_password: String,
    pub nrf_ca_common_name: String,
    pub nrf_ca_organization: String,
    pub nrf_ca_country: String,
    pub nrf_valid_until: String,
    pub nrf_ca_key_size_index: usize, // 0: 1024, 1: 2048, 2: 4096

    pub export_cert_path: String,
    pub export_cert_private: bool,

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
            password_intent: PasswordIntent::EnterRealm,
            current_realm: None,

            dashboard_static_menu,
            dashboard_selected: 0,
            dashboard_on_content: false,
            dashboard_content_cursor: 0,

    
            dashboard_newcert_cn: String::new(),
            dashboard_newcert_altdns: String::new(),
            dashboard_newcert_altip: String::new(),
            dashboard_newcert_validuntil: String::new(),
            dashboard_newcert_type: 0,
            dashboard_newcert_keysize: 2,

            nrf_cursor: 0,
            nrf_name: String::new(),
            nrf_password: String::new(),
            nrf_ca_common_name: String::new(),
            nrf_ca_organization: String::new(),
            nrf_ca_country: String::new(),
            nrf_valid_until: String::new(),
            nrf_ca_key_size_index: 2, // Default to 4096

            export_cert_path: String::new(),
            export_cert_private: false,
            
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
            let (width, height) = size()?;
            if width < 52|| height < 22 { // Size limit
                terminal.draw(|f| super::ui_wrong_size(f, width, height))?;
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Esc => {running = false},
                        _ => {} // pass
                    }
                }
            }else {
                terminal.draw(|f| super::ui_render(f, self))?;
                
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
                            KeyCode::Char('n') | KeyCode::Char('N') => {
                                self.state = AppState::NewRealmForm;
                            },
                            KeyCode::Char('d') | KeyCode::Char('D') | KeyCode::Delete | KeyCode::Backspace => self.realm_delete_action() ,
                            KeyCode::Esc => {running = false},
                            _ => {}
                        },
                        AppState::PasswordPrompt => match key.code {
                            KeyCode::Enter => self.password_submit(),
                            KeyCode::Esc => self.password_escape(),
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
                                if self.nrf_cursor == 6 {
                                    self.nrf_change_key_size(false); // Decrease key size
                                }
                            }
                            KeyCode::Right => {
                                if self.nrf_cursor == 6 {
                                    self.nrf_change_key_size(true); // Increase key size
                                }
                            }
                            KeyCode::Enter => self.nrf_activate_field(),
                            KeyCode::Backspace | KeyCode::Delete => self.nrf_backspace(),
                            KeyCode::Esc => self.back_to_select_realm(),
                            KeyCode::Char(c) => self.nrf_input_char(c),
                            _ => {}
                        },
                        AppState::Dashboard => match key.code {
                            KeyCode::Down => self.dashboard_down(),
                            KeyCode::Up => self.dashboard_up(),
                            KeyCode::Enter => self.dashboard_enter(),
                            KeyCode::Left => self.dashboard_left(),
                            KeyCode::Right => self.dashboard_right(),
                            KeyCode::Esc => self.dashboard_escape(),
                            KeyCode::Backspace | KeyCode::Delete => self.dashboard_backspace(),
                            KeyCode::Char(c) => self.dashboard_input_char(c),
                            _ => {}
                        },
                        AppState::ExportCert => match key.code {
                            KeyCode::Down => {
                                if self.dashboard_content_cursor < 2{
                                    self.dashboard_content_cursor += 1;
                                }
                            },
                            KeyCode::Up => {
                                self.dashboard_content_cursor = self.dashboard_content_cursor.saturating_sub(1);
                            },
                            KeyCode::Enter => self.exportcert_enter(),
                            KeyCode::Esc => {
                                self.state = AppState::Dashboard;
                            },
                            KeyCode::Backspace => self.exportcert_backspace(),
                            KeyCode::Char(c) => self.exportcert_input_char(c),
                            _ => {}
                        },
                    }
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
        self.password_text.clear();
        self.current_realm = None;
        self.realm_list = Realm::list().unwrap();
        self.state = AppState::SelectRealm
    }

    pub fn realm_select_action(&mut self){
        if self.realm_list.len() != 0{
            self.password_text.clear();
            self.password_intent = PasswordIntent::EnterRealm;
            self.state = AppState::PasswordPrompt;
        }else{
            self.switch_to_error("There is currently no realm.\n Use 'N' to create one.".into(), AppState::SelectRealm);
        }
    }

    pub fn realm_delete_action(&mut self) {
        if self.realm_list.len() != 0{
            self.password_text.clear();
            self.password_intent = PasswordIntent::DeleteRealm;
            self.state = AppState::PasswordPrompt;
        }
    }

    // Password
    pub fn password_escape(&mut self){
        match self.password_intent {
            PasswordIntent::EnterRealm | PasswordIntent::DeleteRealm => {self.back_to_select_realm()},
            PasswordIntent::DeleteCert | PasswordIntent::CreateCert => {
                self.password_text.clear();
                self.state=AppState::Dashboard;
            }

        }
        
    }

    pub fn password_submit(&mut self) {
        match self.password_intent {
            PasswordIntent::EnterRealm => {
                let filename = &self.realm_list[self.realm_selected];
                let realm_decode = decrypt_from_file(&filename, &self.password_text);
                match realm_decode {
                    Ok(realm) => {
                        self.current_realm = Some(realm);
                        self.dashboard_content_cursor = 0;
                        self.scroll = 0;
                        self.dashboard_selected = 0;
                        self.state=AppState::Dashboard
                    },
                    Err(e) => {
                        self.switch_to_error(e.to_string(), AppState::PasswordPrompt);
                    },
                }
            },
            PasswordIntent::DeleteRealm => {
                let realm_name = &self.realm_list[self.realm_selected];
                match delete_encrypted_file(realm_name) {
                    Ok(_) => {
                        self.realm_selected = self.realm_selected.saturating_sub(1);
                        self.back_to_select_realm();
                    },
                    Err(e) => {
                        self.switch_to_error(e.to_string(), AppState::PasswordPrompt);
                    },
                };
            },
            PasswordIntent::CreateCert => {
                // Checking file password match
                let filename = &self.realm_list[self.realm_selected];
                let realm_decode = decrypt_from_file(&filename, &self.password_text);
                match realm_decode { // Done to check password
                    Ok(_) => {
                        // Adding a new cert
                        match self.dashboard_create_cert() {
                            Ok(_) => {
                                self.reset_form_create_cert();
                                self.state=AppState::Dashboard;
                                self.dashboard_on_content = false;
                                self.dashboard_selected = self.get_dashboard_menu_len();
                            },
                            Err(e) => {
                                self.switch_to_error(e.to_string(), AppState::Dashboard);
                            },
                        }
                    },
                    Err(e) => {
                        self.switch_to_error(e.to_string(), AppState::PasswordPrompt);
                    },
                }
            }
            PasswordIntent::DeleteCert => {
                // Checking file password match
                let filename = &self.realm_list[self.realm_selected];
                let realm_decode = decrypt_from_file(&filename, &self.password_text);
                match realm_decode { // Done to check password
                    Ok(_) => {
                        // Remove a cert
                        match self.dashboard_remove_cert() {
                            Ok(_) => {
                                let realm_len = self.current_realm.as_ref().map(|realm| realm.certs.len()).unwrap_or(0);
                                let total_len = self.dashboard_static_menu.len().saturating_sub(1) + realm_len;
                                if realm_len == 0{ // No more Cert
                                    self.dashboard_selected = self.dashboard_static_menu.len().saturating_sub(1);
                                }else if self.dashboard_selected > total_len {
                                    self.dashboard_selected = total_len;
                                }
                                self.state=AppState::Dashboard;
                                self.dashboard_on_content = false;
                            },
                            Err(e) => {
                                self.switch_to_error(e.to_string(), AppState::Dashboard);
                            },
                        }
                    },
                    Err(e) => {
                        self.switch_to_error(e.to_string(), AppState::PasswordPrompt);
                    },
                }
            }
        }
        self.password_text.clear();

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
        if self.nrf_cursor < 8 {
            self.nrf_cursor += 1;
        }
    }

    pub fn nrf_previous_field(&mut self) {
        self.nrf_cursor = self.nrf_cursor.saturating_sub(1);
    }

    pub fn nrf_activate_field(&mut self) {
        // Activate button and maybe other
        match self.nrf_cursor {
            7 => {
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
            8 => { // Cancel
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
            5 => &mut self.nrf_valid_until,
            _ => return, // Not an input field
        };
        
        match self.nrf_cursor {
            4 => { // COUNTRY have to be == 2CHAR
                if field.len() < 2 && c.is_alphabetic() {
                    field.push(c.to_ascii_uppercase());
                }
            },
            5 => { // Date, limited to 8
                if field.len() < 8 && c.is_ascii_digit(){
                    field.push(c);
                }
            },
            _ => { // Other, limited to 255
                if field.len() < 255 && (c.is_ascii_alphanumeric() || vec!['!', '@', '#', '$', '%', '&', '*', '(', ')', '-', '=', '+', ' ', '.'].contains(&c)) {
                    field.push(c);
                }
            },
        };
    }

    pub fn nrf_backspace(&mut self) {
        let field = match self.nrf_cursor {
            0 => &mut self.nrf_name,
            1 => &mut self.nrf_password,
            2 => &mut self.nrf_ca_common_name,
            3 => &mut self.nrf_ca_organization,
            4 => &mut self.nrf_ca_country,
            5 => &mut self.nrf_valid_until,
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

    pub fn nrf_create_realm(&self) -> Result<(), Box<dyn std::error::Error>> {
        //BACKEND
        // Create the realm using settings, then create the vault if not already existing
        let key_size = get_key_size(self.nrf_ca_key_size_index);

        let formatted_time = format_date(&self.nrf_valid_until)?;
        
        let new_realm= Realm::new(&self.nrf_name, key_size, &self.nrf_ca_common_name, &self.nrf_ca_organization, &self.nrf_ca_country, &formatted_time)?;
        
        return encrypt_to_file(&self.nrf_name, &self.nrf_password, &new_realm, false);
    }

    // Dashboard

    pub fn dashboard_up(&mut self) {
        if ! self.dashboard_on_content {
            self.scroll=0; // reset scroll on content when changing in menu
            self.dashboard_content_cursor=0; // reset cursor when changing view

            self.dashboard_selected = self.dashboard_selected.saturating_sub(1);
        }else{
            match self.dashboard_selected {
                0 => self.scroll_up(), // Show CA
                1 => {  // New Cert (Previous Field)
                    self.dashboard_content_cursor = self.dashboard_content_cursor.saturating_sub(1);
                },
                2 | 3 => {}, //pass
                _ => { // Certs
                    self.scroll_up()
                },
            }
        }
    }


    pub fn dashboard_down(&mut self) {
        if ! self.dashboard_on_content {
            self.scroll=0; // reset scroll when changing view
            self.dashboard_content_cursor=0; // reset cursor when changing view

            let realm_len = self.current_realm.as_ref().map(|realm| realm.certs.len()).unwrap_or(0);
            let total_len = self.dashboard_static_menu.len() + realm_len;
            if self.dashboard_selected < total_len.saturating_sub(1) {
                self.dashboard_selected += 1;
            }
        }else{
            match self.dashboard_selected {
                0 => self.scroll_down(), // Show CA
                1 => {  // New Cert (Next Field)
                    if self.dashboard_content_cursor < 6 {
                        self.dashboard_content_cursor += 1;
                    }
                },
                2 | 3 => {}, //pass
                _ => { // Certs
                    self.scroll_down()
                },
            }
        }
    }

    pub fn dashboard_right(&mut self) {
        if ! self.dashboard_on_content{
            self.dashboard_on_content = true;
        }else{
            match self.dashboard_selected {
                1 => {  // New Cert
                    match self.dashboard_content_cursor {
                        4 => { // Cert Type
                            if self.dashboard_newcert_type < 2{
                                self.dashboard_newcert_type += 1;
                            }
                        }
                        5 => { // Key Size
                            if self.dashboard_newcert_keysize < 2{
                                self.dashboard_newcert_keysize += 1;
                            }
                        }
                        _ => {} //pass
                    }
                },
                _ => {}, //pass
            }
        }
    }

    pub fn dashboard_left(&mut self) {
        if self.dashboard_on_content{
            match self.dashboard_selected {
                1 => {  // New Cert
                    match self.dashboard_content_cursor {
                        4 => { // Cert Type
                            self.dashboard_newcert_type = self.dashboard_newcert_type.saturating_sub(1);
                        }
                        5 => { // Key Size
                            self.dashboard_newcert_keysize = self.dashboard_newcert_keysize.saturating_sub(1);
                        }
                        _ => {self.dashboard_on_content = false;}, // if not on any interactive field
                    }
                },
                _ => {self.dashboard_on_content = false;}, // if not on any interactive form
            }
        }
    }

    pub fn dashboard_enter(&mut self) {
        if ! self.dashboard_on_content{
            self.dashboard_on_content = true;
        }else{
            match self.dashboard_selected {
                1 => { // New Cert
                    match self.dashboard_content_cursor {
                        6 => { // New Cert (Create)
                            self.password_intent = PasswordIntent::CreateCert;
                            self.state = AppState::PasswordPrompt;
                        }
                        _ => {} //pass
                    }


                }
                _ => {} //pass
            }
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

    pub fn dashboard_input_char(&mut self, c: char) {
        if self.dashboard_on_content{
            match self.dashboard_selected {
                1 => { // New cert
                    let field = match self.dashboard_content_cursor {
                        0 => &mut self.dashboard_newcert_cn,
                        1 => &mut self.dashboard_newcert_altdns,
                        2 => &mut self.dashboard_newcert_altip,
                        3 => &mut self.dashboard_newcert_validuntil,
                        _ => return, // Not an input field
                    };
                    match self.dashboard_content_cursor {
                        2 => { // IP
                            if field.len() < 255 && (c.is_ascii_digit() || vec!['.', ','].contains(&c)){
                                field.push(c);
                            }
                        },
                        3 => { // Valid Until
                            if field.len() < 8 && c.is_ascii_digit(){
                                field.push(c);
                            }
                        },
                        _ => { //Other
                            if field.len() < 255 && (c.is_ascii_alphanumeric() || vec!['.', ',', ' '].contains(&c)){
                                field.push(c);
                            }
                        }
                    }
                }, 
                2 | 3 => {}, // Static
                0 => { // CA Info
                    if vec!['e', 'E'].contains(&c){ // Export
                        self.switch_to_export();
                    }
                },
                _ => { // Cert
                    if vec!['d', 'D'].contains(&c){ // Delete
                        self.password_intent =PasswordIntent::DeleteCert;
                        self.state = AppState::PasswordPrompt;
                    }else if vec!['e', 'E'].contains(&c){ // Export
                        self.switch_to_export();
                    }
                },
            }
        }
    }

    pub fn dashboard_backspace(&mut self) {
        if self.dashboard_on_content{
            match self.dashboard_selected {
                1 => { // New cert
                    let field = match self.dashboard_content_cursor {
                        0 => &mut self.dashboard_newcert_cn,
                        1 => &mut self.dashboard_newcert_altdns,
                        2 => &mut self.dashboard_newcert_altip,
                        3 => &mut self.dashboard_newcert_validuntil,
                        _ => return, // Not an input field
                    };
                    field.pop();
                }, 
                0 | 2 | 3 => {}, // pass static
                _ => { // Cert
                    self.password_intent =PasswordIntent::DeleteCert;
                    self.state = AppState::PasswordPrompt;
                },
            }

        }

    }

    pub fn dashboard_create_cert(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        //BACKEND
        // Create the cert and override the vault

        let altname_dns: Vec<String> = self.dashboard_newcert_altdns
            .split(',')
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();

        let altname_ip: Vec<String> = self.dashboard_newcert_altip
            .split(',')
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();

        let cert_type = get_cert_type(self.dashboard_newcert_type);
        let key_size = get_key_size(self.dashboard_newcert_keysize);
        let formatted_time = format_date(&self.dashboard_newcert_validuntil)?;

        self.current_realm.as_mut().expect("Error: Realm is empty").add_cert(cert_type, key_size, &self.dashboard_newcert_cn, &formatted_time, &altname_dns, &altname_ip)?;
        return encrypt_to_file(&self.current_realm.clone().unwrap().name.clone(), &self.password_text, self.current_realm.as_ref().unwrap(), true);
    }

    pub fn dashboard_remove_cert(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        //BACKEND
        // Remove the cert and override the vault

        let cert_id = self.dashboard_selected.saturating_sub(4);

        self.current_realm.as_mut().expect("Error: Realm is empty").remove_cert(cert_id)?;
        return encrypt_to_file(&self.current_realm.clone().unwrap().name.clone(), &self.password_text, self.current_realm.as_ref().unwrap(), true);
    }

    pub fn reset_form_create_cert(&mut self){
        self.dashboard_newcert_cn.clear();
        self.dashboard_newcert_altdns.clear();
        self.dashboard_newcert_altip.clear();
        self.dashboard_newcert_validuntil.clear();
        self.dashboard_newcert_type = 0; // Reset to Server
        self.dashboard_newcert_keysize = 2; // Reset to 4096
        self.dashboard_content_cursor = 0;
    }

    // Export

    pub fn switch_to_export(&mut self){
        self.state = AppState::ExportCert;
        self.dashboard_content_cursor = 0;
        match get_working_directory(){
            Ok(path) => {self.export_cert_path = path;},
            Err(e) =>{
                self.switch_to_error(e.to_string(), AppState::ExportCert);
            }
        }
    }

    pub fn exportcert_enter(&mut self){
        match self.dashboard_content_cursor {
            0 => {}, //pass
            1 => {self.export_cert_private = !self.export_cert_private},
            2 => { // Button Export
                match self.exportcert_action(){
                    Ok(_) => {
                        self.dashboard_content_cursor = 0;
                        self.state = AppState::Dashboard;
                    },
                    Err(e) => self.switch_to_error(e.to_string(), AppState::ExportCert),
                }
            }
            _ => {} //pass
        }
    }

    pub fn exportcert_backspace(&mut self){
        match self.dashboard_content_cursor {
            0 => { // path input field
                self.export_cert_path.pop();
            },
            _ => {} //pass
        }
    }

    pub fn exportcert_input_char(&mut self, c: char){
        match self.dashboard_content_cursor {
            0 => { // path input field
                if self.export_cert_path.len() < 255 && (c.is_ascii_alphanumeric() || vec!['.', '\\', '/', '$', '(', ')', '~'].contains(&c)){
                    self.export_cert_path.push(c);
                }
            },
            _ => {} //pass
        }        
    }

    pub fn exportcert_action(&mut self) -> Result<(), Box<dyn std::error::Error>>{
        let mut path = resolve_path(self.export_cert_path.as_ref())?;
        if path_exist(&path)? {
            let cert: Cert;
            if self.dashboard_selected == 0 { // CA
                cert = self.current_realm.as_ref().expect("Error: Realm is empty").ca.clone();
            }else{ // Cert
                let cert_id = self.dashboard_selected.saturating_sub(4);
                cert = self.current_realm.as_ref().expect("Error: Realm is empty").certs[cert_id].clone();
            }
            let name = cert.get_subject_name()?;
            let cert_pem = cert.get_cert_txt()?;
            path = path.join(name);

            if self.export_cert_private{ // Also Export Private
                let cert_key = cert.get_private_txt()?;
                path.set_extension("key");
                write_to_file(&path, cert_key.as_bytes(), 0o600, false)?;
            }

            path.set_extension("pem");
            write_to_file(&path, cert_pem.as_bytes(), 0o600, self.export_cert_private)?; // self.export_cert_private allow to remove error msg if already exported but forgot private


        }else {
            return Err(Box::from("Path does not exist"));
        }
        Ok(())
    }

    fn get_dashboard_menu_len(&mut self) -> usize {
        return self.dashboard_static_menu.len().saturating_sub(1) + self.current_realm.as_ref().map(|realm| realm.certs.len()).unwrap()
    }

    fn get_selected_cert(&mut self) -> usize {
        return self.dashboard_selected - self.dashboard_static_menu.len()

    }

}

fn get_key_size( index_keysize : usize) -> KeySize {
    match index_keysize {
        0 => KeySize::Size1024,
        1 => KeySize::Size2048,
        2 => KeySize::Size4096,
        _ => KeySize::Size2048, // Default
    }
}

fn get_cert_type( index_keysize : usize) -> CertType {
    match index_keysize {
        0 => CertType::Server,
        1 => CertType::Client,
        2 => CertType::ServerAndClient,
        _ => CertType::Server, // Default
    }
}

fn format_date(date: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Take a date at format DDMMYYYY
    // And return it as ASN1 date format: YYYYMMDDHHMMSSZ (Z=UTC)
    if date.len() != 8 {
        return Err("date has an invalid size".into());
    }

    // Extract day, month, and year using string slices
    let day = &date[0..2];
    let month = &date[2..4];
    let year = &date[4..8];

    let day_u8 = day.parse::<u8>()?;
    let month_u8 = month.parse::<u8>()?;

    if !(day_u8 >= 1 && day_u8 <= 31){
        return Err("Day is invalid".into());
    }

    if !(month_u8 >= 1 && month_u8 <= 12){
        return Err("Month is invalid".into());
    }

    // Create the ASN1 formatted date
    let formatted_date = format!("{year}{month}{day}120000Z");

    Ok(formatted_date)
}