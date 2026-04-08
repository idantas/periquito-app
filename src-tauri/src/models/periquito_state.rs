use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PeriquitoTask {
    Idle,
    Working,
    Sleeping,
    Compacting,
    Waiting,
}

impl PeriquitoTask {
    pub fn animation_fps(&self) -> f64 {
        match self {
            Self::Compacting => 6.0,
            Self::Sleeping => 2.0,
            Self::Idle => 3.0,
            Self::Waiting => 3.0,
            Self::Working => 4.0,
        }
    }

    pub fn sprite_prefix(&self) -> &str {
        match self {
            Self::Idle => "idle",
            Self::Working => "working",
            Self::Sleeping => "sleeping",
            Self::Compacting => "compacting",
            Self::Waiting => "waiting",
        }
    }

    pub fn bob_duration(&self) -> f64 {
        match self {
            Self::Sleeping => 4.0,
            Self::Idle | Self::Waiting => 1.5,
            Self::Working => 0.4,
            Self::Compacting => 0.5,
        }
    }

    pub fn bob_amplitude(&self) -> f64 {
        match self {
            Self::Sleeping | Self::Compacting => 0.0,
            Self::Idle => 1.5,
            Self::Waiting | Self::Working => 0.5,
        }
    }

    pub fn can_walk(&self) -> bool {
        matches!(self, Self::Idle | Self::Working)
    }

    pub fn display_name(&self) -> &str {
        match self {
            Self::Idle => "Idle",
            Self::Working => "Working...",
            Self::Sleeping => "Sleeping",
            Self::Compacting => "Compacting...",
            Self::Waiting => "Waiting...",
        }
    }

    pub fn walk_frequency_range(&self) -> (f64, f64) {
        match self {
            Self::Sleeping | Self::Waiting => (30.0, 60.0),
            Self::Idle => (8.0, 15.0),
            Self::Working => (5.0, 12.0),
            Self::Compacting => (15.0, 25.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PeriquitoEmotion {
    Neutral,
    Happy,
    Sad,
    Sob,
}

impl PeriquitoEmotion {
    pub fn sway_amplitude(&self) -> f64 {
        match self {
            Self::Neutral => 0.5,
            Self::Happy => 1.0,
            Self::Sad => 0.25,
            Self::Sob => 0.15,
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "happy" => Self::Happy,
            "sad" => Self::Sad,
            "sob" => Self::Sob,
            _ => Self::Neutral,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriquitoState {
    pub task: PeriquitoTask,
    pub emotion: PeriquitoEmotion,
}

impl Default for PeriquitoState {
    fn default() -> Self {
        Self {
            task: PeriquitoTask::Idle,
            emotion: PeriquitoEmotion::Neutral,
        }
    }
}

impl PeriquitoState {
    pub fn sprite_sheet_name(&self) -> String {
        format!("{}_{}", self.task.sprite_prefix(), match self.emotion {
            PeriquitoEmotion::Neutral => "neutral",
            PeriquitoEmotion::Happy => "happy",
            PeriquitoEmotion::Sad => "sad",
            PeriquitoEmotion::Sob => "sad", // fallback: sob uses sad sprite
        })
    }

    pub fn animation_fps(&self) -> f64 {
        match (self.task, self.emotion) {
            (PeriquitoTask::Idle, PeriquitoEmotion::Happy) => 10.0,
            (PeriquitoTask::Idle, PeriquitoEmotion::Sad) => 6.0,
            (PeriquitoTask::Idle, PeriquitoEmotion::Sob) => 4.0,
            (PeriquitoTask::Working, PeriquitoEmotion::Happy) => 12.0,
            (PeriquitoTask::Working, PeriquitoEmotion::Sad) => 8.0,
            (PeriquitoTask::Working, PeriquitoEmotion::Sob) => 6.0,
            (PeriquitoTask::Waiting, PeriquitoEmotion::Happy) => 10.0,
            (PeriquitoTask::Waiting, PeriquitoEmotion::Sad) => 6.0,
            (PeriquitoTask::Waiting, PeriquitoEmotion::Sob) => 5.0,
            (PeriquitoTask::Compacting, _) => 14.0,
            (PeriquitoTask::Sleeping, _) => 6.0,
            (PeriquitoTask::Idle, PeriquitoEmotion::Neutral) => 8.0,
            (PeriquitoTask::Working, PeriquitoEmotion::Neutral) => 10.0,
            (PeriquitoTask::Waiting, PeriquitoEmotion::Neutral) => 8.0,
        }
    }

    pub fn frame_count(&self) -> usize {
        match (self.task, self.emotion) {
            (PeriquitoTask::Idle, PeriquitoEmotion::Happy) => 16,
            (PeriquitoTask::Idle, PeriquitoEmotion::Sad | PeriquitoEmotion::Sob) => 16,
            (PeriquitoTask::Working, PeriquitoEmotion::Sad | PeriquitoEmotion::Sob) => 16,
            (PeriquitoTask::Waiting, PeriquitoEmotion::Sob) => 16,
            (PeriquitoTask::Sleeping, _) => 16,
            _ => 8,
        }
    }

    pub fn columns(&self) -> usize {
        4
    }

    pub fn bob_duration(&self) -> f64 {
        self.task.bob_duration()
    }

    pub fn bob_amplitude(&self) -> f64 {
        match self.emotion {
            PeriquitoEmotion::Sob => 0.0,
            PeriquitoEmotion::Sad => self.task.bob_amplitude() * 0.5,
            _ => self.task.bob_amplitude(),
        }
    }

    pub fn can_walk(&self) -> bool {
        if self.emotion == PeriquitoEmotion::Sob {
            false
        } else {
            self.task.can_walk()
        }
    }
}
