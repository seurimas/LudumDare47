use crate::prelude::*;
use std::collections::HashMap;

pub type Note = usize;
pub const SUBNOTES: i32 = 4;

#[derive(Debug, Clone)]
pub struct Song {
    pub bpm: i32,
    pub structures: Vec<Substructure>,
    pub payouts: Vec<Substructure>,
    pub next_notes: Vec<Note>,
}

impl Song {
    pub fn get_notes_at(&self, beat: i32) -> Vec<Note> {
        let mut notes = Vec::new();
        for structure in self.structures.iter() {
            notes.append(&mut structure.get_notes_at(beat));
        }
        notes
    }
    pub fn get_rewards_at(&self, beat: i32, paid_out: &Vec<Note>) -> Vec<Note> {
        let mut payouts = Vec::new();
        for payout in self.payouts.iter() {
            payouts.append(&mut payout.get_notes_at(beat));
        }
        payouts.retain(|note| !paid_out.contains(note));
        payouts
    }
    pub fn done(&self, beat: i32) -> bool {
        let nominal_beat = beat - ((self.bpm / 60) * 4) - 8;
        for structure in self.structures.iter() {
            if !structure.done(nominal_beat) {
                return false;
            }
        }
        true
    }
}

pub const SONG_COUNT: usize = 5;

impl Song {
    pub fn row_your_boat() -> Self {
        Song {
            bpm: 48,
            structures: vec![Substructure::row_your_boat()],
            payouts: vec![Substructure::reward_row_row()],
            next_notes: vec![C4, D4, E4, F4, G4, A4, B4, C5],
        }
    }
    pub fn alouette() -> Self {
        Song {
            bpm: 120,
            structures: vec![Substructure::alouette()],
            payouts: vec![Substructure::reward_alouette()],
            next_notes: vec![C4, D4, E4, F4, G4, A4, B4, C5],
        }
    }
    pub fn c_scale() -> Self {
        Song {
            bpm: 240,
            structures: vec![Substructure::c_scales()],
            payouts: vec![Substructure::reward_c_scale()],
            next_notes: vec![C4, D4, E4, F4, G4, A4, B4, C5],
        }
    }
    pub fn coffee() -> Self {
        Song {
            bpm: 240,
            structures: vec![Substructure::coffee()],
            payouts: vec![Substructure::reward_coffee()],
            next_notes: vec![C4, D4, E4, F4, G4, A4, B4, C5],
        }
    }
    pub fn donkeys() -> Self {
        Song {
            bpm: 72,
            structures: vec![Substructure::donkeys()],
            payouts: vec![Substructure::reward_donkeys()],
            next_notes: vec![C4, D4, E4, F4, G4, A4, B4, C5],
        }
    }
    pub fn songs() -> [Song; SONG_COUNT] {
        [
            Song::row_your_boat(),
            Song::c_scale(),
            Song::alouette(),
            Song::coffee(),
            Song::donkeys(),
        ]
    }
    pub fn payout_song(notes: &Vec<Note>) -> Self {
        Song {
            bpm: 300,
            structures: vec![Substructure::Scale {
                notes: notes.clone(),
                interval: 4,
            }],
            payouts: vec![],
            next_notes: vec![],
        }
    }
    pub fn lose_song() -> Self {
        let mut notes = vec![
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
        ];
        rand_shuffle(&mut notes);
        println!("{:?}", notes);
        Song {
            bpm: 300,
            structures: vec![Substructure::Scale { notes, interval: 4 }],
            payouts: vec![],
            next_notes: vec![],
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
    Scale {
        notes: Vec<Note>,
        interval: i32,
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
    fn alouette() -> Self {
        let mut notes = HashMap::new();
        // C D E F G A B  C  D  E  F  G
        // 0 2 4 5 7 9 11 12 14 16 17 19
        let progression = [
            // Measure 1
            (0, F4),  // A
            (6, G4),  // lou
            (8, A4),  // et
            (12, A4), // te
            // Measure 2
            (16, G4), // gen
            (18, F4), // tille
            (20, G4), // A
            (22, A4), // lou
            (24, F4), // et
            (28, C4), // te
            // Measure 3
            (32, F4), // A
            (38, G4), // lou
            (40, A4), // et
            (44, A4), // te
            // Measure 4
            (48, G4), // je
            (50, F4), // te
            (52, G4), // plu
            (54, A4), // mer
            (56, F4), // ai
            // Measure 5
            (64, F4), // je
            (66, E4), // te
            (68, F4), // plu
            (70, E4), // mer
            (72, F4), // ai
            (74, A4), // la
            (76, C5), // tete
            // Measure 6
            (80, C5),     // je
            (82, D5),     // te
            (84, C5),     // plu
            (86, B4 + 1), // mer
            (88, A4),     // ai
            (90, G4),     // la
            (92, F4),     // tete
            // Measure 7
            (96, C5),  // Et
            (98, C5),  // le
            (100, C5), // tete
            (104, C4), // Et
            (106, C4), // la
            (108, C4), // tete
            // Measure 8
            (112, C5), // Oh
            (114, C5), // Oh
            (116, C5), // Oh
            (118, C5), // Oh
            (120, C5), // Oh
            (122, C5), // Oh
            (124, C5), // Oh
            (126, C5), // Oh
        ];
        for (beat, note) in progression.iter() {
            notes.insert(*beat, *note);
        }
        let rounds = 2;
        let repeat_at = 64;
        let restart_at = 128 + 64;
        let pitch_up = 0;
        Substructure::Round {
            notes,
            rounds,
            repeat_at,
            pitch_up,
            restart_at,
        }
    }

    fn c_scales() -> Self {
        let mut notes = HashMap::new();
        // C D E F G A B  C  D  E  F  G
        // 0 2 4 5 7 9 11 12 14 16 17 19
        let progression = [
            (0, C4),
            (4, D4),
            (8, E4),
            (12, F4),
            (16, G4),
            (20, A4),
            (24, B4),
            (28, C5),
            (32, B4),
            (36, A4),
            (40, G4),
            (44, F4),
            (48, E4),
            (52, D4),
            (56, C4),
        ];
        for (beat, note) in progression.iter() {
            notes.insert(*beat, *note);
        }
        let rounds = 3;
        let repeat_at = 28;
        let restart_at = 60 + 84;
        let pitch_up = 4;
        Substructure::Round {
            notes,
            rounds,
            repeat_at,
            pitch_up,
            restart_at,
        }
    }

    fn reward_c_scale() -> Self {
        let mut notes = HashMap::new();
        let progression = [
            (12, C5),
            (36, B4),
            (60, A4),
            (74, G4),
            (98, F4),
            (112, E4),
            (124, D4),
            (136, C4),
        ];
        for (beat, note) in progression.iter() {
            notes.insert(*beat, *note);
        }
        let rounds = 1;
        let repeat_at = 144;
        let restart_at = 144;
        let pitch_up = 0;
        Substructure::Round {
            notes,
            rounds,
            repeat_at,
            pitch_up,
            restart_at,
        }
    }

    fn coffee() -> Self {
        let mut notes = HashMap::new();
        // C D E F G A B  C  D  E  F  G
        // 0 2 4 5 7 9 11 12 14 16 17 19
        let progression = [
            (0, C5),  // C
            (4, A4),  // O
            (8, F4),  // F
            (12, F4), // F
            (16, E4), // E
            (20, E4), // E
            (24, E4), // Not
            (28, G4), // Tea
            (32, E4), // But
            (36, F4), // Cof
            (38, E4), // fee
            (40, F4), // is
            (42, G4), // for
            (44, F4), // me
            // Round 2
            (48, A4),     // Cof
            (50, A4),     // fee
            (52, C5),     // mo
            (54, C5),     // cha
            (56, A4),     // Cof
            (58, A4),     // fee
            (60, B4 - 1), // la
            (62, A4),     // tte
            (64, B4 - 1), // Cof
            (66, C5),     // fee
            (68, B4 - 1), // black
            // Next verse
            (72, G4),     // Are
            (74, G4),     // you
            (76, B4 - 1), // gro
            (78, B4 - 1), // ggy
            (80, G4),     // Cof
            (82, G4),     // fee
            (84, A4),     // gives
            (86, G4),     // you
            (88, A4),     // what
            (90, B4 - 1), // you
            (92, A4),     // lack!
            // Round 3 and final verse
            (96, F4),  // When
            (100, F4), // i
            (104, F4), // am
            (108, G4), // feel
            (112, G4), // ing
            (116, G4), // slow
            (120, C4), // Time
            (124, C4), // for
            (128, C5), // a
            (132, C5), // cup
            (136, C4), // of
            (140, F4), // Joe!
        ];
        for (beat, note) in progression.iter() {
            notes.insert(*beat, *note);
        }
        let rounds = 3;
        let repeat_at = 48;
        let restart_at = 48 * 5;
        let pitch_up = 0;
        Substructure::Round {
            notes,
            rounds,
            repeat_at,
            pitch_up,
            restart_at,
        }
    }

    fn reward_coffee() -> Self {
        let mut notes = HashMap::new();
        let progression = [
            (32, C4),
            (64, E4),
            (96, F4),
            (128, G4 + 1),
            (160, A4),
            (192, B4 - 1),
            (224, C5),
            (232, F5),
        ];
        for (beat, note) in progression.iter() {
            notes.insert(*beat, *note);
        }
        let rounds = 1;
        let repeat_at = 240;
        let restart_at = 240;
        let pitch_up = 0;
        Substructure::Round {
            notes,
            rounds,
            repeat_at,
            pitch_up,
            restart_at,
        }
    }

    fn donkeys() -> Self {
        let mut notes = HashMap::new();
        // C D E F G A B  C  D  E  F  G
        // 0 2 4 5 7 9 11 12 14 16 17 19
        let progression = [
            // Round 1
            (0, E4),      // Don
            (2, E4),      // keys
            (4, E4),      // are
            (6, F4 + 1),  // in
            (8, G4 + 1),  // love
            (10, F4 + 1), // with
            (12, E4),     // car
            (14, C4),     // rots
            // Round 2
            (16, G4 + 1), // Car
            (18, G4 + 1), // rots
            (20, G4 + 1), // aren't
            (22, A4),     // in
            (24, B4),     // love
            (26, A4),     // at
            (28, G4 + 1), // all
            // Round 3
            (32, E5), // all
            (36, B4), // all
            (40, E5), // all
            (44, B4), // all
            // Round 4
            (48, B4),     // Car
            (50, C5 + 1), // rots
            (52, B4),     // aren't
            (54, A4),     // in
            (56, G4 + 1), // love
            (58, F4 + 1), // at
            (60, E4),     // all
        ];
        for (beat, note) in progression.iter() {
            notes.insert(*beat, *note);
        }
        let rounds = 4;
        let repeat_at = 16;
        let restart_at = 16 * 7;
        let pitch_up = 0;
        Substructure::Round {
            notes,
            rounds,
            repeat_at,
            pitch_up,
            restart_at,
        }
    }

    fn reward_donkeys() -> Self {
        let mut notes = HashMap::new();
        let progression = [
            (12, E4),
            (24, F4 + 1),
            (36, G4 + 1),
            (48, A4),
            (60, C4 + 1),
            (72, C4),
            (84, E5),
            (96, B4),
        ];
        for (beat, note) in progression.iter() {
            notes.insert(*beat, *note);
        }
        let rounds = 1;
        let repeat_at = 112;
        let restart_at = 112;
        let pitch_up = 0;
        Substructure::Round {
            notes,
            rounds,
            repeat_at,
            pitch_up,
            restart_at,
        }
    }

    fn reward_row_row() -> Self {
        let mut notes = HashMap::new();
        let progression = [
            (6, C4),
            (12, D4),
            (18, E4),
            (24, F4),
            (30, G4),
            (36, A4),
            (42, B4),
            (48, C5),
        ];
        for (beat, note) in progression.iter() {
            notes.insert(*beat, *note);
        }
        let rounds = 1;
        let repeat_at = 57;
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

    fn reward_alouette() -> Self {
        let mut notes = HashMap::new();
        let progression = [
            (16, C4),
            (48, D4),
            (64, E4),
            (80, F4),
            (96, G4),
            (128, A4),
            (144, B4),
            (160, C5),
        ];
        for (beat, note) in progression.iter() {
            notes.insert(*beat, *note);
        }
        let rounds = 1;
        let repeat_at = 128 + 64;
        let restart_at = 128 + 64;
        let pitch_up = 0;
        Substructure::Round {
            notes,
            rounds,
            repeat_at,
            pitch_up,
            restart_at,
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
            Substructure::Scale { notes, interval } => {
                if beat % interval == 0 && (beat / interval) < (notes.len() as i32) {
                    vec![*notes
                        .get((beat / interval) as usize)
                        .expect("Missing scale note")]
                } else {
                    vec![]
                }
            }
        }
    }

    fn done(&self, beat: i32) -> bool {
        match self {
            Substructure::Round { .. } => false,
            Substructure::Scale { notes, interval } => beat > (notes.len() as i32 + 1) * interval,
        }
    }
}
