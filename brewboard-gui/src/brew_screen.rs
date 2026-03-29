use super::settings_screen::NewSettings;
use std::cmp::Ord;
use std::fmt::Display;

use iced::Element;
use iced::widget::{Column, button, text};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SimpleTime {
    minute: i32,
    second: i32,
}

impl SimpleTime {
    pub fn new(minute: i32, second: i32) -> Self {
        SimpleTime { minute, second }
    }

    pub fn advance_by_secs(&mut self, seconds: i32) {
        let new_seconds = self.second + seconds;
        self.minute += new_seconds / 60;
        self.second = new_seconds % 60;
    }
}

impl From<(i32, i32)> for SimpleTime {
    fn from(value: (i32, i32)) -> Self {
        SimpleTime {
            minute: value.0,
            second: value.1,
        }
    }
}

impl Display for SimpleTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.minute, format!("{:0>2}", self.second))
    }
}

#[derive(Debug, Clone)]
pub struct Recipe {
    name: String,
    times: Vec<SimpleTime>,
    targets: Vec<i32>,
    notes: Vec<String>,
}

impl Recipe {
    pub fn new(
        name: String,
        minutes: Vec<i32>,
        seconds: Vec<i32>,
        fullsize_targets: Vec<i32>,
        notes: Vec<String>,
        water_weight: i32,
    ) -> Self {
        let times = minutes
            .into_iter()
            .zip(seconds.into_iter())
            .map(|t| t.into())
            .collect();
        let scale = water_weight as f32 / fullsize_targets[fullsize_targets.len() - 1] as f32;
        let targets = fullsize_targets
            .iter()
            .map(|t| ((*t as f32) * scale) as i32)
            .collect();

        Recipe {
            name,
            times,
            targets,
            notes,
        }
    }
}

#[derive(Debug, Clone)]
pub enum BrewScreenMessage {
    CountUp,
    Cancel,
    ShowNextPour,
    FinishBrew,
}

#[derive(Debug, Clone)]
pub enum Action {
    None,
    ToFinishScreen,
    Cancel,
}

#[derive(Debug, Clone)]
pub struct BrewScreenState {
    recipe: Recipe,
    settings: NewSettings,
    cur_time: SimpleTime,
    next_target_index: usize,
}

impl BrewScreenState {
    pub fn new(recipe: Recipe, settings: NewSettings) -> BrewScreenState {
        BrewScreenState {
            recipe,
            settings,
            cur_time: (0, 0).into(),
            next_target_index: 0,
        }
    }

    pub fn get_settings(&self) -> NewSettings {
        self.settings.clone()
    }

    pub fn get_cur_time(&self) -> SimpleTime {
        self.cur_time.clone()
    }

    pub fn update(&mut self, message: BrewScreenMessage) -> Action {
        match message {
            BrewScreenMessage::CountUp => {
                self.cur_time.advance_by_secs(1);

                if self.next_target_index < self.recipe.targets.len() {
                    if self.cur_time >= self.recipe.times[self.next_target_index] {
                        self.next_target_index += 1;
                    }
                }

                Action::None
            }
            BrewScreenMessage::ShowNextPour => {
                self.next_target_index =
                    std::cmp::min(self.next_target_index + 1, self.recipe.targets.len());

                Action::None
            }
            BrewScreenMessage::FinishBrew => Action::ToFinishScreen,
            BrewScreenMessage::Cancel => Action::Cancel,
        }
    }

    pub fn view(&self) -> Element<'_, BrewScreenMessage> {
        let mut column = Column::new();

        column = column.push(text(format!("Currently Brewing {}", self.recipe.name)));
        column = column.push(text("Weight: xxx.xg"));
        column = column.push(text(format!("Time: {}", self.cur_time)));

        let mut string = format!(
            "Bei {} auf {}g",
            self.recipe
                .times
                .get(self.next_target_index)
                .unwrap_or(&self.recipe.times[self.recipe.times.len() - 1]),
            self.recipe
                .targets
                .get(self.next_target_index)
                .unwrap_or(&self.recipe.targets[self.recipe.targets.len() - 1])
        );
        if !self
            .recipe
            .notes
            .get(self.next_target_index)
            .unwrap_or(&self.recipe.notes[self.recipe.notes.len() - 1])
            .is_empty()
        {
            string.push_str(&format!(
                " und {}",
                self.recipe
                    .notes
                    .get(self.next_target_index)
                    .unwrap_or(&self.recipe.notes[self.recipe.notes.len() - 1])
            ));
        }

        column = column.push(text(string));

        if self.next_target_index >= self.recipe.targets.len()-1 {
            // Finish Brew Button
            column =
                column.push(button(text("Finish Brew")).on_press(BrewScreenMessage::FinishBrew));
        } else {
            // Show next pour button
            column = column
                .push(button(text("Show Next Pour")).on_press(BrewScreenMessage::ShowNextPour));
        }

        column.into()
    }
}
