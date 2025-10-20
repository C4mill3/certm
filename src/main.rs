mod tools;

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


    let submenu2 = tools::front::Menu::new(
        "Menu / Submenu 1 / Submenu 2",
        vec![
            tools::front::MenuItem::Action("sub-choice 1", sub_action1.clone()),
            tools::front::MenuItem::Action("sub-choice 2", sub_action2.clone()),
        ],
    );


    // Create submenu for Choice 2
    let submenu = tools::front::Menu::new(
        "Menu / Submenu 1",
        vec![
            tools::front::MenuItem::Submenu("sub-choice 1", submenu2),
        ],
    );

    // Create main menu
    let mut main_menu = tools::front::Menu::new(
        "Menu",
        vec![
            tools::front::MenuItem::Action("List", action1.clone()),
            tools::front::MenuItem::Submenu("Create", submenu),
            tools::front::MenuItem::Action("Import", action1.clone()),
            tools::front::MenuItem::Action("Export", action1.clone()),
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
