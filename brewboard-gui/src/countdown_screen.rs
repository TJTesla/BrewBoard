use iced::{Element};
use iced::widget::{column, progress_bar, text};

#[derive(Debug, Clone)]
pub enum CountdownScreenMessage {
    CountDown,
    FillProgressBar
}

#[derive(Debug, Clone)]
pub enum Action {
    None, MoveToBrew
}

#[derive(Debug, Clone)]
pub struct CountdownScreenState {
    value: i32,
    start_value: i32,
    progress_fill: i32,
}

impl CountdownScreenState {
    pub fn start_with(value: i32) -> Self {
        CountdownScreenState { value, start_value: value, progress_fill: 0 }
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
            },
            CountdownScreenMessage::FillProgressBar => {
                self.progress_fill += 1;
                Action::None
            }
        }
    }

    pub fn view(&self) -> Element<'_, CountdownScreenMessage> {
        column![
            text(format!("{}", self.value)),
            progress_bar(0.0..=1.0, self.progress_fill as f32 / (self.start_value * 1000) as f32)
        ].into()
    }
}