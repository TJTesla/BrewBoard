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
        <head>
            <title>BrewBoard Manager</title>
            <style>
                body {
                    margin: 0;
                    font-family: Georgia;
                    display: flex;
                    justify-content: center;
                    align-items: center;
                    min-height: 30vh;
                    background: white;
                }

                .container {
                    background: #FAFAFA;
                    padding: 30px;
                    border-radius: 16px;
                    width: 80%;
                }

                .grid {
                    display: grid;
                    grid-template-columns: 1fr 1fr;
                    gap: 10px;
                    width: 100%;
                }

                .top {
                    grid-column: span 2;
                    padding: 30px;
                }

                button {
                    padding: 20px;
                    font-size: 24px;
                    font-family: "Georgia";
                    border: none;
                    border-radius: 8px;
                }

                .title {
                    text-align: center;
                    font-family: "Georgia";
                    font-size: 3rem;
                    font-weight: 700;
                    margin: 20px 0;

                    /* Nice spacing */
                    letter-spacing: 1px;
                }
            </style>
        </head>
        <html>
            <body>
                <div class="container">
                    <h1 class="title">BrewBoard Manager</h1>
                    <form>
                        <div class="grid">
                            <button type="submit" class="top" formaction="/pour_question">Manual Recipe</button>
                            <button type="submit" class="button" formaction="/recipe_list">List of Recipes</button>
                            <button type="submit" class="button" formaction="/brew_list">List of Brews</button>
                        </div>
                    </form>
                </div>
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
            <head>
                <title>How many pours?</title>
                <style>
                    body {
                        margin: 0;
                        font-family: Georgia;
                        height: 30vh;
                        display: flex;
                        align-items: center;
                        justify-content: center;
                    }

                    .container {
                        background: #FAFAFA;
                        padding: 40px;
                        border-radius: 16px;
                        text-align: center;
                        width: 60%;
                    }

                    h1 {
                        margin-bottom: 20px;
                        font-size: 2rem;
                    }

                    input[type="text"] {
                        width: 40%;
                        padding: 12px;
                        margin-bottom: 15px;
                        border: 1px solid #ccc;
                        border-radius: 8px;
                        font-size: 24px;
                        font-family: Georgia;
                        box-sizing: border-box;
                    }

                    button {
                        width: 20%;
                        padding: 12px;
                        border: none;
                        border-radius: 8px;
                        font-size: 24px;
                        font-family: Georgia;
                        cursor: pointer;
                    }
                </style>
            </head>
            <body>
                <div class="container">
                    <form action="/manual_recipe">
                        <h1>How many pours (including the bloom?)</h1>

                        
                        <input type="text" name="pour_number" autofill="off">
                        <button type="submit" value="Continue">Continue</button>
                    </form>
                </div>
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
        <head>
        <title>Create or Edit a Recipe</title>
        <style>
            body {{
                margin: 0;
                font-family: Georgia;
                display: flex;
                justify-content: center;
                align-items: center;
                min-height: 30vh;
                background: white;
            }}

            .container {{
                background: #FAFAFA;
                padding: 30px;
                border-radius: 16px;
                width: 80%;
            }}

            h1 {{
                text-align: center;
                margin-bottom: 20px;
                color: black;
            }}

            input[type="text"],input[type="number"] {{
                width: 100%;
                padding: 10px;
                border-radius: 8px;
                border: 1px solid #ccc;
                margin-bottom: 15px;
                box-sizing: border-box;
                font-family: Georgia;
                font-size: 24px;
            }}

            table {{
                width: 100%;
                border-collapse: collapse;
                margin-bottom: 20px;
                font-family: Georgia;
                font-size: 24px;
            }}



            th {{
                text-align: left;
                padding: 8px;
                color: black;
            }}

            td {{
                padding: 5px;
            }}

            td input {{
                width: 100%;
                padding: 8px;
                border-radius: 6px;
                border: 1px solid #ccc;
                box-sizing: border-box;
            }}

            button {{
                width: 100%;
                padding: 12px;
                border: none;
                border-radius: 8px;
                font-size: 24px;
                cursor: pointer;
                font-family: Georgia;
            }}

            .name_class {{
                text-align: center;
                font-family: Georgia;
                font-size: 24px;
            }}

            .name_class > input {{
                text-align: left;
            }}
        </style>
        </head>
        <body>
            <div class="container">
            <form>
                <h1>Define your new Recipe!</h1>
                {}
                <div class="name_class">
                    <b>Name:</b> 
                    <input type="text" name="name", value="{}" style="width: 50%">
                </div>
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
                <td><input type="text" name="notes" value="{}" size="50"></td>
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
        </div>
    </body
</html>"#, final_button_html));

    Html(new_page)
}


#[derive(Deserialize, Debug)]
struct PourQuestionInput {
    pour_number: String,
}

async fn get_manual_recipe(Query(q): Query<PourQuestionInput>) -> Html<String> {
    let pour_number_parsed: usize = q.pour_number.parse().unwrap_or(1);
    calculate_recipe_detail_html(None, pour_number_parsed, r#"<button type="submit" formmethod="post">Save</button>"#.to_string())
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
            <title>List of Recipes</title>
            <style>
                body {
                    margin: 0;
                    font-family: Georgia;
                    min-height: 30vh;
                    display: flex;
                    justify-content: center;
                    align-items: flex-start;
                    font-size: 24px;
                }

                .container {
                    background: #FAFAFA;
                    margin-top: 40px;
                    padding: 30px;
                    border-radius: 16px;
                    width: 80%;
                }

                h1 {
                    text-align: center;
                    margin-bottom: 20px;
                }

                .home-btn {
                    display: block;
                    margin-bottom: 20px;
                    text-align: center;
                    text-decoration: none;
                    padding: 10px;
                    border-radius: 8px;
                    background: #eeeeee;
                    font-weight: bold;
                    width: 30%;
                    border: none;
                    font-size: 24px;
                }

                .button-row {
                    display: flex;
                    width: 100%;
                    gap: 10px;
                }

                .button-row button {
                    flex: 1;
                    padding: 10px;
                    font-size: 24px;
                }

                .bottom-space {
                    margin-bottom: 20px;
                }

                button {
                    border: none;
                    border-radius: 8px;
                    font-size: 24px;
                    cursor: pointer;
                    font-family: Georgia;
                }
            </style>
        </head>
        <body>
            <div class="container">
                <h1>Overview of all Recipes</h1>
                <form action="/" method="get">
                    <button type="submit" style="width: 20%; padding: 10px;">Back Home</button>
                </form>
                <ul>
    "#.to_string();

    for recipe in recipes {
        list_page.push_str(&format!(
            r#"
            <li class="bottom-space">
                {}<br> 
                Brewcount: {}
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
            </div>
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
            <title>List of Brews</title>
            <style>
                body {
                    margin: 0;
                    font-family: Georgia;
                    min-height: 30vh;
                    display: flex;
                    justify-content: center;
                    align-items: flex-start;
                    font-size: 24px;
                }

                .container {
                    background: #FAFAFA;
                    margin-top: 40px;
                    padding: 30px;
                    border-radius: 16px;
                    width: 80%;
                }

                h1 {
                    text-align: center;
                    margin-bottom: 20px;
                }

                .home-btn {
                    display: block;
                    margin-bottom: 20px;
                    text-align: center;
                    text-decoration: none;
                    padding: 10px;
                    border-radius: 8px;
                    background: #eeeeee;
                    font-weight: bold;
                    width: 30%;
                    border: none;
                    font-size: 24px;
                }

                .button-row {
                    display: flex;
                    width: 100%;
                    gap: 10px;
                }

                .button-row button {
                    flex: 1;
                    padding: 10px;
                    font-size: 24px;
                }

                .bottom-space {
                    margin-bottom: 20px;
                    font-size: 24px;
                }

                button {
                    border: none;
                    border-radius: 8px;
                    font-size: 24px;
                    cursor: pointer;
                    font-family: Georgia;
                    padding: 10px;
                }

                select {
                    border: none;
                    border-radius: 8px;
                    font-size: 24px;
                    font-family: Georgia;
                    padding: 10px;
                }
            </style>
        </head>
    "#.to_string();

    list_page.push_str(&format!(r#"
        <body style="font-size: 100%;">
            <div class="container">
                <h1>Overview of all Brews in Chronological Order</h1>
                <form action="/brew_list/filtered" method="get">
                    <label for="recipes" style="font-size: 24px">Filter for a specific recipe:</label>
                    <select id="recipes" name="recipe_filter">
                        {}
                    </select>
                    <button type="submit" style="width: 15%">Filter</button>
                    {}
                    <button type="submit" style="width: 20%; padding: 10px;" formaction="/" formmethod="get">Back Home</button>
                </form>
                <ul>
    "#, filter_string, recipe_filter_id.map_or("", |_| r#"<button type="submit" style="width: 15%" formaction="/brew_list">Remove Filter</button>"#)));

    for brew in brews {
        let duration_after_brew = (OffsetDateTime::now_local().expect("Could not find current local time.") - brew.timepoint).whole_days();
        let time_str = timepoint_to_string(brew.timepoint);

        list_page.push_str(&format!(
            r#"
            <li class="bottom-space">
                Brew from {} ({} ago)<br>
                using {}
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
            time_str,
            if duration_after_brew == 1 { "1 day ".to_string() } else { format!("{} days", duration_after_brew) },
            &brew.name,
            &brew.brew_id, &brew.brew_id, &brew.brew_id
        ));
    }

    list_page.push_str(r#"
                </ul>
            </div>
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
        <head>
            <title>Recipe Description</title>
            <style>
            body {{
                margin: 0;
                font-family: Georgia;
                display: flex;
                justify-content: center;
                align-items: flex-start;
                min-height: 30vh;
                font-size: 24px;
            }}

            .container {{
                background: #FAFAFA;
                margin-top: 40px;
                padding: 30px;
                border-radius: 16px;
                width: 80%;
            }}  

            h1 {{
                text-align: center;
                margin-bottom: 25px;
            }}

            table {{
                width: 100%;
                border-collapse: collapse;
                overflow: hidden;
                border-radius: 10px;
            }}

            th, td {{
                padding: 12px;
                text-align: left;
            }}

            tbody tr:nth-child(even) {{
                background: #f9f9f9;
            }}

            tbody tr:hover {{
                background: #f1f1f1;
            }}

            th {{
                font-weight: 600;
            }}
        </style>
        </head>
        <body>
            <div class=container>
                <h1>{}</h1>
                <table>
                    <colgroup>
                        <col style="width: 5ch;">
                        <col style="width: 20ch;">
                        <col> <!-- remaining space -->
                    </colgroup>
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
            </div>
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
            <head>
            <title>Description of a Brew</title>
            <style>
            body {{
                margin: 0;
                font-family: Georgia;
                display: flex;
                justify-content: center;
                align-items: flex-start;
                min-height: 30vh;
                font-size: 24px;
            }}

            .container {{
                background: #FAFAFA;
                margin-top: 40px;
                padding: 30px;
                border-radius: 16px;
                width: 80%;
            }}  

            h1 {{
                text-align: center;
                margin-bottom: 25px;
            }}

            table {{
                width: 100%;
                border-collapse: collapse;
                overflow: hidden;
                border-radius: 10px;
            }}

            th, td {{
                padding: 12px;
                text-align: left;
            }}

            tbody tr:nth-child(even) {{
                background: #f9f9f9;
            }}

            tbody tr:hover {{
                background: #f1f1f1;
            }}

            th {{
                font-weight: 600;
            }}
        </style>
        </head>
            <body>
                <div class="container">
                <h1>Description of a Brew from {}</h1>
                    <table>
                        <colgroup>
                            <col style="width: 20ch;">
                            <col>
                        </colgroup>
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
                                        font-family: Georgia;
                                        font-size: 24px;
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
                </div>
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
            <head>
                <title>Edit Brew Notes</title>
                <style>
                    body {{
                        margin: 0;
                        font-family: Georgia;
                        height: 30vh;
                        display: flex;
                        align-items: center;
                        justify-content: center;
                    }}

                    .container {{
                        background: #FAFAFA;
                        padding: 40px;
                        border-radius: 16px;
                        text-align: center;
                        width: 60%;
                    }}

                    h1 {{
                        margin-bottom: 20px;
                        font-size: 2rem;
                    }}

                    input[type="text"] {{
                        width: 40%;
                        padding: 12px;
                        margin-bottom: 15px;
                        border: 1px solid #ccc;
                        border-radius: 8px;
                        font-size: 24px;
                        font-family: Georgia;
                        box-sizing: border-box;
                        }}

                    button {{
                        width: 20%;
                        padding: 12px;
                        border: none;
                        border-radius: 8px;
                        font-size: 24px;
                        font-family: Georgia;
                        cursor: pointer;
                    }}

                    textarea {{
                        padding: 12px;
                        border: none;
                        border-radius: 8px;
                        font-size: 24px;
                        font-family: Georgia;
                        width: 70%;
                    }}
                </style>
            </head>
            <body>
                <div class="container">
                    <h1>Edit your notes for your brew from {}</h1>
                    <form action="/brew_list/edit_notes/save" method="post">
                        <textarea name="notes" rows="4" cols="50">{}</textarea>
                        <input type="hidden" name="id" value="{}">
                        <br>
                        <br>
                        <button type="submit">Save your notes</button>
                    </form>
                </div>
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
    calculate_recipe_detail_html(Some(row.to_database_recipe(q.id)), row_num, r#"<button type="submit" formmethod="post" formaction="/recipe_list/save_edit">Save Edits</button>"#.to_string())
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
