use serde::Deserialize;
use std::{collections::HashMap, path::Path};

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
    path: String,
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
    pub fn new(path: String) -> Self {
        let mut options = Options::default();
        options.parse.relaxed_tasklist_matching = true;
        options.extension.tasklist = true;
        options.extension.front_matter_delimiter = Some("---".into());

        Self { path, options }
    }

    pub fn compile_recipes(self: &Self, recipes: Vec<Recipe>) -> Result<(), String> {
        let mut tag_map: HashMap<String, Vec<String>> = HashMap::new();
        for recipe in recipes {
            for tag in extract_tags(&self.options, &recipe.content) {
                match tag_map.get_mut(&tag) {
                    Some(v) => v.push(recipe.title.clone()),
                    None => {
                        tag_map.insert(tag, vec![recipe.title.clone()]);
                    }
                }
            }

            let target_path = Path::new(&self.path).join("recipes").join(&recipe.title);
            let write_result = match std::fs::create_dir_all(&target_path) {
                Ok(_) => std::fs::write(
                    &target_path.join("index.html"),
                    views::recipe(&recipe).into_string(),
                ),
                Err(_) => return Err(format!("Failed to write {}", recipe.title)),
            };
            match write_result {
                Ok(_) => println!("Wrote {}", target_path.to_str().unwrap()),
                Err(_) => {
                    println!("Failed to write {}", recipe.title);
                    return Err(format!("Failed to write {}", recipe.title));
                }
            };
        }
        let tag_target_path = Path::new(&self.path).join("tags");
        let write_result = match std::fs::create_dir_all(&tag_target_path) {
            Ok(_) => std::fs::write(
                &tag_target_path.join("index.html"),
                views::tags(&tag_map).into_string(),
            ),
            Err(_) => {
                println!("Failed to write tags page");
                return Err("Failed to write tags page".into());
            }
        };
        match write_result {
            Ok(_) => println!("Wrote {}", tag_target_path.to_str().unwrap()),
            Err(_) => {
                println!("Failed to write tags page");
                return Err("Failed to write tags page".into());
            }
        };
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::{extract_tags, Compiler};

    #[test]
    fn single() {
        let compiler = Compiler::new("".into());
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
