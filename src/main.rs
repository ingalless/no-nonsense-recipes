use std::{fs, path::PathBuf};

use comrak::Options;
use maud::{html, Markup};
use rocket::http::Status;

#[macro_use]
extern crate rocket;

mod views;

pub struct Recipe {
    _path: PathBuf,
    title: String,
    content: String,
}

#[get("/recipe/<recipe>")]
fn read_recipe(recipe: &str) -> Result<Markup, Status> {
    let content = fs::read_to_string(format!("./recipes/{}", recipe)).unwrap_or(String::from(""));
    Ok(html! {
        p { (recipe) }
        (maud::PreEscaped(comrak::markdown_to_html(&content, &Options::default())))
    })
}

fn get_recipes() -> Result<Vec<Recipe>, String> {
    let dir = fs::read_dir("./recipes");
    let entries = dir
        .unwrap()
        .map(|res| {
            res.map(|e| Recipe {
                _path: e.path(),
                title: e.file_name().to_str().unwrap().to_string(),
                content: fs::read_to_string(e.path()).unwrap_or(String::from("")),
            })
        })
        .collect::<Result<Vec<_>, std::io::Error>>();
    return match entries {
        Ok(recipes) => Ok(recipes),
        Err(e) => Err(e.to_string()),
    };
}

#[get("/")]
fn index() -> Result<Markup, Status> {
    return match get_recipes() {
        Ok(recipes) => Ok(views::index(recipes)),
        _ => Err(Status::InternalServerError),
    };
}

#[launch]
fn rocket() -> _ {
    match get_recipes() {
        Ok(recipes) => {
            for recipe in recipes {
                println!(
                    "{:?}",
                    comrak::markdown_to_html(&recipe.content, &Options::default())
                )
            }
        }
        Err(e) => {
            println!("{}", e);
        }
    }
    rocket::build().mount("/", routes![index, read_recipe])
}
