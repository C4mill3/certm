mod front;
mod tools;

use tools::mycrypt;
use tools::certs_manager;

use std::io;
use std::rc::Rc;
use crossterm::{
    event,
    terminal,
};

pub fn run_menu() -> Result<(), Box<dyn std::error::Error>> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();

    // Define some example actions wrapped in Rc
    let action1 = Rc::new(|| {
        println!("Action 3 executed!");
        println!("Additional logic for Action 3");
        let _ = event::read(); // Wait for any key press
    });

    let sub_action1 = Rc::new(|| println!("Sub-action 1 executed!"));

    let sub_action2 = Rc::new(|| println!("Sub-action 2 executed!"));


    let submenu2 = front::Menu::new(
        "Menu / Submenu 1 / Submenu 2",
        vec![
            front::MenuItem::Action("sub-choice 1", sub_action1.clone()),
            front::MenuItem::Action("sub-choice 2", sub_action2.clone()),
        ],
    );


    // Create submenu for Choice 2
    let submenu = front::Menu::new(
        "Menu / Submenu 1",
        vec![
            front::MenuItem::Submenu("sub-choice 1", submenu2),
        ],
    );

    // Create main menu
    let mut main_menu = front::Menu::new(
        "Menu",
        vec![
            front::MenuItem::Action("List", action1.clone()),
            front::MenuItem::Submenu("Create", submenu),
            front::MenuItem::Action("Import", action1.clone()),
            front::MenuItem::Action("Export", action1.clone()),
        ],
    );

    // Run the menu system
    main_menu.run(&mut stdout)?;
    
    // Disable raw mode
    terminal::disable_raw_mode()?;
    Ok(())
}



fn main() {
    if let Err(err) = run_menu() {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}

// Test mycrypt
//fn main() -> Result<(), Box<dyn std::error::Error>> {
//    let password = "my_secret_password";
//    let data = mycrypt::MyData {
//        name: "aaaaa".to_string(),
//        age: 30,
//    };
//    println!("Data original: {:?}",  &data);
//    let _ = mycrypt::encrypt_to_file("mydata", password, &data)?;
//  
//
//    let decrypted_data = mycrypt::decrypt_from_file("mydata", "my_secret_password");
//    println!("Decrypted data: {:?}",  &decrypted_data);
//    Ok(())
//}

// Test Cert_Manager
//fn main() -> Result<(), Box<dyn std::error::Error>> {
//    let mut realm= certs_manager::Realm::new("test", certs_manager::KeySize::Size1024, "test.com", "TEST", "FR")?;
//    realm.add_cert(certs_manager::CertType::ServerAndClient, certs_manager::KeySize::Size1024, "home.org", &vec![String::from("home.org")], &vec![])?;
//    realm.add_cert(certs_manager::CertType::Server, certs_manager::KeySize::Size1024, "POLS", &vec![String::from("TEST")], &vec![])?;
//
//    println!("{:?}", realm);
//
//    let cert1= &realm.certs[0];
//    println!("See this:\n{}", cert1.get_cert_txt()?);
//    println!("This is private:\n{}", cert1.get_private_txt()?);
//    println!("here is the truth: {}", cert1.is_signed_by(&realm.ca)?);
//    println!("here is what you shouldn't see: {}", cert1.is_signed_by(&cert1)?);
//    println!("{}", &realm.certs[0].get_info_txt()?);
//    println!("{}", &realm.certs[0].get_subject_name()?);
//    
//    
//    Ok(())
//}