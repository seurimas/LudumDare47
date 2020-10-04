use crate::prelude::*;
use std::collections::HashMap;

pub type Note = usize;
pub const SUBNOTES: i32 = 4;

#[derive(Debug, Clone)]
pub struct Song {
    pub bpm: i32,
    pub structures: Vec<Substructure>,
    pub payouts: Vec<Substructure>,
}

impl Song {
    pub fn get_notes_at(&self, beat: i32) -> Vec<Note> {
        let mut notes = Vec::new();
        for structure in self.structures.iter() {
            notes.append(&mut structure.get_notes_at(beat));
        }
        notes
    }
    pub fn get_rewards_at(&self, beat: i32) -> Vec<Note> {
        let mut payouts = Vec::new();
        for payout in self.payouts.iter() {
            payouts.append(&mut payout.get_notes_at(beat));
        }
        payouts
    }
}

impl Default for Song {
    fn default() -> Self {
        Song {
            bpm: 48,
            structures: vec![Substructure::row_your_boat()],
            payouts: vec![Substructure::random()],
        }
    }
}

#[derive(Debug, Clone)]
pub enum Substructure {
    Round {
        notes: HashMap<i32, Note>,
        rounds: i32,
        repeat_at: i32,
        restart_at: i32,
        pitch_up: usize,
    },
    Random {
        notes: Vec<Note>,
        interval: i32,
        chance: f32,
    },
}

impl Substructure {
    fn row_your_boat() -> Self {
        let mut notes = HashMap::new();
        // C D E F G A B  C  D  E  F  G
        // 0 2 4 5 7 9 11 12 14 16 17 19
        let progression = [
            (0, C4), // Row
            (3, C4), // Row
            (6, C4), // Row
            (8, D4),
            (9, E4),
            (12, E4), // Gently/Round 2
            (14, D4),
            (15, E4),
            (17, F4),
            (18, G4), // Stream
            (24, C5), // Merrily
            (25, C5),
            (26, C5),
            (27, G4), // Merrily
            (28, G4),
            (29, G4),
            (30, E4), // Merrily
            (31, E4),
            (32, E4),
            (33, C4), // Merrily
            (34, C4),
            (35, C4),
            (36, G4), // Life
            (38, F4), // Is
            (39, E4), // But
            (41, D4), // A
            (42, C4), // Dream
        ];
        for (beat, note) in progression.iter() {
            notes.insert(*beat, *note);
        }
        let rounds = 2;
        let repeat_at = 12;
        let restart_at = 57;
        let pitch_up = 0;
        Substructure::Round {
            notes,
            rounds,
            repeat_at,
            pitch_up,
            restart_at,
        }
    }

    fn random() -> Self {
        Substructure::Random {
            notes: vec![C4, D4, E4, F4, G4, A4, B4, C5, D5, G5],
            interval: 12,
            chance: 1.,
        }
    }

    fn get_notes_at(&self, beat: i32) -> Vec<Note> {
        match self {
            Substructure::Round {
                notes,
                rounds,
                repeat_at,
                pitch_up,
                restart_at,
            } => {
                let mut notes_at = Vec::new();
                for i in 0..*rounds {
                    let nominal_beat = (beat % restart_at) - (repeat_at * i);
                    if let Some(note) = notes.get(&nominal_beat) {
                        let note = (note + (*pitch_up * i as usize)) % NOTE_COUNT;
                        notes_at.push(note);
                    }
                }
                notes_at
            }
            Substructure::Random {
                notes,
                interval,
                chance,
            } => {
                if (beat % interval) == 0 && rand_chance(*chance) {
                    vec![*rand_in(notes)]
                } else {
                    vec![]
                }
            }
        }
    }
}
