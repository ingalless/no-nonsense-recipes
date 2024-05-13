use itertools::{FilterOk, Itertools};
use serde::Deserialize;
use std::{
    array::IntoIter,
    collections::HashMap,
    fs::{self, read_to_string, DirEntry, File, ReadDir},
    io::{self, Read},
    path::{Path, PathBuf},
};

use comrak::{
    nodes::{AstNode, NodeValue},
    parse_document, Arena, Options,
};

use crate::{views, Recipe};

#[derive(Deserialize, Debug)]
struct Frontmatter {
    tags: Vec<String>,
}

impl Frontmatter {
    fn default() -> Self {
        Self { tags: Vec::new() }
    }
}

pub struct Compiler {
    options: Options,
    output_dir: PathBuf,
    src_dir: PathBuf,
}

fn process_node<'a>(node: &'a AstNode<'a>) -> Vec<String> {
    let mut tags = vec![];
    match node.data.borrow().value {
        NodeValue::FrontMatter(ref text) => {
            let raw = text.replace("---", "");
            let frontmatter: Frontmatter =
                serde_yaml::from_str(&raw).unwrap_or(Frontmatter::default());
            for tag in frontmatter.tags {
                tags.push(tag)
            }
        }
        _ => (),
    };

    tags
}

fn extract_tags(options: &Options, content: &String) -> Vec<String> {
    let arena = Arena::new();
    let root = parse_document(&arena, &content, &options);
    let mut tags: Vec<String> = vec![];

    for c in root.children() {
        for tag in process_node(&c) {
            tags.push(tag);
        }
    }

    return tags;
}

impl Compiler {
    pub fn new(src_dir: &PathBuf, output_dir: &PathBuf) -> Self {
        let mut options = Options::default();
        options.parse.relaxed_tasklist_matching = true;
        options.extension.tasklist = true;
        options.extension.front_matter_delimiter = Some("---".into());

        Self {
            src_dir: src_dir.into(),
            output_dir: output_dir.into(),
            options,
        }
    }

    fn make_recipe(path: PathBuf) -> Recipe {
        let slug = path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .replace(".md", "");
        // TODO: This should be read from the content
        let title = slug.replace("-", " ");

        Recipe { slug, title, path }
    }

    pub fn build(self: &Self) -> Result<(), String> {
        let dir = fs::read_dir(&self.src_dir);
        let recipes_dir = dir
            .expect("src dir does not exist")
            .filter_ok(|res| res.path().is_dir());
        let mut tag_map: HashMap<String, Vec<String>> = HashMap::new();

        for dir_result in recipes_dir {
            // NOTE: At this level we are currently at the folders level. We don't have any
            // markdown here.
            let entry = match dir_result {
                Ok(d) => d,
                Err(e) => {
                    println!("{}", e.to_string());
                    continue;
                }
            };

            if entry.file_name().to_str().unwrap().ends_with(".md") {
                let recipe = Self::make_recipe(entry.path());
                let content = read_to_string(recipe.path).unwrap_or("".to_string());
                for tag in extract_tags(&self.options, &content) {
                    match tag_map.get_mut(&tag) {
                        Some(v) => v.push(recipe.slug.clone()),
                        None => {
                            tag_map.insert(tag, vec![recipe.slug.clone()]);
                        }
                    }
                }
            };
        }

        // for recipe in recipes {
        //     let target_path = Path::new(&self.path).join("recipes").join(&recipe.slug);
        //     let write_result = match std::fs::create_dir_all(&target_path) {
        //         Ok(_) => std::fs::write(
        //             &target_path.join("index.html"),
        //             views::recipe(&recipe).into_string(),
        //         ),
        //         Err(_) => return Err(format!("Failed to write {}", recipe.slug)),
        //     };
        //     match write_result {
        //         Ok(_) => println!("Wrote {}", target_path.to_str().unwrap()),
        //         Err(_) => {
        //             println!("Failed to write {}", recipe.slug);
        //             return Err(format!("Failed to write {}", recipe.slug));
        //         }
        //     };
        // }
        // let tag_target_path = Path::new(&self.path).join("tags");
        // let write_result = match std::fs::create_dir_all(&tag_target_path) {
        //     Ok(_) => std::fs::write(
        //         &tag_target_path.join("index.html"),
        //         views::tags(&tag_map).into_string(),
        //     ),
        //     Err(_) => {
        //         println!("Failed to write tags page");
        //         return Err("Failed to write tags page".into());
        //     }
        // };
        // match write_result {
        //     Ok(_) => println!("Wrote {}", tag_target_path.to_str().unwrap()),
        //     Err(_) => {
        //         println!("Failed to write tags page");
        //         return Err("Failed to write tags page".into());
        //     }
        // };
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::{
        fs::{self, ReadDir},
        io,
        path::PathBuf,
    };
    use tempdir::TempDir;

    use super::{extract_tags, Compiler};

    fn setup_env(tmp_recipe_dir: &TempDir, tmp_compiled_dir: &TempDir) -> Result<(), io::Error> {
        std::env::set_var("APP_RECIPES_PATH", tmp_recipe_dir.path());
        std::env::set_var("APP_COMPILED_PATH", tmp_compiled_dir.path());

        Ok(())
    }

    #[test]
    fn it_compiles() -> Result<(), io::Error> {
        let tmp_recipe_dir = TempDir::new("test_recipes_read")?;
        let tmp_compiled_dir = TempDir::new("test_compiled_read")?;
        std::fs::create_dir(tmp_recipe_dir.path().join("soy-salmon"))
            .expect("Failed to create soy-salmon dir");
        let recipe_file = tmp_recipe_dir.path().join("soy-salmon").join("index.md");
        let image_file = tmp_recipe_dir.path().join("soy-salmon").join("image.jpg");
        fs::write(
            recipe_file,
            "# Honey Soy Salmon\n![My Picture](./image.jpg)",
        )?;
        fs::write(image_file, "my image")?;
        setup_env(&tmp_recipe_dir, &tmp_compiled_dir).expect("Failed to setup tempdirs");

        let (tmp_recipe_path, tmp_compiled_path) =
            (tmp_recipe_dir.into_path(), tmp_compiled_dir.into_path());
        let compiler = Compiler::new(&tmp_compiled_path, &tmp_recipe_path);

        compiler.build().expect("Failed to build recipes");

        let expected_compiled_path = tmp_compiled_path
            .join("recipes")
            .join("soy-salmon")
            .join("index.html");
        assert_eq!(
            fs::read_dir(tmp_compiled_path.join("recipes").join("soy-salmon"))
                .into_iter()
                .collect::<Vec<ReadDir>>()
                .len(),
            2
        );
        let compiled_recipe =
            fs::read_to_string(expected_compiled_path).expect("Compiled file not created");
        assert_eq!(compiled_recipe.contains("Honey Soy Salmon"), true);

        Ok(())
    }

    #[test]
    fn tags_extract_correctly() {
        let compiler = Compiler::new(&PathBuf::default(), &PathBuf::default());
        let content = "---
tags:
- asian
- curry
---
"
        .to_string();
        let expected_tags = vec![String::from("asian"), String::from("curry")];
        let tags = extract_tags(&compiler.options, &content);
        assert_eq!(expected_tags, tags);
    }
}
