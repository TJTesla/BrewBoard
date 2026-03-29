use time::OffsetDateTime;

use iced::Element;
use iced::widget::{button, row, text};

use iced::widget::Column;

#[derive(Debug, Clone, Default)]
pub struct DefaultScreenState {
    pub old_brews: Vec<OldSettings>,
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
    pub recipe_name: String,
    pub timepoint: Option<OffsetDateTime>,
}

impl OldSettings {
    pub fn new() -> Self {
        OldSettings {
            brew_id: None,
            water_temp: None,
            grind_size: "".to_string(),
            coffee_weight: None,
            water_weight: None,
            notes: "".to_string(),
            recipe_id: None,
            recipe_name: String::new(),
            timepoint: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum DefaultScreenMessage {
    ChoseBrew(OldSettings),
}

impl DefaultScreenState {
    pub fn update(&mut self, message: DefaultScreenMessage) -> OldSettings {
        match message {
            DefaultScreenMessage::ChoseBrew(brew) => brew,
        }
    }

    pub fn view(&self) -> Element<'_, DefaultScreenMessage> {
        let olds = Column::from_vec(
            self.old_brews
                .iter()
                .map(|brew| {
                    button(text(format!(
                        "Brew with {}g of coffee",
                        brew.coffee_weight.unwrap_or(0)
                    )))
                    .on_press(DefaultScreenMessage::ChoseBrew(brew.clone()))
                    .into()
                })
                .collect(),
        );

        let new =
            button(text("New Brew")).on_press(DefaultScreenMessage::ChoseBrew(OldSettings::new()));

        row!(olds, new).into()
    }
}
