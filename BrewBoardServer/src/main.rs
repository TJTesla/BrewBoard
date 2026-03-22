use axum::extract::{Query, Multipart, State};
use axum_extra::extract::{Form};
use axum::response::{Html, Redirect};
use axum::routing::{get};
use axum::Router;
use serde::Deserialize;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use sqlx::{Pool, Postgres};
use sqlx::postgres::PgPoolOptions;
use std::env;

#[derive(Clone, Debug)]
struct AppState {
    pool: Pool<Postgres>
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    // Thingy for debug printing I think?
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgresql://localhost:5432/brewboarddb").await.unwrap();

    let state = AppState { pool };

    // build our application with some routes
    let app = Router::new()
        .route("/", get(get_root))
        .route("/pour_question", get(get_pour_question))
        .route("/manual_recipe", get(get_manual_recipe).post(accept_new_manual_recipe))
        .route("/json_recipe", get(get_json_recipe).post(accept_json_recipe))
        .with_state(state);

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:1234")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    let _ = axum::serve(listener, app).await;
}

async fn get_root() -> Html<&'static str> {
    Html(
        r#"
        <!doctype html>
        <html>
            <body>
                <p>
                    <a href="/pour_question"><button class="button">Manual Recipe</button></a>
                </p>
                <p>
                    <a href="/json_recipe"><button class="button">JSON Recipe</button></a>
                </p>
                <p>
                    <a href="/recipe_list"><button class="button">List of Recipes</button></a>
                </p>
                <p>
                    <a href="/brew_list"><button class="button">List of Brews</button></a>
                </p>
            </body>
        </html>
        "#,
    )
}

async fn get_pour_question() -> Html<&'static str> {
    Html(
        r#"
        <!DOCTYPE html>
        <html>
            <body>
                <form action="/manual_recipe">
                    <label for="pour_number">
                        How many pours (including the bloom?)
                    </label>
                    <br>
                    <input type="text" name="pour_number">
                    <input type="submit" value="Continue">
                </form>
            </body>
        </html>
        "#,
    )
}

#[derive(Deserialize, Debug)]
struct PourQuestionInput {
    pour_number: u32,
}

async fn get_manual_recipe(Query(q): Query<PourQuestionInput>) -> Html<String> {
    let mut new_page = r#"
    <!DOCTYPE html>
    <html>
        <form method="POST">
            Name: <input type="text" name="name">
            <table>
                <tr>
                    <th>
                        <label>Minute</label>
                    </th>
                    <th>
                        <label>Second</label>
                    </th>
                    <th>
                        <label>Next Target</label>
                    </th>
                    <th>
                        <label>Notes</label>
                    </th>
                </tr>
    "#
    .to_string();
    let number = q.pour_number;
    for _ in 0..number {
        new_page.push_str(&format!(
            r#"
            <tr>
                <td><input type="number" name="min"></td>
                <td><input type="number" name="sec"></td>
                <td><input type="number" name="target"></td>
                <td><input type="text" name="note"></td>
            </tr>
            "#
        ));
    }
    new_page.push_str(r#"
            </table>
        <input type="submit" value="Save">
    </form>
</html>"#);

    Html(new_page)
}


async fn get_json_recipe() -> Html<&'static str> {
    Html(
        r#"
        <!DOCTYPE html>
        <html>
            <form method="POST" enctype="multipart/form-data">
                <input type="file" name="file" accept="application/json">
                <input type="submit" value="Save">
            </form>
        </html>
        "#
    )
}


#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
struct ManualRecipe {
    name: String,
    min: Vec<i32>,
    sec: Vec<i32>,
    target: Vec<i32>,
    note: Vec<String>
}

impl ManualRecipe {
    fn from_recipe(recipe: Recipe) -> ManualRecipe {
        let mut result = ManualRecipe {
            name: recipe.name,
            min: vec![], sec: vec![], target: vec![], note: vec![]
        };

        for pour in recipe.pours {
            result.min.push(pour.min);
            result.sec.push(pour.sec);
            result.target.push(pour.target);
            result.note.push(pour.note);
        }

        result
    }

    async fn insert_into_database(&self, state: &AppState) {
        let _ = sqlx::query!(
            "
            INSERT INTO recipe (name, minutes, seconds, targets, notes) VALUES (
                $1, $2, $3, $4, $5
            )",
            &self.name,
            &self.min,
            &self.sec,
            &self.target,
            &self.note
        )
        .execute(&state.pool).await;
    }
}

async fn accept_new_manual_recipe(State(state): State<AppState>, Form(manual_recipe): Form<ManualRecipe>) -> Redirect {
    manual_recipe.insert_into_database(&state).await;

    Redirect::to("/")
}

#[derive(Debug, Deserialize, Clone)]
struct Pour {
    min: i32,
    sec: i32,
    target: i32,
    note: String
}

#[derive(Debug, Deserialize, Clone)]
struct Recipe {
    name: String,
    pours: Vec<Pour>
}




async fn accept_json_recipe(State(state): State<AppState>, mut multipart: Multipart) -> Redirect {
    while let Some(field) = multipart.next_field().await.unwrap() {
        if field.name() == Some("file") {
            let data = field.bytes().await.unwrap();

            let parsed: Recipe = serde_json::from_slice(&data).unwrap();

            ManualRecipe::from_recipe(parsed).insert_into_database(&state).await;
        }
    }

    Redirect::to("/")
}