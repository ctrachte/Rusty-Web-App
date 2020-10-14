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
// Serve = Custom handler and options for static file serving.
// This handler makes it simple to serve static files from a directory on the local file system. 
// To use it, construct a StaticFiles using either [StaticFiles::from()] or [StaticFiles::new()] then simply mount the handler at a desired path.
// When mounted, the handler will generate route(s) that serve the desired static files.
use rocket::Request;
use rocket::response::Redirect;
use rocket::State;
//Request guard to retrieve managed state.
//This type can be used as a request guard to retrieve the state Rocket is managing for some type T. 
//This allows for the sharing of state across any number of handlers.
 //A value for the given type must previously have been registered to be managed by Rocket via Rocket::manage(). 
 //The type being managed must be thread safe and sendable across thread boundaries. In other words, it must implement [Send] + [Sync] + 'static.
use rocket::response::content;
use rocket::request::{Form, FormError, FormDataError};
use rocket_contrib::templates::{Template, handlebars};
use std::sync::atomic::{AtomicUsize, Ordering};
//An integer type which can be safely shared between threads.
// This type has the same in-memory representation as the underlying integer type,
// usize. For more about the differences between atomic types and non-atomic types
 // as well as information about the portability of this type, please see the module-level documentation.
use handlebars::{Helper, Handlebars, Context, RenderContext, Output, HelperResult, JsonRender};

#[derive(Serialize)]
struct TemplateContext {
    title: &'static str,
    name: Option<String>,
    items: Vec<&'static str>,
    // This key tells handlebars which template is the parent.
    parent: &'static str,
}

struct HitCount(AtomicUsize);

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

// route demonstrating handlebars templating
#[get("/handlebars")]
fn handlebars() -> Redirect {
    Redirect::to("/hello/Caleb_Trachte")
}

// route that templates a custom name query string
#[get("/hello/<name>")]
fn hello_name(name: String) -> Template {
    Template::render("index", &TemplateContext {
        title: "Hello",
        name: Some(name),
        items: vec!["One", "Two", "Three"],
        parent: "layout",
    })
}

// basic "about" route that demonstrates templating with handlebars
#[get("/about")]
fn about() -> Template {
    Template::render("about", &TemplateContext {
        title: "About",
        name: None,
        items: vec!["Four", "Five", "Six"],
        parent: "layout",
    })
}

// 404 catch helper route
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

// index route which returns a static html page
#[get("/")] // <-- route attribute
fn index() -> Option<NamedFile> { // <-- route handler
    NamedFile::open("static/index.html").ok()
}

// this returns only a static string, a simple example
// #[get("/hello/<name>")]
// fn hello_name(name: &RawStr) -> String {
//     format!("Hello, {}!", name.as_str())
// }

// this returns only a static string, a simple example
#[get("/hello")]
pub fn hello() -> &'static str {
    "Hello, outside world!"
}

// returns a static html form
#[get("/form")]
fn form() -> Option<NamedFile> {
    NamedFile::open("static/Form.html").ok()
}

// posts the form to a test handler
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

// uses our managed state to track the hit count to this route
#[get("/visitors")]
fn visitors(hit_count: State<HitCount>) -> content::Html<String> {
    hit_count.0.fetch_add(1, Ordering::Relaxed);
    let msg = "Your visit has been recorded!";
    let count = format!("Visits: {}", count(hit_count));
    content::Html(format!("{}<br /><br />{}", msg, count))
}

// helper method to return total count as a string
#[get("/count")]
fn count(hit_count: State<HitCount>) -> String {
    hit_count.0.load(Ordering::Relaxed).to_string()
}

// the main function of a route
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
    // mounting each of the routes...
    .manage(HitCount(AtomicUsize::new(0)))
    //Add state to the state managed by this instance of Rocket.
    // This method can be called any number of times as long as each call refers to a different T.
    // Managed state can be retrieved by any request handler via the State request guard. In particular,
    // if a value of type T is managed by Rocket, adding State<T> to the list of arguments
    // in a request handler instructs Rocket to retrieve the managed value
    .register(catchers![not_found])
    // registering the catchers: 404 etc. we can add one here as an example.
    .attach(Template::custom(|engines| {
        engines.handlebars.register_helper("wow", Box::new(wow_helper));
    }))
    .launch();
}