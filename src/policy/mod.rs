pub mod engine;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Terminate,
    Callback,
    Log,
    Ignore,
}

impl std::str::FromStr for Action {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "terminate" => Action::Terminate,
            "callback" => Action::Callback,
            "log" => Action::Log,
            "ignore" => Action::Ignore,
            _ => Action::Log,
        })
    }
}
