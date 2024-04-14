use std::{fs, path::PathBuf};

use maud::{html, Markup};
use rocket::http::Status;

#[macro_use]
extern crate rocket;

mod views;

pub struct Recipe {
    _path: PathBuf,
    title: String,
}

#[get("/recipe/<recipe>")]
fn read_recipe(recipe: &str) -> Result<Markup, Status> {
    Ok(html! {
        p { (recipe) }
    })
}

#[get("/")]
fn index() -> Result<Markup, Status> {
    let dir = fs::read_dir("./recipes");
    if dir.is_err() {
        println!("Could not read dir");
        return Err(Status::InternalServerError);
    }
    let entries = dir
        .unwrap()
        .map(|res| {
            res.map(|e| Recipe {
                _path: e.path(),
                title: e.file_name().to_str().unwrap().to_string(),
            })
        })
        .collect::<Result<Vec<_>, std::io::Error>>();
    return match entries {
        Ok(recipes) => Ok(views::index(recipes)),
        _ => Err(Status::InternalServerError),
    };
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, read_recipe])
}
