use std::{fs, path::PathBuf};

use comrak::{markdown_to_html, Options};
use maud::Markup;
use rocket::http::{ContentType, Status};

#[macro_use]
extern crate rocket;

mod views;

#[derive(Clone)]
pub struct Recipe {
    _path: PathBuf,
    title: String,
    content: String,
}

impl Recipe {
    pub fn to_html(self: &Self) -> String {
        let mut options = Options::default();
        options.parse.relaxed_tasklist_matching = true;
        options.extension.tasklist = true;
        options.extension.front_matter_delimiter = Some("---".into());
        markdown_to_html(&self.content, &options)
    }
}

#[get("/recipe/<recipe>")]
fn read_recipe(recipe: &str) -> Result<(Status, (ContentType, String)), Status> {
    let content =
        fs::read_to_string(format!("./compiled/{}.html", recipe)).unwrap_or(String::from(""));
    Ok((Status::Ok, (ContentType::HTML, content)))
}

fn get_recipes(path: String) -> Result<Vec<Recipe>, String> {
    let dir = fs::read_dir(path);
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
    let path = std::env::var("APP_RECIPES_PATH").unwrap_or("./recipes".into());
    return match get_recipes(path) {
        Ok(recipes) => Ok(views::index(recipes)),
        _ => Err(Status::InternalServerError),
    };
}

#[launch]
fn rocket() -> _ {
    let path = std::env::var("APP_RECIPES_PATH").unwrap_or("./recipes".into());
    match get_recipes(path) {
        Ok(recipes) => {
            for recipe in recipes {
                let write_result = std::fs::write(
                    format!("./compiled/{}.html", recipe.title),
                    views::recipe(&recipe).into_string(),
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

#[cfg(test)]
mod test {
    use std::{fs, io};

    use super::rocket;
    use crate::rocket_uri_macro_index;
    use rocket::form::validate::Contains;
    use rocket::http::Status;
    use rocket::local::blocking::Client;
    use rocket::uri;
    use tempdir::TempDir;

    #[test]
    fn index() -> Result<(), io::Error> {
        let tmp_dir = TempDir::new("recipes")?;
        let file_path = tmp_dir.path().join("soy-salmon.md");
        fs::write(file_path, "#Honey Soy Salmon")?;
        std::env::set_var("APP_RECIPES_PATH", tmp_dir.path());

        let client = Client::tracked(rocket()).expect("valid rocket instance");

        let response = client.get(uri!(index)).dispatch();
        assert_eq!(response.status(), Status::Ok);
        let content = response.into_string();
        assert_eq!(content.contains("No Nonsense Recipes"), true);
        assert_eq!(content.contains("soy-salmon"), true);

        Ok(())
    }
}
