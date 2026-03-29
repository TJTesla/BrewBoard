use crate::brew_screen;

use iced::Element;
use iced::widget::{button, column, text};

#[derive(Debug, Clone)]
pub enum FinishScreenMessage {
    BackHome,
}

#[derive(Debug, Clone)]
pub struct FinishScreenState {
    finish_time: brew_screen::SimpleTime,
    finish_weight: i32,
}

impl FinishScreenState {
    pub fn new(finish_time: brew_screen::SimpleTime, finish_weight: i32) -> Self {
        FinishScreenState {
            finish_time,
            finish_weight,
        }
    }

    pub fn view(&self) -> Element<'_, FinishScreenMessage> {
        column![
            text(format!(
                "Finished a brew with {}ml of water in {}",
                self.finish_weight, self.finish_time
            )),
            button(text("Home")).on_press(FinishScreenMessage::BackHome)
        ]
        .into()
    }
}
