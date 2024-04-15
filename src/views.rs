use maud::{html, Markup};

use crate::Recipe;

pub fn index(recipes: Vec<Recipe>) -> Markup {
    html! {
        h1 { "No Nonsense Recipes" }
        p { "I got fed up with recipe sites with lots of ads, so I decided to build my own." }
        ul {
            @for recipe in recipes {
                li { a href=(format!("/recipe/{}", recipe.title)) { (recipe.title) } }
            }
        }
    }
}

pub fn recipe(title: &String, content: &String) -> Markup {
    html! {
        p { (title) }
        (maud::PreEscaped(content))
    }
}
