#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate serde_derive;

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
use rocket::Request;
use rocket::response::Redirect;
use rocket::State;
use std::sync::atomic::{AtomicUsize, Ordering};
use rocket::response::content;
use rocket::request::{Form, FormError, FormDataError};
use rocket_contrib::templates::{Template, handlebars};
use handlebars::{Helper, Handlebars, Context, RenderContext, Output, HelperResult, JsonRender};

#[derive(Serialize)]
struct TemplateContext {
    title: &'static str,
    name: Option<String>,
    items: Vec<&'static str>,
    // This key tells handlebars which template is the parent.
    parent: &'static str,
}

#[get("/handlebars")]
fn handlebars() -> Redirect {
    Redirect::to("/hello/Caleb_Trachte")
}

#[get("/hello/<name>")]
fn hello_name(name: String) -> Template {
    Template::render("index", &TemplateContext {
        title: "Hello",
        name: Some(name),
        items: vec!["One", "Two", "Three"],
        parent: "layout",
    })
}

#[get("/about")]
fn about() -> Template {
    Template::render("about", &TemplateContext {
        title: "About",
        name: None,
        items: vec!["Four", "Five", "Six"],
        parent: "layout",
    })
}

#[catch(404)]
fn not_found(req: &Request) -> Template {
    let mut map = std::collections::HashMap::new();
    map.insert("path", req.uri().path());
    Template::render("error/404", &map)
}

fn wow_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output
) -> HelperResult {
    if let Some(param) = h.param(0) {
        out.write("<b><i>")?;
        out.write(&param.value().render())?;
        out.write("</b></i>")?;
    }

    Ok(())
}

// fn rocket() -> rocket::Rocket {
//     rocket::ignite()
//         .mount("/", routes![index, hello, about])
//         .register(catchers![not_found])
//         .attach(Template::custom(|engines| {
//             engines.handlebars.register_helper("wow", Box::new(wow_helper));
//         }))
// }
#[get("/")]
fn index() -> Option<NamedFile> {
    NamedFile::open("static/index.html").ok()
}

// #[get("/hello/<name>")]
// fn hello_name(name: &RawStr) -> String {
//     format!("Hello, {}!", name.as_str())
// }

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

#[get("/form")]
fn form() -> Option<NamedFile> {
    NamedFile::open("static/Form.html").ok()
}

#[post("/form", data = "<form>")]
fn test_form(form: Result<Form<FormInput>, FormError>) -> String {
    match form {
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
        form,
        test_form,
        hello,
        hello_name,
        about,
        visitors,
        count
        ])
    .mount("/static", StaticFiles::from("static"))
    .manage(HitCount(AtomicUsize::new(0)))
    .register(catchers![not_found])
    .attach(Template::custom(|engines| {
        engines.handlebars.register_helper("wow", Box::new(wow_helper));
    }))
    .launch();
}