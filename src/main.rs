use axum::{
    extract::{Query, State},
    http::header,
    response::IntoResponse,
    routing::get,
    Router,
};
use const_format::formatcp;
use maud::{html, Markup, DOCTYPE};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::{collections::HashMap, error::Error, sync::Arc, time::Duration};

const STYLE: &str = include_str!("site.css");

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
                link rel="stylesheet" type="text/css" href="/site.css";
            }
            body class="dark:text-white dark:bg-slate-900" {
                nav class="py-4" {
                    ul class="flex flex-col gap-4 justify-center items-center sm:flex-row" {
                        @for link in &links {
                            li { a href=(link.href) class="hover:text-violet-500" { (link.children) } }
                        }
                    }
                }
                main class="container flex flex-col gap-4 justify-center items-center mx-auto" { (children) }
            }
        }
    }
}

const BUTTON: &str = "bg-transparent border font-medium focus:outline-none px-4 py-2 focus:ring-4 rounded text-sm text-center hover:text-white inline-flex";
const BUTTON_PRIMARY: &str = formatcp!("{BUTTON} {}", " hover:bg-violet-500 dark:hover:bg-violet-400 border-violet-600 dark:border-violet-300 focus:ring-violet-400 dark:focus:ring-violet-500 text-violet-600 dark:text-violet-300");
const BUTTON_SUCCESS: &str = formatcp!("{BUTTON} {}", " hover:bg-green-500 dark:hover:bg-green-400 border-green-600 dark:border-green-300 focus:ring-green-400 dark:focus:ring-green-500 text-green-600 dark:text-green-300");
const BUTTON_ERROR: &str = formatcp!("{BUTTON} {}", " hover:bg-red-500 dark:hover:bg-red-400 border-red-600 dark:border-red-300 focus:ring-red-400 dark:focus:ring-red-500 text-red-600 dark:text-red-300");

const TD: &str = "p-2 border border-slate-300 dark:border-slate-600";

const DIALOG: &str = "p-4 dark:bg-slate-900 dark:text-white rounded border sm:min-w-sm open:flex open:flex-col gap-4 fixed inset-0 z-20";

async fn style() -> impl IntoResponse {
    (
        [
            (header::CACHE_CONTROL, "max-age=2592000"),
            (header::CONTENT_TYPE, "text/css"),
        ],
        STYLE,
    )
}

async fn index() -> Markup {
    page(html! {
        h1 class="text-lg" { "Hello, World!" }
        p class="p-2 text-red" {
            "Consequatur accusamus itaque illo ut saepe corporis voluptatem. Aut provident quasi voluptatem. Sunt non
            fuga officiis fugit aliquam numquam hic. Voluptatem ratione magni dolor ut."
        }
    })
}

async fn games(
    Query(query): Query<HashMap<String, String>>,
    State(state): State<Arc<AppState>>,
) -> Markup {
    let games = sqlx::query_as!(Game, "SELECT slug, name FROM game;")
        .fetch_all(&state.pool)
        .await;
    page(html! {
        @if query.contains_key("add") {
            dialog open class=(DIALOG) {
                h2 class="text-xl"{ "Add Game" }
                form method="post" class="flex flex-col gap-4 justify-center" {
                    input
                      type="text"
                      name="name"
                      placeholder="Name"
                      required
                      autofocus
                      class="rounded invalid:border-red dark:bg-slate-900";
                    textarea
                      name="description"
                      placeholder="Description"
                      class="rounded invalid:border-red dark:bg-slate-900" {}
                    div class="flex justify-between" {
                        button type="submit" name="submit" value="add" class=(BUTTON_SUCCESS) { "Submit" }
                        a href="/games" class=(BUTTON_PRIMARY) { "Close" }
                    }
                }
            }
            a href="/games" class="fixed inset-0 z-10 bg-black/50 backdrop-blur-sm" {}
        }
        h1 class="text-xl font-bold" { "Games" }
        form method="post" class="flex flex-col gap-4 justify-center items-center" {
            div class="flex flex-row gap-2" {
                a href="/games?add" class=(BUTTON_PRIMARY) {
                    span class="w-4 h-4 i-tabler-plus";
                }
                button type="submit" name="submit" value="remove" class=(BUTTON_ERROR) { "Remove" }
            }
            @match games {
                Ok(games) => {
                    table {
                        thead {
                            tr {
                                th class=(TD);
                                th class=(TD) { "Name" }
                            }
                        }
                        tbody {
                            @for game in games {
                                tr {
                                    td class=(TD) {
                                        input type="checkbox" name="slugs" value=(game.slug) class="dark:bg-slate-900";
                                    }
                                    td class=(TD) {
                                        a href=(format!("/games/{}", game.slug)) class="hover:text-violet-500" {
                                            (game.name)
                                        }
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
        }
    })
}

struct AppState {
    pool: Pool<Postgres>,
}

impl AppState {
    fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let shared_state = Arc::new(AppState::new(
        PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(3))
            .connect(&std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://postgres:postgres@localhost:5432/funicular".to_string()
            }))
            .await?,
    ));

    axum::Server::bind(&"0.0.0.0:1111".parse()?)
        .serve(
            Router::new()
                .route("/site.css", get(style))
                .route("/", get(index))
                .route("/games", get(games).post(games))
                .with_state(shared_state)
                .into_make_service(),
        )
        .await?;

    Ok(())
}
