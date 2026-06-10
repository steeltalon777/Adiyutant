use serde::{Deserialize, Serialize};

/// Origin of a nudge/notification instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NudgeSource {
    Checkpoint,
    LocalRule,
    Timer,
    External,
}

impl NudgeSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            NudgeSource::Checkpoint => "Checkpoint",
            NudgeSource::LocalRule => "LocalRule",
            NudgeSource::Timer => "Timer",
            NudgeSource::External => "External",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Checkpoint" => Some(NudgeSource::Checkpoint),
            "LocalRule" => Some(NudgeSource::LocalRule),
            "Timer" => Some(NudgeSource::Timer),
            "External" => Some(NudgeSource::External),
            _ => None,
        }
    }
}

/// Instruction for the UI layer to display a notification.
///
/// This is the boundary DTO that the Android / CLI layer receives
/// when a nudge should be shown to the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationInstructionDto {
    pub source: String,
    pub title: String,
    pub body: String,
    pub action_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nudge_source_round_trip() {
        for s in &[
            NudgeSource::Checkpoint,
            NudgeSource::LocalRule,
            NudgeSource::Timer,
            NudgeSource::External,
        ] {
            let str = s.as_str();
            let back = NudgeSource::from_str(str).unwrap();
            assert_eq!(*s, back);
        }
    }
}
