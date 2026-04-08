use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParrotLevel {
    Egg = 1,
    Chick = 2,
    Parrot = 3,
    Macaw = 4,
    Phoenix = 5,
}

impl ParrotLevel {
    pub fn name(&self) -> &str {
        match self {
            Self::Egg => "Egg",
            Self::Chick => "Chick",
            Self::Parrot => "Parrot",
            Self::Macaw => "Macaw",
            Self::Phoenix => "Phoenix",
        }
    }

    pub fn emoji(&self) -> &str {
        match self {
            Self::Egg => "🥚",
            Self::Chick => "🐥",
            Self::Parrot => "🦜",
            Self::Macaw => "🦚",
            Self::Phoenix => "🔥",
        }
    }

    pub fn xp_threshold(&self) -> u32 {
        match self {
            Self::Egg => 0,
            Self::Chick => 100,
            Self::Parrot => 500,
            Self::Macaw => 2000,
            Self::Phoenix => 5000,
        }
    }

    pub fn min_accuracy(&self) -> u32 {
        match self {
            Self::Egg => 0,
            Self::Chick => 30,
            Self::Parrot => 50,
            Self::Macaw => 70,
            Self::Phoenix => 85,
        }
    }

    pub fn all() -> &'static [ParrotLevel] {
        &[Self::Egg, Self::Chick, Self::Parrot, Self::Macaw, Self::Phoenix]
    }

    pub fn level_for(xp: u32, accuracy: u32, current: ParrotLevel) -> ParrotLevel {
        let mut best = current;
        for &level in Self::all().iter().rev() {
            if level <= current {
                break;
            }
            if xp >= level.xp_threshold() && accuracy >= level.min_accuracy() {
                best = level;
                break;
            }
        }
        best
    }
}

impl Default for ParrotLevel {
    fn default() -> Self {
        Self::Egg
    }
}
