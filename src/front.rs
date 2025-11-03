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