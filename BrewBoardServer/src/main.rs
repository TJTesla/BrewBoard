use axum::extract::{Query};
use axum_extra::extract::{Form};
use axum::response::{Html, Redirect};
use axum::routing::{get};
use axum::Router;
use serde::Deserialize;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Thingy for debug printing I think?
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // build our application with some routes
    let app = Router::new()
        .route("/", get(get_root))
        .route("/pour_question", get(get_pour_question))
        .route("/manual_recipe", get(get_manual_recipe).post(accept_new_manual_recipe));

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
                    <br>
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
        <input type="submit" value="Speichern">
    </form>
</html>"#);

    Html(new_page)
}



#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
struct ManualRecipe {
    min: Vec<u32>,
    sec: Vec<u32>,
    target: Vec<u32>,
    note: Vec<String>
}

async fn accept_new_manual_recipe(Form(manual_recipe): Form<ManualRecipe>) -> Redirect {
    // Change by adding to database instead
    dbg!(manual_recipe);
    Redirect::to("/")
}
