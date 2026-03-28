use std::collections::HashMap;
use std::fmt::Display;

use iced::{Element};
use iced::widget::{button, pick_list, row, text, text_input, column, scrollable};

#[derive(Debug, Clone)]
pub enum SettingsScreenMessage {
    WaterTempChange(i32),
    GrindSizeChange(String),
    CoffeeWeightChange(i32),
    WaterWeightChange(i32),
    RecipeChosen(ChoosableRecipe),
    RecipeStart,
    BackToDefault
}

#[derive(Debug, Clone)]
pub struct ChoosableRecipe {
    id: i32,
    name: String
}

impl PartialEq for ChoosableRecipe {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for ChoosableRecipe {}

impl PartialOrd for ChoosableRecipe {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl Display for ChoosableRecipe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl From<(i32, String)> for ChoosableRecipe {
    fn from(value: (i32, String)) -> Self {
        ChoosableRecipe { id: value.0, name: value.1 }
    }
}


pub enum Action {
    None, ReturnToDefault, MoveToCountdown
}


#[derive(Debug, Clone)]
pub struct SettingsScreenState {
    water_temp: i32,
    grind_size: String,
    coffee_weight: i32,
    water_weight: i32,
    recipe_id: Option<ChoosableRecipe>,

    recipe_names: Vec<ChoosableRecipe>
}

impl SettingsScreenState {
    const WATER_TEMP_DEFAULT: i32 = 94;
    const COFFEE_WEIGHT_DEFAULT: i32 = 18;
    const WATER_WEIGHT_DEFAULT: i32 = 300;

    pub fn new(water_temp: Option<i32>, grind_size: String, coffee_weight: Option<i32>, water_weight: Option<i32>, recipe_id: Option<i32>, recipe_name: String) -> SettingsScreenState {
        SettingsScreenState { 
            water_temp: water_temp.unwrap_or(SettingsScreenState::WATER_TEMP_DEFAULT),
            grind_size: grind_size, 
            coffee_weight: coffee_weight.unwrap_or(SettingsScreenState::COFFEE_WEIGHT_DEFAULT), 
            water_weight: water_weight.unwrap_or(SettingsScreenState::WATER_WEIGHT_DEFAULT), 
            recipe_id: recipe_id.map_or(None, |id| Some((id, recipe_name).into())), 
            recipe_names: Vec::new() 
        }
    }

    pub fn set_recipe_names(&mut self, names: HashMap<i32, String>) {
        self.recipe_names = names.into_iter().map(|tuple| tuple.into()).collect();
    }

    pub fn update(&mut self, message: SettingsScreenMessage) -> Action {
        match message {
            SettingsScreenMessage::WaterTempChange(num) => self.water_temp += num,
            SettingsScreenMessage::GrindSizeChange(size) => self.grind_size = size,
            SettingsScreenMessage::CoffeeWeightChange(num) => self.coffee_weight += num,
            SettingsScreenMessage::WaterWeightChange(num) => self.water_weight += num,
            SettingsScreenMessage::RecipeChosen(choosable) => self.recipe_id = Some(choosable),
            SettingsScreenMessage::RecipeStart => {
                return Action::MoveToCountdown;
            },
            SettingsScreenMessage::BackToDefault => {
                return Action::ReturnToDefault;
            }
        }

        Action::None
    }

    pub fn view(&self) -> Element<'_, SettingsScreenMessage> {
        let water_weight_row = row![
            text("Water Weight"),
            button(text("-10g"))
                .on_press(SettingsScreenMessage::WaterWeightChange(-10)),
            button(text("-1g"))
                .on_press(SettingsScreenMessage::WaterWeightChange(-1)),
            text(format!("{}g", self.water_weight)),
            button(text("+1g"))
                .on_press(SettingsScreenMessage::WaterWeightChange(1)),
            button(text("+10g"))
                .on_press(SettingsScreenMessage::WaterWeightChange(10)),
        ];

        let coffee_weight_row = row![
            text("Coffee Weight"),
            button(text("-5g"))
                .on_press(SettingsScreenMessage::CoffeeWeightChange(-5)),
            button(text("-1g"))
                .on_press(SettingsScreenMessage::CoffeeWeightChange(-1)),
            text(format!("{}g", self.coffee_weight)),
            button(text("+1g"))
                .on_press(SettingsScreenMessage::CoffeeWeightChange(1)),
            button(text("+5g"))
                .on_press(SettingsScreenMessage::CoffeeWeightChange(5)),
        ];

        let water_temp_row = row![
            text("Water Temperature"),
            button(text("-10°C"))
                .on_press(SettingsScreenMessage::WaterTempChange(-10)),
            button(text("-1°C"))
                .on_press(SettingsScreenMessage::WaterTempChange(-1)),
            text(format!("{}°C", self.water_temp)),
            button(text("+1°C"))
                .on_press(SettingsScreenMessage::WaterTempChange(1)),
            button(text("+10°C"))
                .on_press(SettingsScreenMessage::WaterTempChange(10)),
        ];

        let grind_size_row = row![
            text("Grind Size"),
            text_input("", &self.grind_size)
                .id("grind-size-input")
                .on_input(SettingsScreenMessage::GrindSizeChange)
        ];

        let recipe_choosing_row = row![
            text("Choose a recipe"),
            pick_list(self.recipe_names.clone(), self.recipe_id.clone(), SettingsScreenMessage::RecipeChosen)
        ];

        scrollable(
            column![
                button("Back")
                    .on_press(SettingsScreenMessage::BackToDefault),
                water_weight_row,
                coffee_weight_row,
                text(format!("Ratio: {}", self.water_weight as f32 / self.coffee_weight as f32)),
                water_temp_row,
                grind_size_row,
                recipe_choosing_row,
                button("Start Recipe!")
                    .on_press(SettingsScreenMessage::RecipeStart)
            ]
        ).into()
    }
}





