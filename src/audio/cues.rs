//! When the JONES'IN! sounds play, from the Tetris celebration's clock before and after a frame.
//!
//! The fanfare starts with the celebration (the moment the four lines vanish and the rainbow
//! text pops in). The voice clip, when there is one, comes in [`VOICE_AT`] seconds later, over
//! the fanfare's held chord.

/// Seconds into the celebration that the voice clip starts: after the fanfare's arpeggio.
pub const VOICE_AT: f64 = 0.3;

/// What to play this frame for JONES'IN!.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct JonesinCues {
    pub fanfare: bool,
    pub voice: bool,
}

/// `before` and `after` are the celebration's elapsed time at the start and end of the frame,
/// `None` when no celebration is running.
pub fn jonesin_cues(before: Option<f64>, after: Option<f64>) -> JonesinCues {
    let Some(now) = after else {
        return JonesinCues::default();
    };
    // A new celebration: none before, or one that restarted (a Tetris during the last one).
    let started = before.map_or(true, |was| now < was);
    let voice_was_due = !started && before.is_some_and(|was| was >= VOICE_AT);
    JonesinCues {
        fanfare: started,
        voice: now >= VOICE_AT && !voice_was_due,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_without_a_celebration() {
        assert_eq!(jonesin_cues(None, None), JonesinCues::default());
        assert_eq!(jonesin_cues(Some(1.9), None), JonesinCues::default(), "ending is silent");
    }

    #[test]
    fn fanfare_when_it_starts() {
        let cues = jonesin_cues(None, Some(0.0));
        assert!(cues.fanfare);
        assert!(!cues.voice);
    }

    #[test]
    fn voice_once_when_its_time_comes() {
        assert!(!jonesin_cues(Some(0.1), Some(0.2)).voice);
        assert!(jonesin_cues(Some(0.29), Some(0.31)).voice);
        assert!(!jonesin_cues(Some(0.31), Some(0.33)).voice, "only once");
        assert!(!jonesin_cues(Some(0.31), Some(0.33)).fanfare);
    }

    #[test]
    fn a_long_frame_gives_both() {
        // The first frame after a hitch can jump straight past VOICE_AT.
        assert_eq!(jonesin_cues(None, Some(0.5)), JonesinCues { fanfare: true, voice: true });
    }

    #[test]
    fn a_second_tetris_restarts_both() {
        // Back-to-back Tetrises: the clock goes back to 0 while the last celebration runs.
        let cues = jonesin_cues(Some(1.2), Some(0.0));
        assert!(cues.fanfare);
        assert!(!cues.voice);
        assert!(jonesin_cues(Some(0.2), Some(0.35)).voice);
    }
}
