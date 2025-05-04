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
            .expect("failed blablabla");
    
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
    use std::{io, fs, path::Path};
    use super::utility;
    use std::io::Write;

    // Var
    const CAS_ROOT : &'static str = "datas";

    // func

    fn generate_new_ca(common_name : &str, country : &str, state : &str, locality : &str) -> io::Result<bool> {
        
        let main_folder = &Path::new(CAS_ROOT).join(common_name);

        if utility::path_exist(main_folder)? {
            return Ok(false);
        }

        fs::create_dir( main_folder);

        fs::create_dir(&Path::new(main_folder).join("certs"));
        fs::create_dir(&Path::new(main_folder).join("crl"));
        fs::create_dir(&Path::new(main_folder).join("private"));
        let index_path = main_folder.join("index.txt");
        fs::File::create(&index_path)?; 

        let serial_path = main_folder.join("serial");
        let mut serial_file = fs::File::create(&serial_path)?;
        serial_file.write_all(b"1000")?;

        // Command
        let command = "openssl";

        let key_path = main_folder.join("private").join("ca.key");
        let key_str = key_path.to_str().expect("Invalid key path");
        
        let args = ["genpkey", "-algorithm", "RSA", "-out", key_str, "-pkeyopt", "rsa_keygen_bits:4096"];
        
        let _output = utility::run_command(command, &args);


        return Ok(true);
    }

    
}