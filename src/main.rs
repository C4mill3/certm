#[macro_use] extern crate rocket;
use rocket_dyn_templates::{Template, context};
use rocket::fs::FileServer;

#[get("/test")]
fn test() -> &'static str {
    "Hello, world!"
}

#[get("/")]
fn index() -> Template {
    Template::render("home/index", context! { field: "toasting", test:"eeeee" })
}

#[get("/demo")]
fn demo() -> Template {
    Template::render("home/demo", context! { field: "toasting", test:"eeeee" })
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Template::fairing())
        .mount("/", routes![index, test, demo])
        .mount("/static", FileServer::from("static"))
}