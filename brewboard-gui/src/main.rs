use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use time::{OffsetDateTime, UtcOffset};

use std::collections::HashMap;
use std::sync::Arc;


use iced::time::{Duration};
use iced::{Element, Subscription, Task};

use crate::countdown_screen::CountdownScreenMessage;
use crate::default_screen::DefaultScreenState;
use crate::settings_screen::NewSettings;

pub mod brew_screen;
pub mod countdown_screen;
pub mod default_screen;
pub mod finish_screen;
pub mod settings_screen;

const OFFSET_TO_UTC: i8 = 1;


enum Message {
    DefaultScreen(default_screen::DefaultScreenMessage),
    SettingsScreen(settings_screen::SettingsScreenMessage),
    CountdownScreen(countdown_screen::CountdownScreenMessage),
    BrewScreen(brew_screen::BrewScreenMessage),
    FinishScreen(finish_screen::FinishScreenMessage),

    LoadDefaultScreen(Vec<default_screen::OldSettings>),
    LoadedDBConnection((Arc<Pool<Postgres>>, Vec<default_screen::OldSettings>)),
    FetchedRecipeNames(HashMap<i32, String>),
    FetchedRecipeDetails(brew_screen::Recipe),
    IdleMessage
}

#[derive(Debug, Clone)]
enum Screen {
    DefaultScreen(default_screen::DefaultScreenState),
    SettingsScreen(settings_screen::SettingsScreenState),
    CountdownScreen(countdown_screen::CountdownScreenState),
    BrewScreen(brew_screen::BrewScreenState),
    FinishScreen(finish_screen::FinishScreenState),
}

impl Default for Screen {
    fn default() -> Self {
        Screen::DefaultScreen(default_screen::DefaultScreenState::default())
    }
}

#[derive(Debug, Default, Clone)]
struct State {
    pool: Option<Arc<Pool<Postgres>>>,
    screen: Screen,
}

impl State {
    fn new() -> (Self, Task<Message>) {
        (
            State {
                pool: None,
                screen: Screen::DefaultScreen(default_screen::DefaultScreenState {
                    old_brews: Vec::new(),
                }),
            },
            Task::perform(connect_db(), Message::LoadedDBConnection),
        )
    }
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

async fn get_last_brews(
    pool: Arc<Pool<Postgres>>,
    number: i64,
) -> Vec<default_screen::OldSettings> {
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
            timepoint: Some(setting.timepoint),
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

async fn get_recipe_details(
    pool: Arc<Pool<Postgres>>,
    recipe_id: i32,
    water_weight: i32,
) -> brew_screen::Recipe {
    let data = sqlx::query!(
        "SELECT name, minutes, seconds, targets, notes
        FROM recipe
        WHERE id = $1",
        recipe_id
    )
    .fetch_one(pool.as_ref())
    .await
    .unwrap();

    brew_screen::Recipe::new(
        data.name,
        data.minutes,
        data.seconds,
        data.targets,
        data.notes,
        water_weight,
    )
}

async fn save_new_brew(pool: Arc<Pool<Postgres>>, settings: NewSettings) {
    let now_utc = OffsetDateTime::now_utc();
    let now_with_offset = OffsetDateTime::new_in_offset(now_utc.date(), now_utc.time(), UtcOffset::from_hms(OFFSET_TO_UTC, 0, 0).unwrap());

    let _ = sqlx::query!(
        "INSERT INTO brew (water_temp, grind_size, coffee_weight, water_weight, recipe_id, timepoint) VALUES (
            $1, $2, $3, $4, $5, $6
        )",
        settings.get_water_temp(),
        settings.get_grind_size(),
        settings.get_coffee_weight(),
        settings.get_water_weight(),
        settings.get_chosen_recipe().get_id(),
        now_with_offset
    )
    .execute(pool.as_ref()).await;
}


fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::DefaultScreen(message) => {
            if let Screen::DefaultScreen(default) = &mut state.screen {
                let res = default.update(message);

                // Change to settings screen
                state.screen = Screen::SettingsScreen(settings_screen::SettingsScreenState::new(
                    res.water_temp,
                    res.grind_size,
                    res.coffee_weight,
                    res.water_weight,
                    res.recipe_id,
                    res.recipe_name,
                ));
                return Task::perform(
                    get_recipe_names(state.pool.clone().unwrap()),
                    Message::FetchedRecipeNames,
                );
            }
            Task::none()
        }
        Message::SettingsScreen(message) => {
            if let Screen::SettingsScreen(settings) = &mut state.screen {
                let action = settings.update(message);

                match action {
                    settings_screen::Action::None => {
                        return Task::none();
                    }
                    settings_screen::Action::ReturnToDefault => {
                        return Task::perform(
                            get_last_brews(state.pool.clone().unwrap(), 3),
                            Message::LoadDefaultScreen,
                        );
                    }
                    settings_screen::Action::MoveToCountdown => {
                        let (recipe_id, water_weight) = (
                            settings.get_settings().get_chosen_recipe().get_id(),
                            settings.get_settings().get_water_weight(),
                        );
                        state.screen = Screen::CountdownScreen(
                            countdown_screen::CountdownScreenState::start_with(
                                3,
                                settings.get_settings(),
                            ),
                        );
                        return Task::perform(
                            get_recipe_details(
                                state.pool.clone().unwrap(),
                                recipe_id,
                                water_weight,
                            ),
                            Message::FetchedRecipeDetails,
                        );
                    }
                }
            }
            Task::none()
        }
        Message::CountdownScreen(message) => {
            if let Screen::CountdownScreen(countdown) = &mut state.screen {
                let action = countdown.update(message);

                match action {
                    countdown_screen::Action::None => {
                        return Task::none();
                    }
                    countdown_screen::Action::MoveToBrew => {
                        state.screen = Screen::BrewScreen(brew_screen::BrewScreenState::new(
                            countdown.get_recipe_cache(),
                            countdown.get_settings_cache(),
                        ));

                        return Task::none();
                    }
                }
            }
            Task::none()
        }
        Message::BrewScreen(message) => {
            if let Screen::BrewScreen(brew) = &mut state.screen {
                let action = brew.update(message);

                match action {
                    brew_screen::Action::None => {
                        return Task::none();
                    }
                    brew_screen::Action::Cancel => {
                        state.screen =
                            Screen::SettingsScreen(settings_screen::SettingsScreenState::new(
                                None,
                                String::new(),
                                None,
                                None,
                                None,
                                String::new(),
                            ));

                        return Task::perform(
                            get_recipe_names(state.pool.clone().unwrap()),
                            Message::FetchedRecipeNames,
                        );
                    }
                    brew_screen::Action::ToFinishScreen => {
                        let settings = brew.get_settings();
                        state.screen = Screen::FinishScreen(finish_screen::FinishScreenState::new(brew.get_cur_time(), settings.get_water_weight()));

                        return Task::perform(save_new_brew(state.pool.clone().unwrap(), settings), |_| Message::IdleMessage);
                    }
                }
            }
            Task::none()
        }
        Message::FinishScreen(_) => {
            if let Screen::FinishScreen(_) = &state.screen {
                // Currently the only message that is possible is the BackHome Message
                return Task::perform(
                    get_last_brews(state.pool.clone().unwrap(), 3),
                    Message::LoadDefaultScreen,
                );
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
        }
        Message::FetchedRecipeNames(names) => {
            if let Screen::SettingsScreen(settings) = &mut state.screen {
                settings.set_recipe_names(names);
            }
            Task::none()
        }
        Message::FetchedRecipeDetails(recipe) => {
            if let Screen::CountdownScreen(countdown) = &mut state.screen {
                countdown.set_recipe_cache(recipe);
            }
            Task::none()
        }
        Message::IdleMessage => {
            Task::none()
        }
    }
}

fn view(state: &State) -> Element<'_, Message> {
    match &state.screen {
        Screen::DefaultScreen(default) => default.view().map(Message::DefaultScreen),
        Screen::SettingsScreen(settings) => settings.view().map(Message::SettingsScreen),
        Screen::CountdownScreen(countdown) => countdown.view().map(Message::CountdownScreen),
        Screen::BrewScreen(brew) => brew.view().map(Message::BrewScreen),
        Screen::FinishScreen(finish) => finish.view().map(Message::FinishScreen),
    }
}

fn subscription(state: &State) -> Subscription<Message> {
    let (second_countdown, milli_countdown) = if let Screen::CountdownScreen(_) = state.screen {
        (
            iced::time::every(Duration::from_secs(1))
                .map(|_| Message::CountdownScreen(CountdownScreenMessage::CountDown)),
            iced::time::every(Duration::from_millis(1))
                .map(|_| Message::CountdownScreen(CountdownScreenMessage::FillProgressBar)),
        )
    } else {
        (Subscription::none(), Subscription::none())
    };

    let brew_stopwatch = if let Screen::BrewScreen(_) = state.screen {
        iced::time::every(Duration::from_secs(1))
            .map(|_| Message::BrewScreen(brew_screen::BrewScreenMessage::CountUp))
    } else {
        Subscription::none()
    };

    Subscription::batch(vec![second_countdown, milli_countdown, brew_stopwatch])
}

fn main() -> iced::Result {
    iced::application(State::new, update, view)
        .subscription(subscription)
        .run()
}
