#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPane {
    History,
    Branches,
    Commands,
    Details,
}

impl FocusPane {
    pub fn next(self) -> Self {
        match self {
            Self::History => Self::Branches,
            Self::Branches => Self::Commands,
            Self::Commands => Self::Details,
            Self::Details => Self::History,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::History => Self::Details,
            Self::Branches => Self::History,
            Self::Commands => Self::Branches,
            Self::Details => Self::Commands,
        }
    }
}
