use std::path::Path;

mod tools;


fn main(){
    let args = [".gitignore"];
    println!("{}", tools::utility::run_command("/bin/cat", &args));
    println!("-------");
    let dir  = Path::new("datas");
    println!("{:?}", tools::utility::list_in_path(dir, tools::utility::FSItemType::All));
    println!("{:?}", tools::utility::list_in_path(dir, tools::utility::FSItemType::Files));
    println!("{:?}", tools::utility::list_in_path(dir, tools::utility::FSItemType::Directory));
    return;
} 
