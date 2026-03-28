use sqlx::{Pool, Postgres};
use sqlx::postgres::PgPoolOptions;
use time::OffsetDateTime;

use std::sync::Arc;

use iced::{Element, Task};
use iced::widget::{text, button, row};


pub mod default_screen {
    use iced::widget::Column;

    use super::*;

    #[derive(Debug, Clone, Default)]
    pub struct DefaultScreenState {
        pub pool: Option<Arc<Pool<Postgres>>>,
        pub old_brews: Vec<OldSettings>
    }

    #[derive(Debug, Clone)]
    pub struct OldSettings {
        pub brew_id: Option<i32>,
        pub water_temp: Option<i32>,
        pub grind_size: String,
        pub coffee_weight: Option<i32>,
        pub water_weight: Option<i32>,
        pub notes: String,
        pub recipe_id: Option<i32>,
        pub timepoint: Option<OffsetDateTime>
    }

    impl OldSettings {
        pub fn new() -> Self {
            OldSettings { brew_id: None, water_temp: None, grind_size: "".to_string(), coffee_weight: None, water_weight: None, notes: "".to_string(), recipe_id: None, timepoint: None }
        }
    }

    #[derive(Debug, Clone)]
    pub enum DefaultScreenMessage {
        ChoseBrew(OldSettings)
    }

    

    impl DefaultScreenState {
        pub fn update(&mut self, message: DefaultScreenMessage) -> OldSettings {
            match message {
                DefaultScreenMessage::ChoseBrew(brew) => brew
            }
        }

        pub fn view(&self) -> Element<'_, DefaultScreenMessage> {
            println!("Viewing, vec has size {}", self.old_brews.len());
            let olds = Column::from_vec(
                self.old_brews.iter().map(|brew| 
                    button(text(format!("Brew with {}g of coffee", brew.coffee_weight.unwrap_or(0))))
                        .on_press(DefaultScreenMessage::ChoseBrew(brew.clone()))
                        .into()
                ).collect()
            );

            let new = button(text("New Brew"))
                .on_press(DefaultScreenMessage::ChoseBrew(OldSettings::new()));

            row!(
                olds, new
            ).into()
        }        
    }
}


#[derive(Debug, Clone)]
enum Screen {
    DefaultScreen(default_screen::DefaultScreenState),
}

impl Screen {
}

impl Default for Screen {
    fn default() -> Self {
        Screen::DefaultScreen(
            default_screen::DefaultScreenState::default()
        )
    }
}

#[derive(Debug, Default, Clone)]
struct State {
    pool: Option<Arc<Pool<Postgres>>>,
    screen: Screen
}

impl State {
    fn new() -> ( Self, Task<Message> ) {
        (
            State {
                pool: None,
                screen: Screen::DefaultScreen(
                    default_screen::DefaultScreenState {
                        pool: None,
                        old_brews: Vec::new()
                    }
                )
            },

            Task::perform(connect_db(), Message::Loaded)
        )
        
    }
}


async fn get_last_brews(pool: &Pool<Postgres>, number: i64) -> Vec<default_screen::OldSettings> {
    let data = sqlx::query!(
        "SELECT brew.id as brew_id, water_temp, grind_size, coffee_weight, water_weight, brew.notes as brew_notes, recipe.id as recipe_id, recipe.name as recipe_name, timepoint
        FROM brew
            LEFT JOIN recipe
            ON brew.recipe_id = recipe.id
        ORDER BY timepoint DESC
        LIMIT $1;",
        number
    )
        .fetch_all(pool)
        .await
        .unwrap();

    let mut old_settings = Vec::new();
    for setting in data {
        old_settings.push(default_screen::OldSettings {
            brew_id: Some(setting.brew_id),
            water_temp: Some(setting.water_temp),
            grind_size: setting.grind_size,
            coffee_weight: Some(setting.coffee_weight),
            water_weight: Some(setting.water_weight),
            notes: setting.brew_notes.unwrap_or(String::new()),
            recipe_id: Some(setting.recipe_id),
            timepoint: Some(setting.timepoint)
        });
    }

    println!("Getting last brews");

    old_settings
}



async fn connect_db() -> (Pool<Postgres>, Vec<default_screen::OldSettings>) {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgresql://localhost:5432/brewboarddb")
        .await
        .unwrap();

    let old_brews = get_last_brews(&pool, 3).await;
    (pool, old_brews)
}




enum Message {
    DefaultScreen(default_screen::DefaultScreenMessage),
    Loaded((Pool<Postgres>, Vec<default_screen::OldSettings>))
}

fn update(state: &mut State, message: Message) {
    match message {
        Message::DefaultScreen(message) => {
            if let Screen::DefaultScreen(default) = &mut state.screen {
                let res = default.update(message);
                println!("{:?}", res);
            }
        },
        Message::Loaded((pool, old_settings)) => {
            state.pool = Some(Arc::new(pool));
            if let Screen::DefaultScreen(default) = &mut state.screen {
                default.old_brews = old_settings;
                default.pool = state.pool.clone();
            }
        }
    }
}

fn view(state: &State) -> Element<'_, Message> {
    match &state.screen {
        Screen::DefaultScreen(default) => default.view().map(Message::DefaultScreen)
    }
}

//#[tokio::main]
fn main() -> iced::Result {
    iced::application(State::new, update, view).run()
}