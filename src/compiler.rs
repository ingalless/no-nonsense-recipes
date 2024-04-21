use std::path::Path;

use comrak::{
    nodes::{AstNode, NodeValue},
    parse_document, Arena, Options,
};

use crate::{views, Recipe};

pub struct Compiler {
    path: String,
}

fn iter_nodes<'a, F>(node: &'a AstNode<'a>, f: &F)
where
    F: Fn(&'a AstNode<'a>),
{
    f(node);
    for c in node.children() {
        iter_nodes(c, f);
    }
}

fn extract_tags(content: &String) {
    let arena = Arena::new();
    let mut options = Options::default();
    options.parse.relaxed_tasklist_matching = true;
    options.extension.tasklist = true;
    options.extension.front_matter_delimiter = Some("---".into());
    let root = parse_document(&arena, &content, &options);

    iter_nodes(root, &|node| match &mut node.data.borrow_mut().value {
        &mut NodeValue::FrontMatter(ref text) => {
            println!("frontmatter: {}", text);
        }
        _ => (),
    });
}

impl Compiler {
    pub fn new(path: String) -> Self {
        Self { path }
    }

    pub fn compile_recipes(self: &Self, recipes: Vec<Recipe>) -> Result<(), String> {
        for recipe in recipes {
            extract_tags(&recipe.content);
            let target_path = Path::new(&self.path).join(format!("{}.html", recipe.title));
            let write_result = std::fs::write(&target_path, views::recipe(&recipe).into_string());
            match write_result {
                Ok(_) => println!("Wrote {}", target_path.to_str().unwrap()),
                Err(_) => {
                    println!("Failed to write {}", recipe.title);
                    return Err(format!("Failed to write {}", recipe.title));
                }
            };
        }
        Ok(())
    }
}
