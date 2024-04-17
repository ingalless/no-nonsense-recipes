use std::path::Path;

use crate::{views, Recipe};

pub struct Compiler {
    path: String,
}

impl Compiler {
    pub fn new(path: String) -> Self {
        Self { path }
    }

    pub fn compile_recipes(self: &Self, recipes: Vec<Recipe>) -> Result<(), String> {
        for recipe in recipes {
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
