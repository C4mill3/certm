mod front;
use front::App;

mod tools;
use tools::utility::{CA_VAULT, resolve_path, create_folder};

fn main() -> Result<(), Box<dyn std::error::Error>> {

    create_folder(&resolve_path(CA_VAULT)?, 0o700)?;

    // create app and run it
    let mut app = App::new();
    let res = app.run();

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}