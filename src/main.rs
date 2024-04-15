use std::{fs, path::PathBuf};

use comrak::{markdown_to_html, Options};
use maud::Markup;
use rocket::http::{ContentType, Status};

#[macro_use]
extern crate rocket;

mod views;

pub struct Recipe {
    _path: PathBuf,
    title: String,
    content: String,
}

#[get("/recipe/<recipe>")]
fn read_recipe(recipe: &str) -> Result<(Status, (ContentType, String)), Status> {
    let content =
        fs::read_to_string(format!("./compiled/{}.html", recipe)).unwrap_or(String::from(""));
    Ok((Status::Ok, (ContentType::HTML, content)))
}

fn get_recipes() -> Result<Vec<Recipe>, String> {
    let dir = fs::read_dir("./recipes");
    let entries = dir
        .unwrap()
        .map(|res| {
            res.map(|e| Recipe {
                _path: e.path(),
                title: e
                    .file_name()
                    .to_str()
                    .unwrap()
                    .to_string()
                    .replace(".md", ""),
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
                let write_result = std::fs::write(
                    format!("./compiled/{}.html", recipe.title),
                    views::recipe(
                        &recipe.title,
                        &markdown_to_html(recipe.content.as_str(), &Options::default()),
                    )
                    .into_string(),
                );
                match write_result {
                    Ok(_) => println!("Wrote {}", recipe.title),
                    _ => println!("Failed to write {}", recipe.title),
                };
            }
        }
        Err(e) => {
            println!("{}", e);
        }
    }
    rocket::build().mount("/", routes![index, read_recipe])
}
