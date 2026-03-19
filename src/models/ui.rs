#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPane {
    History,
    Branches,
    Commands,
    Details,
    Output,
}

impl FocusPane {
    pub fn next(self) -> Self {
        match self {
            Self::History => Self::Branches,
            Self::Branches => Self::Commands,
            Self::Commands => Self::Details,
            Self::Details => Self::Output,
            Self::Output => Self::History,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::History => Self::Output,
            Self::Branches => Self::History,
            Self::Commands => Self::Branches,
            Self::Details => Self::Commands,
            Self::Output => Self::Details,
        }
    }
}
