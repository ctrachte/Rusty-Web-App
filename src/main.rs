#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

use rocket::http::RawStr;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/hello/<name>")]
fn hello_name(name: &RawStr) -> String {
    format!("Hello, {}!", name.as_str())
}

#[get("/hello")]
pub fn hello() -> &'static str {
    "Hello, outside world!"
}

fn main() {
    rocket::ignite().mount("/", routes![index, hello, hello_name]).launch();
}