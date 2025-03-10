#[derive(Clone, Copy)]
pub enum BackgroundTypes {
    Desert
}

impl BackgroundTypes{
    pub fn content(&self) -> &str {
        match self {
            BackgroundTypes::Desert => include_str!("./desert.json")
        }
    }
}
