use axum::{extract::State, routing::get, Router};
use maud::{html, Markup, DOCTYPE};
use railwind::{parse_to_string, CollectionOptions, Source};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::{error::Error, time::Duration};

#[derive(Clone, PartialEq)]
struct Game {
    slug: String,
    name: String,
}

struct Link<'a> {
    href: &'a str,
    children: Markup,
}

impl<'a> Link<'a> {
    fn new(href: &'a str, children: Markup) -> Self {
        Self { href, children }
    }
}

fn page(children: Markup) -> Markup {
    let links = vec![
        Link::new("/", html! { "Home" }),
        Link::new("/games", html! { "Games" }),
    ];
    let body = html! {
        body ."dark:bg-slate-900" ."dark:text-white" {
            nav ."py-4" {
                ul .flex .flex-col ."sm:flex-row" .items-center .justify-center ."gap-4" {
                    @for link in &links {
                        li { a href=(link.href) ."hover:text-violet-500" { (link.children) } }
                    }
                }
            }
            main .container .mx-auto .flex .flex-col .items-center .justify-center ."gap-4" { (children) }
        }
    };
    let style = parse_to_string(
        Source::String(body.to_owned().into_string(), CollectionOptions::Html),
        true,
        &mut vec![],
    );
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width,initial-scale=1";
                title { "Funicular" }
                link
                    rel="icon"
                    type="image/svg+xml"
                    href="data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIxZW0iIGhlaWdodD0iMWVtIiB2aWV3Qm94PSIwIDAgMjQgMjQiPjxwYXRoIGZpbGw9Im5vbmUiIHN0cm9rZT0iY3VycmVudENvbG9yIiBkPSJNNy40NzggMTguMTQ5YTEuNSAxLjUgMCAwIDEtMi45NTQuNTJtMTEuOTk5LTIuMjVhMS41IDEuNSAwIDAgMCAyLjk1NC0uNTJNOCAxMS43NThWNC42MzZtOCA1LjY0OFYzLjE4Mm02Ljk3IDYuMjNjLjAxOS0uNDc3LjAzLS45OC4wMy0xLjUwM0MyMyA0LjQxIDIyLjUgMiAyMi41IDJsLTIxIDMuODE4UzEgOC40MSAxIDExLjkxYzAgLjUyMy4wMTEgMS4wMjIuMDMgMS40OTJtMjEuOTQtMy45OUMyMi44NjIgMTIuMTI3IDIyLjUgMTQgMjIuNSAxNGwtMjEgMy44MThzLS4zNjItMS43NDMtLjQ3LTQuNDE3bTIxLjk0LTMuOTljLTEwLjY1Ni45NzMtMjEuMzAyIDMuODE4LTIxLjk0IDMuOTlNMjMgMTlMMSAyMyIvPjwvc3ZnPg==";
                link
                    rel="stylesheet"
                    type="text/css"
                    href="https://cdn.jsdelivr.net/npm/tailwindcss@3.3.3/src/css/preflight.min.css";
                style { (style) }
            }
            (body)
        }
    }
}

async fn index() -> Markup {
    page(html! {
        h1 .text-lg { "Hello, World!" }
        p ."text-red-500" ."p-2" {
            "Consequatur accusamus itaque illo ut saepe corporis voluptatem. Aut provident quasi voluptatem. Sunt non
            fuga officiis fugit aliquam numquam hic. Voluptatem ratione magni dolor ut."
        }
    })
}

async fn games(State(pool): State<Pool<Postgres>>) -> Markup {
    let games = sqlx::query_as!(Game, "SELECT slug, name FROM game;")
        .fetch_all(&pool)
        .await;
    page(html! {
        h1 .text-xl .font-bold { "Games" }
        @match games {
            Ok(games) => {
                table {
                    thead {
                        tr {
                            th ."p-2" .border ."border-slate-300" ."dark:border-slate-600";
                            th ."p-2" .border ."border-slate-300" ."dark:border-slate-600" { "Name" }
                        }
                    }
                    tbody {
                        @for game in games {
                            tr {
                                td ."p-2" .border ."border-slate-300" ."dark:border-slate-600" {
                                    input
                                        type="checkbox"
                                        name="slugs"
                                        value=(game.slug)
                                        ."dark:bg-slate-900"
                                        ."dark:border-white";
                                }
                                td ."p-2" .border ."border-slate-300" ."dark:border-slate-600" {
                                    a href=(format!("/games/{}", game.slug)) ."hover:text-violet-500" { (game.name) }
                                }
                            }
                        }
                    }
                }
            },
            Err(_) => {
                p { "No games..." }
            }
        }
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/funicular".to_string()
        }))
        .await?;

    axum::Server::bind(&"0.0.0.0:1111".parse()?)
        .serve(
            Router::new()
                .route("/", get(index))
                .route("/games", get(games).with_state(pool))
                .into_make_service(),
        )
        .await?;

    Ok(())
}
