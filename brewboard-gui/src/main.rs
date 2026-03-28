use sqlx::{Pool, Postgres};
use sqlx::postgres::PgPoolOptions;

use std::collections::HashMap;
use std::sync::Arc;

use iced::{Element, Subscription, Task};
use iced::time::{self, Duration};

use crate::countdown_screen::CountdownScreenMessage;
use crate::default_screen::DefaultScreenState;


pub mod default_screen;
pub mod settings_screen;
pub mod countdown_screen;


enum Message {
    DefaultScreen(default_screen::DefaultScreenMessage),
    SettingsScreen(settings_screen::SettingsScreenMessage),
    CountdownScreen(countdown_screen::CountdownScreenMessage),

    LoadDefaultScreen(Vec<default_screen::OldSettings>),
    LoadedDBConnection((Arc<Pool<Postgres>>, Vec<default_screen::OldSettings>)),
    FetchedRecipeNames(HashMap<i32, String>)
}

#[derive(Debug, Clone)]
enum Screen {
    DefaultScreen(default_screen::DefaultScreenState),
    SettingsScreen(settings_screen::SettingsScreenState),
    CountdownScreen(countdown_screen::CountdownScreenState)
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
    fn new() -> ( Self, Task<Message> ) {(
        State {
            pool: None,
            screen: Screen::DefaultScreen( default_screen::DefaultScreenState { old_brews: Vec::new() } )
        },
        Task::perform(connect_db(), Message::LoadedDBConnection)
    )}
}



async fn connect_db() -> (Arc<Pool<Postgres>>, Vec<default_screen::OldSettings>) {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgresql://localhost:5432/brewboarddb")
        .await
        .unwrap();

    let pool = Arc::new(pool);

    let old_brews = get_last_brews(pool.clone(), 3).await;
    (pool, old_brews)
}

async fn get_last_brews(pool: Arc<Pool<Postgres>>, number: i64) -> Vec<default_screen::OldSettings> {
    let data = sqlx::query!(
        "SELECT brew.id as brew_id, water_temp, grind_size, coffee_weight, water_weight, brew.notes as brew_notes, recipe.id as recipe_id, recipe.name as recipe_name, timepoint
        FROM brew
            LEFT JOIN recipe
            ON brew.recipe_id = recipe.id
        ORDER BY timepoint DESC
        LIMIT $1;",
        number
    )
        .fetch_all(&*pool)
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
            recipe_name: setting.recipe_name,
            timepoint: Some(setting.timepoint)
        });
    }

    old_settings
}

async fn get_recipe_names(pool: Arc<Pool<Postgres>>) -> HashMap<i32, String> {
    let data = sqlx::query!(
        "SELECT id, name
        FROM recipe;"
    )
        .fetch_all(pool.as_ref())
        .await
        .unwrap();

    data.into_iter().map(|row| (row.id, row.name)).collect()
}





fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::DefaultScreen(message) => {
            if let Screen::DefaultScreen(default) = &mut state.screen {
                let res = default.update(message);
                
                // Change to settings screen
                state.screen = Screen::SettingsScreen(settings_screen::SettingsScreenState::new(res.water_temp, res.grind_size, res.coffee_weight, res.water_weight, res.recipe_id, res.recipe_name));
                return Task::perform(get_recipe_names(state.pool.clone().unwrap()), Message::FetchedRecipeNames);
            }
            Task::none()
        },
        Message::SettingsScreen(message) => {
            if let Screen::SettingsScreen(settings) = &mut state.screen {
                let action = settings.update(message);

                match action {
                    settings_screen::Action::None => { return Task::none(); },
                    settings_screen::Action::ReturnToDefault => {
                        return Task::perform(get_last_brews(state.pool.clone().unwrap(), 3), Message::LoadDefaultScreen);
                    },
                    settings_screen::Action::MoveToCountdown => {
                        // TODO
                        state.screen = Screen::CountdownScreen(countdown_screen::CountdownScreenState::start_with(3));
                        return Task::none();
                    }
                }
            }
            Task::none()
        },
        Message::CountdownScreen(message) => {
            if let Screen::CountdownScreen(countdown) = &mut state.screen {
                let action = countdown.update(message);

                match action {
                    countdown_screen::Action::None => { return Task::none(); },
                    countdown_screen::Action::MoveToBrew => {
                        // TODO
                        println!("GO GO GO");
                        return Task::none();
                    }
                }
            }
            Task::none()
        }
        Message::LoadDefaultScreen(old_brews) => {
            state.screen = Screen::DefaultScreen(DefaultScreenState { old_brews });
            Task::none()
        }
        Message::LoadedDBConnection((pool, old_settings)) => {
            state.pool = Some(pool);
            if let Screen::DefaultScreen(default) = &mut state.screen {
                default.old_brews = old_settings;
            }
            Task::none()
        },
        Message::FetchedRecipeNames(names) => {
            if let Screen::SettingsScreen(settings) = &mut state.screen {
                settings.set_recipe_names(names);
            }
            Task::none()
        }
    }
}

fn view(state: &State) -> Element<'_, Message> {
    match &state.screen {
        Screen::DefaultScreen(default) => default.view().map(Message::DefaultScreen),
        Screen::SettingsScreen(settings) => settings.view().map(Message::SettingsScreen),
        Screen::CountdownScreen(countdown) => countdown.view().map(Message::CountdownScreen)
    }
}


fn subscription(state: &State) -> Subscription<Message> {
    if let Screen::CountdownScreen(_) = state.screen {
        Subscription::batch(vec![
            time::every(Duration::from_secs(1)).map(|_| Message::CountdownScreen(CountdownScreenMessage::CountDown)),
            time::every(Duration::from_millis(1)).map(|_| Message::CountdownScreen(CountdownScreenMessage::FillProgressBar))
        ])
    } else {
        Subscription::none()
    }

} 



fn main() -> iced::Result {
    iced::application(State::new, update, view)
        .subscription(subscription)
        .run()
}