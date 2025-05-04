#[macro_use] extern crate rocket;
use rocket_dyn_templates::{Template, context};
use rocket::fs::FileServer;

mod cert_manager;

#[get("/test")]
fn test() -> &'static str {
    "Hello, world!"
}

#[get("/")]
fn index() -> Template {
    Template::render("home/index", context! { certsauth: "toasting", choice:"eeeee" })
}

#[get("/demo")]
fn demo() -> Template {
    Template::render("home/demo", context! { field: "toasting", test:"eaa" })
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Template::fairing())
        .mount("/", routes![index, test, demo])
        .mount("/static", FileServer::from("static"))
}