#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

use rocket::http::RawStr;
//A RawStr is an unsanitzed, unvalidated, and undecoded raw string from an HTTP message.
// It exists to separate validated string inputs, represented by the String, &str,
//  and Cow<str> types, from unvalidated inputs, represented by &RawStr.
use rocket::response::NamedFile;
// response = Types and traits to build and send responses.
//  The return type of a Rocket handler can be any type that implements the Responder trait,
//  which means that the type knows how to generate a [Response].
// NamedFile = A file with an associated name; responds with the Content-Type based on the file extension.
use rocket_contrib::serve::StaticFiles;
use rocket::State;
use std::sync::atomic::{AtomicUsize, Ordering};
use rocket::response::content;
use rocket::request::{Form, FormError, FormDataError};

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

#[derive(Debug, FromFormValue)]
enum FormOption {
    A, B, C
}

#[derive(Debug, FromForm)]
struct FormInput<'r> {
    checkbox: bool,
    number: usize,
    #[form(field = "type")]
    radio: FormOption,
    password: &'r RawStr,
    #[form(field = "textarea")]
    text_area: String,
    select: FormOption,
}

#[post("/", data = "<sink>")]
fn sink(sink: Result<Form<FormInput>, FormError>) -> String {
    match sink {
        Ok(form) => format!("{:?}", &*form),
        Err(FormDataError::Io(_)) => format!("Form input was invalid UTF-8."),
        Err(FormDataError::Malformed(f)) | Err(FormDataError::Parse(_, f)) => {
            format!("Invalid form input: {}", f)
        }
    }
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
    .mount("/", routes![
        index,
        hello, 
        hello_name, 
        visitors,
        count
        ])
    .mount("/static", StaticFiles::from("static"))
    .manage(HitCount(AtomicUsize::new(0)))
    .launch();
}