use iced::Element;
use iced::widget::{column, progress_bar, text};
use super::settings_screen::NewSettings;

#[derive(Debug, Clone)]
pub enum CountdownScreenMessage {
    CountDown,
    FillProgressBar,
}

#[derive(Debug, Clone)]
pub enum Action {
    None,
    MoveToBrew,
}

#[derive(Debug, Clone)]
pub struct CountdownScreenState {
    value: i32,
    start_value: i32,
    progress_fill: i32,
    
    settings_cache: NewSettings,
    recipe_cache: Option<super::brew_screen::Recipe>
}

impl CountdownScreenState {
    pub fn start_with(value: i32, settings: NewSettings) -> Self {
        CountdownScreenState {
            value,
            start_value: value,
            progress_fill: 0,
            settings_cache: settings,
            recipe_cache: None,
        }
    }

    pub fn get_settings_cache(&self) -> NewSettings {
        self.settings_cache.clone()
    }

    pub fn set_recipe_cache(&mut self, recipe: super::brew_screen::Recipe) {
        self.recipe_cache = Some(recipe);
    }

    pub fn get_recipe_cache(&self) -> super::brew_screen::Recipe {
        // Unwrap because if the app couldn't get the recipe in 3(ish) seconds, then we won't get it in the next fractions of a second
        // Which is needed if we want to begin the brew
        self.recipe_cache.clone().unwrap()
    }

    pub fn update(&mut self, message: CountdownScreenMessage) -> Action {
        match message {
            CountdownScreenMessage::CountDown => {
                self.value -= 1;
                if self.value <= 0 {
                    Action::MoveToBrew
                } else {
                    Action::None
                }
            }
            CountdownScreenMessage::FillProgressBar => {
                self.progress_fill += 1;
                Action::None
            }
        }
    }

    pub fn view(&self) -> Element<'_, CountdownScreenMessage> {
        column![
            text(format!("{}", self.value)),
            progress_bar(
                0.0..=1.0,
                self.progress_fill as f32 / (self.start_value * 1000) as f32
            )
        ]
        .into()
    }
}
