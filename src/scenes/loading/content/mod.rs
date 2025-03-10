#[derive(Clone, Copy)]
pub enum LoadingBarMessages {
    Lab0intro
}

impl LoadingBarMessages{
    pub fn content(&self) -> &str {
        match self {
            LoadingBarMessages::Lab0intro => include_str!("./lab_0_loading_bar.json")
        }
    }
}

#[derive(Clone, Copy)]
pub enum LoadingButtonMessages {
    Lab0intro
}

impl LoadingButtonMessages{
    pub fn content(&self) -> &str {
        match self {
            LoadingButtonMessages::Lab0intro => include_str!("./lab_0_loading_button.json")
        }
    }
}

