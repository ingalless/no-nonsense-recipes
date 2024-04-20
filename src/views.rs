use maud::{html, Markup};

use crate::Recipe;

fn layout(rest: Markup) -> Markup {
    html! {
        (head())
        body class="m-8" {
            nav class="my-4 w-full border-b print:hidden" {
                a href="/" class="text-blue-500" { "Home" }
            }
            main {
                (rest)
            }
        }
    }
}

fn head() -> Markup {
    html! {
        head {
            meta name="viewport" content="width=device-width, initial-scale=1" {}
            script src="https://cdn.tailwindcss.com?plugins=typography" {}
        }
    }
}

pub fn index(recipes: Vec<Recipe>) -> Markup {
    layout(html! {
        section class="prose" {
            h1 { "No Nonsense Recipes" }
            p { "Recipes, without the ads and newsletter popups." }
            p { em { "Print friendly too!" } }
            ul {
                @for recipe in recipes {
                    li { a href=(format!("/recipe/{}", recipe.title)) { (recipe.title) } }
                }
            }
        }
    })
}

pub fn recipe(recipe: &Recipe) -> Markup {
    layout(html! {
        article class="prose-sm" {
            (maud::PreEscaped(recipe.to_html()))
        }
    })
}
