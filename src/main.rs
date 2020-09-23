#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

use rocket::http::RawStr;
use rocket::response::NamedFile;
use rocket_contrib::serve::StaticFiles;
use rocket::State;
use std::sync::atomic::{AtomicUsize, Ordering};
use rocket::response::content;

#[get("/")]
fn index() -> Option<NamedFile> {
    NamedFile::open("static/index.html").ok()
}

#[get("/hello/<name>")]
fn hello_name(name: &RawStr) -> String {
    format!("Hello, {}!", name.as_str())
}

#[get("/hello")]
pub fn hello() -> &'static str {
    "Hello, outside world!"
}

struct HitCount(AtomicUsize);

#[get("/visitors")]
fn visitors(hit_count: State<HitCount>) -> content::Html<String> {
    hit_count.0.fetch_add(1, Ordering::Relaxed);
    let msg = "Your visit has been recorded!";
    let count = format!("Visits: {}", count(hit_count));
    content::Html(format!("{}<br /><br />{}", msg, count))
}

#[get("/count")]
fn count(hit_count: State<HitCount>) -> String {
    hit_count.0.load(Ordering::Relaxed).to_string()
}

fn main() {
    rocket::ignite()
    .mount("/", routes![index, hello, hello_name, visitors, count])
    .mount("/static", StaticFiles::from("static"))
    .manage(HitCount(AtomicUsize::new(0)))
    .launch();
}