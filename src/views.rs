use itertools::sorted;
use maud::{html, Markup};
use std::collections::HashMap;

use crate::Recipe;

fn nav_link(text: &str, href: &str) -> Markup {
    html! {
        a href=(href) class="text-teal-600 tracking-tight font-sm" { (text) }
    }
}

fn layout(rest: Markup) -> Markup {
    html! {
        (head())
        body {
            nav class="my-4 px-8 w-full flex justify-between items-center print:hidden" {
                a href="/" class="text-teal-700 text-lg" { "What's for dinner?" }
                div class="space-x-4" {
                    (nav_link("Browse by tags", "/tags"))
                }
            }
            main {
                (rest)
            }

            footer class="w-full my-12 text-xs text-center" {
                code { "</> by Jonny" }
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
        section class="mt-24 mx-8" {
            p class="text-2xl max-w-lg" { "Recipes, without the ads and newsletter popups." }

            div class="mt-24" {
                h2 class="text-xl mb-2 text-teal-900 font-bold" { "Our Recipes." }
                ul class="ml-1" {
                    @for recipe in sorted(recipes) {
                        li { a class="text-teal-600 font-bold" href=(format!("/recipes/{}", recipe.title)) { (recipe.title) } }
                    }
                }
            }
        }

        section class="mt-24 p-24 w-full bg-teal-900" {
            div class="mx-8 text-teal-50" {
                h2 class="text-teal-50 text-xl" { "Why?" }
                p class="max-w-xl text-sm" { "Each week I'm asked \"What should we have for dinner this week?\". This question stumps me. \"But it's okay\" I think to myself, \"I can Google it!\" But doing so only results in disappointment as I fight ads, popups and woefully slow websites. This website is an attempt to capture the recipes we enjoy and share them without any of the nonsense that comes with mainstream recipe websites." }
            }
        }
    })
}

pub fn tags(tags: &HashMap<String, Vec<String>>) -> Markup {
    layout(html! {
        section class="prose-sm mx-8" {
            @for (tag, recipes) in tags {
                h2 class="text-teal-900 font-bold" { (tag) }
                ul {
                    @for recipe in sorted(recipes) {
                        li { a class="text-teal-600" href=(format!("/recipes/{}", recipe)) { (recipe) } }
                    }
                }
            }
        }
    })
}

pub fn recipe(recipe: &Recipe) -> Markup {
    layout(html! {
        article class="prose-sm mx-8" {
            (maud::PreEscaped(recipe.to_html()))
        }
    })
}
