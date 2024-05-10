use std::{
    fs::{self, DirEntry},
    path::PathBuf,
};

use compiler::Compiler;
use comrak::{markdown_to_html, Options};
use itertools::Itertools;
use maud::Markup;
use rocket::{fs::FileServer, http::Status};

#[macro_use]
extern crate rocket;

mod compiler;
mod views;

#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub struct Recipe {
    _path: PathBuf,
    slug: String,
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
impl Eq for Recipe {}
impl Ord for Recipe {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.slug.cmp(&other.slug)
    }
}

fn map_to_recipe(entry: DirEntry) -> Recipe {
    let slug = entry
        .file_name()
        .to_str()
        .unwrap()
        .to_string()
        .replace(".md", "");
    let title = slug.replace("-", " ");

    let recipe = Recipe {
        _path: entry.path(),
        slug,
        title,
        content: fs::read_to_string(entry.path().join("index.md")).unwrap_or(String::from("")),
    };
    println!("{:?}", recipe);
    return recipe;
}
fn get_recipes(path: String) -> Result<Vec<Recipe>, String> {
    println!("In get recipes");
    let dir = fs::read_dir(path);
    let recipes_dir = dir.unwrap().filter_ok(|res| res.path().is_dir());

    let entries = recipes_dir.map_ok(map_to_recipe).collect();
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
    let recipes_path = std::env::var("APP_RECIPES_PATH").unwrap_or("./recipes".into());
    let compiled_path = std::env::var("APP_COMPILED_PATH").unwrap_or("./compiled".into());
    let compiler = Compiler::new(compiled_path.clone());

    match get_recipes(recipes_path) {
        Ok(recipes) => compiler
            .compile_recipes(recipes)
            .expect("Failed to compile recipes"),
        Err(e) => {
            println!("Get recipes failed {}", e);
        }
    }
    rocket::build()
        .mount("/", routes![index])
        .mount("/", FileServer::from(compiled_path))
}

#[cfg(test)]
mod test {
    use std::{fs, io};

    use super::rocket;
    use rocket::form::validate::Contains;
    use rocket::http::Status;
    use rocket::local::blocking::Client;
    use tempdir::TempDir;

    fn setup_env(tmp_recipe_dir: &TempDir, tmp_compiled_dir: &TempDir) -> Result<(), io::Error> {
        std::env::set_var("APP_RECIPES_PATH", tmp_recipe_dir.path());
        std::env::set_var("APP_COMPILED_PATH", tmp_compiled_dir.path());

        Ok(())
    }

    #[test]
    fn index() -> Result<(), io::Error> {
        let tmp_recipe_dir = TempDir::new("test_recipes_index")?;
        let tmp_compiled_dir = TempDir::new("test_compiled_index")?;
        std::fs::create_dir(tmp_recipe_dir.path().join("soy-salmon"))
            .expect("Failed to create soy-salmon dir");
        let file_path = tmp_recipe_dir.path().join("soy-salmon").join("index.md");
        fs::write(file_path, "# Honey Soy Salmon")?;
        setup_env(&tmp_recipe_dir, &tmp_compiled_dir).expect("Failed to setup tempdirs");

        let client = Client::tracked(rocket()).expect("valid rocket instance");

        let response = client.get("/").dispatch();
        assert_eq!(response.status(), Status::Ok);
        let content = response.into_string();
        assert_eq!(content.contains("What's for dinner?"), true);
        assert_eq!(content.contains("href=\"/recipes/soy-salmon\""), true);
        assert_eq!(content.contains("soy salmon"), true);

        tmp_recipe_dir.close()?;
        tmp_compiled_dir.close()?;
        Ok(())
    }

    #[test]
    fn read_recipe() -> Result<(), io::Error> {
        let tmp_recipe_dir = TempDir::new("test_recipes_read")?;
        let tmp_compiled_dir = TempDir::new("test_compiled_read")?;
        std::fs::create_dir(tmp_recipe_dir.path().join("soy-salmon"))
            .expect("Failed to create soy-salmon dir");
        let file_path = tmp_recipe_dir.path().join("soy-salmon").join("index.md");
        fs::write(file_path, "# Honey Soy Salmon")?;
        setup_env(&tmp_recipe_dir, &tmp_compiled_dir).expect("Failed to setup tempdirs");

        let client = Client::tracked(rocket()).expect("valid rocket instance");

        let expected_compiled_path = tmp_compiled_dir
            .path()
            .join("recipes")
            .join("soy-salmon")
            .join("index.html");

        let response = client.get("/recipes/soy-salmon").dispatch();
        assert_eq!(response.status(), Status::Ok);
        let content = response.into_string();
        assert_eq!(content.contains("Honey Soy Salmon"), true);

        let compiled =
            fs::read_to_string(expected_compiled_path).expect("Compiled file not created");
        assert_eq!(compiled.contains("Honey Soy Salmon"), true);

        tmp_recipe_dir.close()?;
        tmp_compiled_dir.close()?;
        Ok(())
    }
}
