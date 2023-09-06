use axum::{extract::State, http::header, response::IntoResponse, routing::get, Router};
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
use const_format::formatcp;
use maud::{html, Markup, DOCTYPE};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::{error::Error, sync::Arc, time::Duration};

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

fn page(children: Markup, pre: Option<Markup>) -> Markup {
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
            body class="min-h-screen dark:text-white dark:bg-slate-900" {
                @if let Some(pre) = pre { (pre) }
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

const BUTTON: &str = "inline-flex items-center py-2 px-4 text-sm font-medium text-center bg-transparent rounded border hover:text-white focus:ring-4 focus:outline-none";
const BUTTON_PRIMARY: &str = formatcp!("{BUTTON} {}", " text-violet-600 border-violet-600 dark:text-violet-300 dark:border-violet-300 hover:bg-violet-500 focus:ring-violet-400 dark:hover:bg-violet-400 dark:focus:ring-violet-500");
const BUTTON_SUCCESS: &str = formatcp!("{BUTTON} {}", " text-green-600 border-green-600 dark:text-green-300 dark:border-green-300 hover:bg-green-500 focus:ring-green-400 dark:hover:bg-green-400 dark:focus:ring-green-500");
const BUTTON_ERROR: &str = formatcp!("{BUTTON} {}", " text-red-600 border-red-600 dark:text-red-300 dark:border-red-300 hover:bg-red-500 focus:ring-red-400 dark:hover:bg-red-400 dark:focus:ring-red-500");

const DIALOG: &str = "hidden z-10 justify-center items-center w-full h-full target:flex bg-black/50 backdrop-blur-sm";

async fn style() -> impl IntoResponse {
    (
        [
            (header::CACHE_CONTROL, "max-age=2592000"),
            (header::CONTENT_TYPE, "text/css"),
        ],
        STYLE,
    )
}

async fn index() -> impl IntoResponse {
    page(
        html! {
            h1 class="text-lg" { "Hello, World!" }
            p class="p-2 text-red-500" {
                "Consequatur accusamus itaque illo ut saepe corporis voluptatem. Aut provident quasi voluptatem. Sunt non
                fuga officiis fugit aliquam numquam hic. Voluptatem ratione magni dolor ut."
            }
        },
        None,
    )
}

async fn games(games: Result<Vec<Game>, sqlx::Error>) -> Markup {
    page(
        html! {
            h1 class="text-xl font-bold" { "Games" }
            form method="post" enctype="multipart/form-data" class="flex flex-col gap-4 justify-center items-center" {
                div class="relative overflow-x-auto shadow-md sm:rounded" {
                    table class="w-full" {
                        caption class="bg-white dark:bg-slate-800 p-3 space-x-2" {
                            a href="#add" class=(BUTTON_PRIMARY) { span class="w-4 h-4 i-tabler-plus"; }
                            button type="submit" name="submit" value="remove" class=(BUTTON_ERROR) {
                                span class="w-4 h-4 i-tabler-trash";
                            }
                        }
                        thead class="text-xs text-gray-700 uppercase bg-slate-50 dark:bg-slate-700 dark:text-gray-400" {
                            tr {
                                th class="p-3" { input type="checkbox" name="slugs_all" class="dark:bg-slate-900"; }
                                th class="px-6 py-3" { "Name" }
                            }
                        }
                        tbody {
                            @match games {
                                Ok(games) => {
                                    @for game in games {
                                        tr class="bg-white border-b last:border-0 dark:bg-slate-800 dark:border-slate-700" {
                                            td class="p-3" {
                                                input
                                                    type="checkbox"
                                                    name="slugs"
                                                    value=(game.slug)
                                                    class="dark:bg-slate-900";
                                            }
                                            td class="px-6 py-3" {
                                                a href=(format!("/games/{}", game.slug)) class="hover:text-violet-500" {
                                                    (game.name)
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
                    }
                }
            }
        },
        Some(html! {
            dialog id="add" class=(DIALOG) {
                div class="flex z-10 flex-col gap-4 p-4 max-w-sm rounded border dark:text-white dark:bg-slate-900" {
                    h2 class="text-xl" { "Add Game" }
                    form method="post" enctype="multipart/form-data" class="flex flex-col gap-4 justify-center" {
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
                            button type="submit" name="submit" value="add" class=(BUTTON_SUCCESS) { span class="w-4 h-4 i-tabler-check"; }
                            a href="#!" class=(BUTTON_PRIMARY) { span class="w-4 h-4 i-tabler-x"; }
                        }
                    }
                }
                a href="#!" class="fixed inset-0" {}
            }
        }),
    )
}

#[derive(TryFromMultipart)]
struct GamesPayload {
    submit: String,
    name: Option<String>,
    description: Option<String>,
    slugs_all: Option<bool>,
    slugs: Vec<String>,
}

async fn games_post(
    State(state): State<Arc<AppState>>,
    TypedMultipart(form): TypedMultipart<GamesPayload>,
) -> impl IntoResponse {
    match form.submit.as_str() {
        "add" => {
            sqlx::query!(
                "INSERT INTO game (name, description) VALUES ($1, $2);",
                form.name,
                form.description
            )
            .execute(&state.pool)
            .await
            .unwrap();
        }
        "remove" => {
            if form.slugs_all.is_some_and(|a| a) {
                sqlx::query!("DELETE FROM game;")
                    .execute(&state.pool)
                    .await
                    .unwrap();
            } else {
                sqlx::query!("DELETE FROM game WHERE slug = ANY($1);", &form.slugs)
                    .execute(&state.pool)
                    .await
                    .unwrap();
            }
        }
        _ => {}
    };

    games(
        sqlx::query_as!(Game, "SELECT slug, name FROM game;")
            .fetch_all(&state.pool)
            .await,
    )
    .await
}

async fn games_get(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    games(
        sqlx::query_as!(Game, "SELECT slug, name FROM game;")
            .fetch_all(&state.pool)
            .await,
    )
    .await
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
                .route("/games", get(games_get).post(games_post))
                .with_state(shared_state)
                .into_make_service(),
        )
        .await?;

    Ok(())
}
