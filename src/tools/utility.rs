use std::process::Command;
use std::{env, str};
use std::{io, path::Path, path::PathBuf};
use std::fs::{self, set_permissions, Permissions, File};
use std::os::unix::fs::PermissionsExt; // For Unix only
use std::io::Write;
use shellexpand;

pub const CA_VAULT : &'static str = "~/.ca_vault/";


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

pub fn list_in_path(path: &Path, filter : FSItemType) -> Result<Vec<String>, Box<dyn std::error::Error>> {
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

    }else{
        return Err(Box::from("No such path "));
    }
    return Ok(items);
}

pub fn path_exist(path: &Path) -> Result<bool, io::Error> {
    if path.is_dir() || path.is_file(){
        return Ok(true);
    }else {
        return Ok(false);
    }
}

pub fn sanitize_name(filename: &str) -> String {
    // Remove any characters that are not alphanumeric or _ -
    let safe_name: String = filename
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .collect();

    // Ensure the name is not empty
    if safe_name.is_empty() {
        String::from("empty_name") // Or handle this case differently
    } else {
        safe_name
    }
}

pub fn resolve_path(path: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let resolved_path = shellexpand::full(path)?.to_string();
    Ok(PathBuf::from(resolved_path))
}

pub fn create_folder(path: &Path, permissions: u32) -> Result<(), Box<dyn std::error::Error>> {
    // permissions exemple : 0o600
    // Create the folder
    fs::create_dir_all(&path)?;

    // Set the permissions
    let permissions = Permissions::from_mode(permissions);
    set_permissions(&path, permissions)?;

    Ok(())
}

pub fn write_to_file(filepath: &Path, data: &[u8], permissions: u32, override_existing: bool) -> Result<(), Box<dyn std::error::Error>> {
    
    // Check for existing file if not overriding
    if !override_existing && filepath.exists() {
        return Err(Box::from("File already exists"));
    }

    // Open or create the file
    let mut file = File::create(&filepath)?;
    file.write_all(data)?;

    // Set file permissions
    let permissions = Permissions::from_mode(permissions);
    set_permissions(&filepath, permissions)?;

    Ok(())
}

pub fn delete_file(filepath: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::remove_file(filepath).map_err(|e| e.into())
}

pub fn get_working_directory() -> Result<String, Box<dyn std::error::Error>> {
    let path = env::current_dir()?;
    Ok(path.to_string_lossy().into_owned())
}