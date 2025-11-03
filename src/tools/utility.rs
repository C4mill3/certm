use std::process::Command;
use std::str;
use std::{io, path::Path, path::PathBuf};
use std::fs::{self, set_permissions, Permissions, File};
use std::os::unix::fs::PermissionsExt; // For Unix only
use std::io::Write;
use shellexpand;

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

pub fn resolve_path(path: &str) -> PathBuf {
    let resolved_path = shellexpand::tilde(path).to_string();
    PathBuf::from(resolved_path)
}

pub fn write_to_file(filepath: &Path, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    // Open or create the file with specified options
    let mut file = File::create(&filepath)?;
    file.write_all(data)?;

    // Set file permissions to 600 
    let permissions = Permissions::from_mode(0o600); // (rw- --- ---)
    set_permissions(&filepath, permissions)?;

    Ok(())
}
