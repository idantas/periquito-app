use crate::models::periquito_state::PeriquitoEmotion;
use std::collections::HashMap;

const SAD_THRESHOLD: f64 = 0.45;
const HAPPY_THRESHOLD: f64 = 0.6;
const SOB_ESCALATION_THRESHOLD: f64 = 0.9;
const INTENSITY_DAMPEN: f64 = 0.5;
const DECAY_RATE: f64 = 0.92;
const INTER_EMOTION_DECAY: f64 = 0.9;
const NEUTRAL_COUNTER_DECAY: f64 = 0.85;

pub struct EmotionState {
    scores: HashMap<PeriquitoEmotion, f64>,
    current: PeriquitoEmotion,
}

impl EmotionState {
    pub fn new() -> Self {
        let mut scores = HashMap::new();
        scores.insert(PeriquitoEmotion::Happy, 0.0);
        scores.insert(PeriquitoEmotion::Sad, 0.0);
        Self {
            scores,
            current: PeriquitoEmotion::Neutral,
        }
    }

    pub fn current_emotion(&self) -> PeriquitoEmotion {
        self.current
    }

    pub fn record_emotion(&mut self, raw_emotion: &str, intensity: f64) {
        let emotion = PeriquitoEmotion::from_str(raw_emotion);

        if emotion != PeriquitoEmotion::Neutral {
            let dampened = intensity * INTENSITY_DAMPEN;
            let score = self.scores.entry(emotion).or_insert(0.0);
            *score = (*score + dampened).min(1.0);

            let keys: Vec<PeriquitoEmotion> = self.scores.keys().cloned().collect();
            for key in keys {
                if key != emotion {
                    if let Some(s) = self.scores.get_mut(&key) {
                        *s *= INTER_EMOTION_DECAY;
                    }
                }
            }
        } else {
            for s in self.scores.values_mut() {
                *s *= NEUTRAL_COUNTER_DECAY;
            }
        }

        self.update_current();
    }

    pub fn decay_all(&mut self) -> bool {
        let mut any_changed = false;
        for s in self.scores.values_mut() {
            let old = *s;
            *s *= DECAY_RATE;
            if *s < 0.01 {
                *s = 0.0;
            }
            if (*s - old).abs() > 0.001 {
                any_changed = true;
            }
        }
        if any_changed {
            self.update_current();
        }
        any_changed
    }

    fn update_current(&mut self) {
        let best = self
            .scores
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal));

        self.current = match best {
            Some((&emotion, &score)) => {
                let threshold = if emotion == PeriquitoEmotion::Sad {
                    SAD_THRESHOLD
                } else {
                    HAPPY_THRESHOLD
                };
                if score >= threshold {
                    if emotion == PeriquitoEmotion::Sad && score >= SOB_ESCALATION_THRESHOLD {
                        PeriquitoEmotion::Sob
                    } else {
                        emotion
                    }
                } else {
                    PeriquitoEmotion::Neutral
                }
            }
            None => PeriquitoEmotion::Neutral,
        };
    }
}
