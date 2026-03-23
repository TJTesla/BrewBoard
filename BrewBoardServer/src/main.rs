use axum::extract::{Query, State};
use axum_extra::extract::{Form};
use axum::response::{Html, Redirect};
use axum::routing::{get, post};
use axum::Router;
use serde::Deserialize;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use sqlx::{Pool, Postgres};
use sqlx::postgres::PgPoolOptions;
use std::env;
use time::OffsetDateTime;


#[derive(Clone, Debug)]
struct AppState {
    pool: Pool<Postgres>,
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
        .route("/recipe_list", get(get_recipe_list))
        .route("/recipe_list/descr", post(accept_recipe_descr))
        .route("/recipe_list/edit", post(accept_recipe_edit))
        .route("/recipe_list/save_edit", post(accept_recipe_edit_save))
        .route("/recipe_list/delete", post(accept_recipe_delete))
        .route("/brew_list", get(get_brew_list))
        .route("/brew_list/descr", post(accept_brew_descr))
        .route("/brew_list/edit_notes", post(accept_brew_notes_edit))
        .route("/brew_list/edit_notes/save", post(accept_brew_notes_edit_save))
        .route("/brew_list/delete", post(accept_brew_delete))
        .route("/brew_list/filtered", get(get_filtered_brew_list))
        .with_state(state);

    // run it
    let listener = tokio::net::TcpListener::bind("0.0.0.0:1234")
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


#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
struct ManualRecipe {
    name: String,
    minutes: Vec<i32>,
    seconds: Vec<i32>,
    targets: Vec<i32>,
    notes: Vec<String>
}

impl ManualRecipe {
    async fn insert_into_database(&self, state: &AppState) {
        let _ = sqlx::query!(
            "
            INSERT INTO recipe (name, minutes, seconds, targets, notes) VALUES (
                $1, $2, $3, $4, $5
            )",
            &self.name,
            &self.minutes,
            &self.seconds,
            &self.targets,
            &self.notes
        )
        .execute(&state.pool).await;
    }

    fn to_database_recipe(self, id: i32) -> DatabaseRecipe {
        DatabaseRecipe { id, name: self.name, minutes: self.minutes, seconds: self.seconds, targets: self.targets, notes: self.notes }
    }
}

#[derive(Debug, Deserialize)]
struct DatabaseRecipe {
    id: i32,
    name: String,
    minutes: Vec<i32>,
    seconds: Vec<i32>,
    targets: Vec<i32>,
    notes: Vec<String>
}

#[derive(Debug)]
struct RecipeListResult {
    id: i32, name: String, count: Option<i64>
}

fn calculate_recipe_detail_html(data: Option<DatabaseRecipe>, pour_number: usize, final_button_html: String) -> Html<String> {
    let mut new_page = format!(r#"
    <!DOCTYPE html>
    <html>
        <form>
            {}
            Name: <input type="text" name="name", value={}>
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
    "#, 
    data.as_ref().map_or("".to_string(), | recipe | format!(r#"<input type="hidden" name="id" value={}>"#, recipe.id)),
    data.as_ref().map_or("".to_string(), | recipe | recipe.name.clone()));
    
    for i in 0..pour_number as usize {
        new_page.push_str(&format!(
            r#"
            <tr>
                <td><input type="number" name="minutes" value="{}"></td>
                <td><input type="number" name="seconds" value="{}"></td>
                <td><input type="number" name="targets" value="{}"></td>
                <td><input type="text" name="notes" value="{}"></td>
            </tr>
            "#,
            data.as_ref().map_or("".to_string(), | recipe | recipe.minutes[i].to_string()),
            data.as_ref().map_or("".to_string(), | recipe | recipe.seconds[i].to_string()),
            data.as_ref().map_or("".to_string(), | recipe | recipe.targets[i].to_string()),
            data.as_ref().map_or("".to_string(), | recipe | recipe.notes[i].clone())
        ));
    }
    new_page.push_str(&format!(r#"
            </table>
        {}
    </form>
</html>"#, final_button_html));

    Html(new_page)
}


#[derive(Deserialize, Debug)]
struct PourQuestionInput {
    pour_number: usize,
}

async fn get_manual_recipe(Query(q): Query<PourQuestionInput>) -> Html<String> {
    calculate_recipe_detail_html(None, q.pour_number, r#"<input type="submit" value="Save" formmethod="post">"#.to_string())
}



async fn get_recipe_list(State(state): State<AppState>) -> Html<String> {
    let recipes: Vec<RecipeListResult> = sqlx::query_as!( RecipeListResult,
        "SELECT recipe.id, name, count 
        FROM recipe 
            LEFT JOIN (
                SELECT recipe_id, COUNT(*) AS count
                FROM brew
                GROUP BY recipe_id
            ) as brewcounts 
        ON brewcounts.recipe_id = recipe.id
        ORDER BY brewcounts.count DESC NULLS LAST;"
    ).fetch_all(&state.pool).await.unwrap();

    let mut list_page = r#"
    <!DOCTYPE html>
    <html>
        <head>
            <style>
                .button-row {
                    display: flex;
                    width: 100%;
                    gap: 10px;
                }

                .button-row button {
                    flex: 1;
                    padding: 10px;
                    font-size: 16px;
                }

                .bottom-space {
                    margin-bottom: 20px;
                }
            </style>
        </head>
        <body style="font-size: 100%;">
            <ul>
    "#.to_string();

    for recipe in recipes {
        list_page.push_str(&format!(
            r#"
            <li class="bottom-space">
                {}, Brewcount: {}
                <form>
                    <div class="button-row">
                        <button style="flex:1; gap: 20p;" type="submit" formaction="/recipe_list/descr?id={}" formmethod="post">
                            Description
                        </button>
                        <button style="flex:1; gap: 20p;" type="submit" formaction="/recipe_list/edit?id={}" formmethod="post">
                            Edit
                        </button>
                        <button style="flex:1; gap: 20p;" type="submit" formaction="/recipe_list/delete?id={}" formmethod="post" onclick="return confirm('Are you sure you want to delete this recipe?');">
                            Delete
                        </button>
                    </div>
                </form>
            </li>
            "#,
            &recipe.name,
            &recipe.count.unwrap_or(0),
            &recipe.id, &recipe.id, &recipe.id
        ));
    }

    list_page.push_str(r#"
            </ul>
            <a href="/">Back Home...</a>
        </body>
    </html>
    "#);

    Html(list_page)
}


fn timepoint_to_string(timepoint: OffsetDateTime) -> String {
    let format = time::format_description::parse("[day].[month].[year repr:last_two], [hour]:[second]").expect("Could not parse time format");
    timepoint.format(&format).expect("Error while trying to format time point")
}

struct BrewListResult {
    brew_id: i32, name: String, timepoint: OffsetDateTime
}

async fn brew_list_string(state: &AppState, recipe_filter_id: Option<i32>) -> String {
    let brews: Vec<BrewListResult> = if let Some(recipe_filter) = recipe_filter_id {
        sqlx::query_as!( BrewListResult,
            "SELECT brew.id as brew_id, recipe.name, timepoint
            FROM brew
                LEFT JOIN recipe
                ON brew.recipe_id = recipe.id
            WHERE recipe.id = $1
            ORDER BY timepoint DESC;",
            recipe_filter
        ).fetch_all(&state.pool).await.unwrap() 
    } else { 
        sqlx::query_as!( BrewListResult,
            "SELECT brew.id as brew_id, recipe.name, timepoint
            FROM brew
                LEFT JOIN recipe
                ON brew.recipe_id = recipe.id
            ORDER BY timepoint DESC;"
        ).fetch_all(&state.pool).await.unwrap() 
    };

    let recipes: Vec<RecipeListResult> = sqlx::query_as!( RecipeListResult,
        "SELECT recipe.id, name, count 
        FROM recipe 
            LEFT JOIN (
                SELECT recipe_id, COUNT(*) AS count
                FROM brew
                GROUP BY recipe_id
            ) as brewcounts 
        ON brewcounts.recipe_id = recipe.id
        ORDER BY brewcounts.count DESC NULLS LAST;"
    ).fetch_all(&state.pool).await.unwrap();

    let mut filter_string= "".to_string();
    for recipe in recipes {
        filter_string.push_str(&format!(r#"<option value="{}">{}</option>{}"#, recipe.id, recipe.name, "\n"));
    }

    let mut list_page = r#"
    <!DOCTYPE html>
    <html>
        <head>
            <style>
                .button-row {
                    display: flex;
                    width: 100%;
                    gap: 10px;
                }

                .button-row button {
                    flex: 1;
                    padding: 10px;
                    font-size: 16px;
                }

                .bottom-space {
                    margin-bottom: 20px;
                }
            </style>
        </head>
    "#.to_string();

    list_page.push_str(&format!(r#"
        <body style="font-size: 100%;">
            <h2>List of all brews in chronological order</h2>
            <form action="/brew_list/filtered" method="get">
                <label for="recipes">Filter for a specific recipe:</label>
                <select id="recipes" name="recipe_filter">
                    {}
                </select>
                <button type="submit">Filter</button>
            </form>
            <a href="/">Back Home...</a>
            {}
            <ul>
    "#, filter_string, recipe_filter_id.map_or("", |_| r#"<br> <a href="/brew_list">Remove Filter</a>"#)));

    for brew in brews {
        let time_str = timepoint_to_string(brew.timepoint);

        list_page.push_str(&format!(
            r#"
            <li class="bottom-space">
                Brew from {} using {}
                <form>
                    <div class="button-row">
                        <button style="flex:1; gap: 20p;" type="submit" formaction="/brew_list/descr?id={}" formmethod="post">
                            Brew Description
                        </button>
                        <button style="flex:1; gap: 20p;" type="submit" formaction="/brew_list/edit_notes?id={}" formmethod="post">
                            Edit Notes
                        </button>
                        <button style="flex:1; gap: 20p;" type="submit" formaction="/brew_list/delete?id={}" formmethod="post" onclick="return confirm('Are you sure you want to delete this recipe?');">
                            Delete
                        </button>
                    </div>
                </form>
            </li>
            "#,
            time_str, &brew.name,
            &brew.brew_id, &brew.brew_id, &brew.brew_id
        ));
    }

    list_page.push_str(r#"
            </ul>
        </body>
    </html>
    "#);

    list_page
}

async fn get_brew_list(State(state): State<AppState>) -> Html<String> {
    Html(brew_list_string(&state, None).await)
}

#[derive(Debug, Deserialize)]
struct FilterFormResult {
    recipe_filter: i32
}

async fn get_filtered_brew_list(State(state): State<AppState>, Form(form): Form<FilterFormResult>) -> Html<String> {
    Html(brew_list_string(&state, Some(form.recipe_filter)).await)
}


#[derive(Debug, Clone, Deserialize)]
struct IdQuery {
    id: i32
}

async fn accept_recipe_descr(State(state): State<AppState>, Query(q): Query<IdQuery>) -> Html<String> {
    let row: ManualRecipe = sqlx::query_as!(ManualRecipe, "SELECT name, minutes, seconds, targets, notes FROM recipe WHERE id=$1;", q.id)
            .fetch_one(&state.pool).await.expect(&format!("Error while trying to find index {} of a recipe", q.id));

    let mut page = format!(r#"
    <!DOCTYPE html>
    <html>
        <body>
            <h1>{}</h1>
            <table>
                <tr>
                    <th>Time</th>
                    <th>Next Pour Target</th>
                    <th>Notes</th>
                </tr>
    "#, row.name);

    for i in 0..row.minutes.len() {
        page.push_str(&format!(r#"
            <tr>
                <td>{}:{}</td>
                <td>{}ml</td>
                <td>{}</td>
            </tr>
            "#,
            row.minutes[i], format!("{:0>2}", row.seconds[i]),
            row.targets[i], row.notes[i]
        ));
    }

    page.push_str(r#"
            </table>
        </body>
    </html>
    "#);

    Html(page)
}

struct BrewDescriptionResult {
    water_temp: i32,
    grind_size: String,
    coffee_weight: i32,
    water_weight: i32,
    brew_notes: Option<String>,
    timepoint: OffsetDateTime,
    recipe_id: i32,
    recipe_name: String
}

async fn accept_brew_descr(State(state): State<AppState>, Query(q): Query<IdQuery>) -> Html<String> {
    let brew: BrewDescriptionResult = sqlx::query_as!( BrewDescriptionResult,
        "SELECT water_temp, grind_size, coffee_weight, water_weight, brew.notes AS brew_notes, timepoint, recipe.id AS recipe_id, recipe.name AS recipe_name
        FROM brew
            LEFT JOIN recipe
            ON brew.recipe_id = recipe.id
        WHERE brew.id = $1
        ORDER BY timepoint DESC;",
        q.id
    ).fetch_one(&state.pool).await.expect("Could not find brew with the given id");

    Html(format!(r#"
        <!DOCTYPE html>
        <html>
            <body>
                Description of a Brew from {}
                <table>
                    <tr>
                        <td>Recipe</td>
                        <td>
                            <form method="post" action="/recipe_list/descr?id={}" style="display:inline;">
                                <button type="submit" style="
                                    background:none;
                                    border:none;
                                    color:blue;
                                    text-decoration:underline;
                                    cursor:pointer;
                                    padding:0;
                                ">
                                {}
                                </button>
                            </form>
                        </td>
                    </tr>
                    <tr>
                        <td>Water Temperature</td>
                        <td>{}°C</td>
                    </tr>
                    <tr>
                        <td>Grind Size</td>
                        <td>{}</td>
                    </tr>
                    <tr>
                        <td>Weight in Water</td>
                        <td>{}ml</td>
                    </tr>
                    <tr>
                        <td>Weight in Coffee</td>
                        <td>{}g</td>
                    </tr>
                    {}
                </table>
            </body>
        </html>"#,
        timepoint_to_string(brew.timepoint),
        brew.recipe_id, brew.recipe_name,
        brew.water_temp,
        brew.grind_size,
        brew.water_weight,
        brew.coffee_weight,
        brew.brew_notes.map_or("".to_string(), | notes | format!(r#"
            <tr>
                <td>Notes</td>
                <td>{}</td>
            </tr>
        "#, notes))
    ))
}

async fn accept_brew_notes_edit(State(state): State<AppState>, Query(q): Query<IdQuery>) -> Html<String> {
    let brew = sqlx::query!("SELECT timepoint, notes FROM brew WHERE id=$1", q.id).fetch_one(&state.pool).await.expect("Could not find brew with the given id");

    Html(format!(
        r#"
        <!DOCTYPE html>
        <html>
            <body>
                <h3>Edit your notes for your brew from {}</h3>
                <form action="/brew_list/edit_notes/save" method="post">
                    <textarea name="notes" rows="4" cols="50">{}</textarea>
                    <input type="hidden" name="id" value="{}">
                    <br>
                    <button type="submit">Save your notes</button>
                </form>
            </body>
        </html>
        "#,
        timepoint_to_string(brew.timepoint), brew.notes.unwrap_or("".to_string()), q.id
    ))
}



async fn accept_recipe_edit(State(state): State<AppState>, Query(q): Query<IdQuery>) -> Html<String> {
    let row: ManualRecipe = sqlx::query_as!(ManualRecipe, "SELECT name, minutes, seconds, targets, notes FROM recipe WHERE id=$1;", q.id)
            .fetch_one(&state.pool).await.expect(&format!("Error while trying to find index {} of a recipe", q.id));
    let row_num = row.minutes.len();
    calculate_recipe_detail_html(Some(row.to_database_recipe(q.id)), row_num, r#"<input type="submit" formmethod="post" formaction="/recipe_list/save_edit" value="Save Edits">"#.to_string())
}

async fn accept_recipe_edit_save(State(state): State<AppState>, Form(form): Form<DatabaseRecipe>) -> Redirect {
    let _ = sqlx::query!(
        "UPDATE recipe
        SET name = $1, minutes = $2, seconds = $3, targets = $4, notes = $5
        WHERE id = $6",
        &form.name, &form.minutes, &form.seconds, &form.targets, &form.notes, &form.id    
    ).execute(&state.pool).await;

    Redirect::to("/recipe_list")
}

#[derive(Debug, Deserialize)]
struct BrewNoteEditSaveForm {
    id: i32,
    notes: String
}

async fn accept_brew_notes_edit_save(State(state): State<AppState>, Form(form): Form<BrewNoteEditSaveForm>) -> Redirect {
    let _ = sqlx::query!(
        "UPDATE brew
        SET notes = $1
        WHERE id = $2",
        &form.notes, &form.id
    ).execute(&state.pool).await;

    Redirect::to("/brew_list")
}

async fn accept_new_manual_recipe(State(state): State<AppState>, Form(manual_recipe): Form<ManualRecipe>) -> Redirect {
    manual_recipe.insert_into_database(&state).await;

    Redirect::to("/")
}

async fn accept_recipe_delete(State(state): State<AppState>, Query(q): Query<IdQuery>) -> Redirect {
    let _ = sqlx::query!(
        "DELETE FROM recipe
        WHERE id=$1",
        q.id
    ).execute(&state.pool).await;

    Redirect::to("/recipe_list")
}

async fn accept_brew_delete(State(state): State<AppState>, Query(q): Query<IdQuery>) -> Redirect {
    let _ = sqlx::query!("DELETE FROM brew WHERE id=$1", q.id).execute(&state.pool).await;

    Redirect::to("/brew_list")
}


