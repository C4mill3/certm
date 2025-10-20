mod utility{
    use std::process::Command;
    use std::str;
    use std::{io, fs, path::Path};

    pub enum FSItemType {
        All,
        Files,
        Directory
    }
    
    pub fn run_command(command : &str, args : &[&str]) -> String {
        let mut construct_command = Command::new(command);
        if args.len() > 0 {
            construct_command.args(args);
        }
        let output = construct_command
            .output()
            .expect("failed");
    
        let stdout = str::from_utf8(&output.stdout).expect("Invalid UTF-8 sequence");
    
        return stdout.to_string(); 
    }
    
    pub fn list_in_path(path: &Path, filter : FSItemType) -> io::Result<Vec<String>> {
        let mut items: Vec<String> = Vec::new();
    
        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
    
                match filter {
                    FSItemType::All => {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            items.push(name.to_string());
                        }
                    }
                    FSItemType::Files => {
                        if path.is_file() {
                            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                                items.push(name.to_string());
                            }
                        }
                    }
                    FSItemType::Directory => {
                        if path.is_dir() {
                            if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                                items.push(dir_name.to_string());
                            }
                        }
                    }
                }
                
            }
            return Ok(items);
        } else {
            return Ok(Vec::new());
        }
    }

    pub fn path_exist(path: &Path) -> Result<bool, io::Error> {
        if path.is_dir() || path.is_file(){
            return Ok(true);
        }else {
            return Ok(false);
        }
    }
    

}    


pub mod certs_manager{
    
    // Import
    use std::{fs, io, path::Path};
    use super::utility::{self, FSItemType};
    use std::io::Write;

    // Var
    const CAS_ROOT : &'static str = "datas";

    // func

    pub fn generate_new_ca(key_size : u32, common_name : &str, organization : &str, country : &str) -> io::Result<bool> {
        
        if key_size != 2048 && key_size != 4096 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "key_size must be either 2048 or 4096",
            ));
        }

        let main_folder = &Path::new(CAS_ROOT).join(common_name);

        if utility::path_exist(main_folder)? {
            return Ok(false);
        }
        
        // Init File Structure

        fs::create_dir(main_folder)?;
        fs::create_dir(&main_folder.join("certs"))?;
        fs::create_dir(&main_folder.join("csr"))?;
        fs::create_dir(&main_folder.join("private"))?;
        fs::create_dir(&main_folder.join("ca"))?;
        let index_path = main_folder.join("index.txt");
        fs::File::create(&index_path)?; 

        let serial_path = main_folder.join("serial");
        let mut serial_file = fs::File::create(&serial_path)?;
        serial_file.write_all(b"1000")?;

        // Command
        let openssl_command = "openssl";
        
        // Generate private CA key
        let key_path = main_folder.join("ca").join("ca.key");
        let key_path_str = key_path.to_str().expect("Invalid key path");
        let keygen_option = format!("rsa_keygen_bits:{}", key_size);
        let args = [ "genpkey", "-algorithm", "RSA", "-out", key_path_str, "-pkeyopt", &keygen_option ];
        
        utility::run_command(openssl_command, &args);

        // autosigning CA (generate public CA)
        let ca_cert_path = main_folder.join("ca").join("ca.crt");
        let ca_cert_str = ca_cert_path.to_str().expect("Invalid CA certificate path");
        let subj_str = format!("/C={}/O={}/CN={}",
                               country, organization, common_name);
        let args = [ "req", "-key", key_path_str, "-new", "-x509",
                                         "-out", ca_cert_str, "-days", "3650", "-subj", &subj_str];
        
        utility::run_command(openssl_command, &args);

        return Ok(true);
    }


    pub fn list_ca() -> io::Result<Vec<String>> {
        let root_folder = &Path::new(CAS_ROOT);

        return utility::list_in_path( root_folder, FSItemType::Directory);
    }

    pub fn list_cert(ca_name: &str) -> io::Result<Vec<String>> {
        let root_folder = &Path::new(CAS_ROOT);
        let ca_folder = root_folder.join(ca_name).join("certs");
        if !ca_folder.exists(){
            return Err(io::Error::new(io::ErrorKind::NotFound, "CA folder not found"));
        }

        let list = utility::list_in_path( &ca_folder, FSItemType::Files)?;
        
        let mut certs_names = Vec::new();
        for filename in list{
            let name = if let Some(pos) = filename.rfind('.') {
                filename[..pos].to_string()
            }else{
                filename
            };
            certs_names.push(name);
        }
        return Ok(certs_names);

    }

    pub fn generate_new_cert(ca_name: &str, key_size: i32, common_name: &str, organization : &str, country : &str) -> std::io::Result<bool> {

        let ca_folder = Path::new(CAS_ROOT).join(ca_name);
        if !ca_folder.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "CA folder not found"));
        }

        if key_size != 2048 && key_size != 4096 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "key_size must be either 2048 or 4096",
            ));
        }

        // Define paths for the CA certificate and key
        let ca_cert_path = ca_folder.join("ca").join("ca.crt");
        let ca_cert_path_str = ca_cert_path.to_str().expect("Invalid cert path");
        let ca_key_path = ca_folder.join("ca").join("ca.key");
        let ca_key_path_str = ca_key_path.to_str().expect("Invalid key path");

        if !ca_cert_path.exists() || !ca_key_path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "CA certificate or key not found"));
        }

        // Define paths for the new server key, CSR, and certificate
        let cert_key_path = ca_folder.join("private").join(format!("{}.key", common_name));
        let cert_key_path_str = cert_key_path.to_str().expect("Invalid key path");
        let cert_csr_path = ca_folder.join("csr").join(format!("{}.csr", common_name));
        let cert_csr_path_str = cert_csr_path.to_str().expect("Invalid csr path");
        let cert_pub_path = ca_folder.join("certs").join(format!("{}.crt", common_name));
        let cert_pub_path_str = cert_pub_path.to_str().expect("Invalid cert path");
        if cert_pub_path.exists() {
            return Ok(false);
        }


        // Command
        let openssl_command = "openssl";

        // Cert private key

        let keygen_option = format!("rsa_keygen_bits:{}", key_size);
        let args = [ "genpkey", "-algorithm", "RSA", "-out",cert_key_path_str,
                                        "-pkeyopt", &keygen_option];
        utility::run_command(openssl_command, &args);

        // Cert csr
        let subj = format!( "/C={}/O={}/CN={}", country, organization, common_name);
        let args = [ "req", "-new", "-key", cert_key_path_str, "-out", cert_csr_path_str, "-subj", &subj ];
        utility::run_command(openssl_command, &args);

        
        // Create an extension file with the subjectAltName for the server certificate
        let conf_file_path = ca_folder.join("temp.cnf");
        let conf_file_path_str = conf_file_path.to_str().expect("invalid config file path");

        fs::copy("config/default_x509.cnf", conf_file_path_str)?;
        
        let toadd = format!("\nDNS.1 = {}\n", common_name);
        // TODO modulable DNS.2, IP, ETC
        let mut file = fs::OpenOptions::new().append(true).open(&conf_file_path)?;
        file.write_all(toadd.as_bytes())?;
        
        // Sign the server certificate with the CA certificate and key
        let args = [ "x509", "-req", "-in", cert_csr_path_str, "-CA", ca_cert_path_str, "-CAkey", ca_key_path_str,
                                         "-out", cert_pub_path_str, "-days", "365", "-extfile", conf_file_path_str,];
        utility::run_command(openssl_command, &args);
        
        let _ = fs::remove_file(conf_file_path);

        Ok(true)
    }

    pub fn rm_ca(ca_name: &str) -> std::io::Result<bool> {
        
        let ca_folder = Path::new(CAS_ROOT).join(ca_name);
        if !ca_folder.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "CA folder not found"));
        }

        let _ = fs::remove_file(ca_folder);


        Ok(true)
    }

}


pub mod front {
    use std::io::{self, Write};
    use std::rc::Rc;
    use crossterm::{
        cursor,
        event::{self, Event, KeyCode, KeyEvent},
        execute,
        terminal::{self, ClearType},
        ExecutableCommand,
    };

    pub enum MenuItem {
        Action(&'static str, Rc<dyn Fn()>),
        Submenu(&'static str, Menu),
    }

    pub struct Menu {
        pub title: &'static str,
        pub items: Vec<MenuItem>,
        pub current_index: usize,
    }

    impl Menu {
        pub fn new(title: &'static str, items: Vec<MenuItem>) -> Self {
            Self {
                title,
                items,
                current_index: 0,
            }
        }

        pub fn run(&mut self, stdout: &mut io::Stdout) -> Result<(), Box<dyn std::error::Error>> {
            loop {
                self.display(stdout)?;

                if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                    match code {
                        KeyCode::Up => {
                            if self.current_index > 0 {
                                self.current_index -= 1;
                            }
                        }
                        KeyCode::Down => {
                            if self.current_index < self.items.len() - 1 {
                                self.current_index += 1;
                            }
                        }
                        KeyCode::Enter => {
                            match &self.items[self.current_index] {
                                MenuItem::Action(_, action) => {
                                    // Clear screen and execute action
                                    execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0), cursor::Hide)?;
                                    // Clone the Rc to get ownership and call the closure
                                    let action_cloned = action.clone();
                                    (&*action_cloned)();
                                    return Ok(()); // end action
                                }
                                MenuItem::Submenu(_, submenu) => {
                                    // Run submenu
                                    let mut sub = submenu.clone();
                                    sub.run(stdout)?;
                                }
                            }
                        }
                        KeyCode::Esc | KeyCode::Char('q') => return Ok(()), // Exit
                        _ => {}
                    }
                }
            }
        }

        fn display(&self, stdout: &mut io::Stdout) -> Result<(), Box<dyn std::error::Error>> {
            execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0), cursor::Hide)?;
            write!(stdout, "{}\r\n", self.title)?;
            for (i, item) in self.items.iter().enumerate() {
                let prefix = if i == self.current_index { "\x1b[1m> " } else { "  " };
                write!(stdout, "{}", prefix)?;
                match item {
                    MenuItem::Action(name, _) => write!(stdout, "{}", name)?,
                    MenuItem::Submenu(name, _) => write!(stdout, "{}", name)?,
                }
                write!(stdout, "\x1b[0m\r\n")?; // reset + new line
            }
            stdout.flush()?;
            Ok(())
        }
    }

    impl Clone for Menu {
        fn clone(&self) -> Self {
            // Simple clone; actions are stored in Rc so they can be cloned cheaply
            Self {
                title: self.title,
                items: self.items.iter().map(|item| match item {
                    MenuItem::Action(name, action) => MenuItem::Action(name, action.clone()),
                    MenuItem::Submenu(name, submenu) => MenuItem::Submenu(name, submenu.clone()),
                }).collect(),
                current_index: self.current_index,
            }
        }
    }
}
