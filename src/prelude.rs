pub use crate::assets::{
    AnimationId, PrefabStorage, SoundStorage, SpriteEntityPrefabData, SpriteStorage, NOTE_COUNT,
};
pub use crate::music::Note;
pub use crate::player::Player;
pub use crate::stage::{Platform, StageState};
pub use amethyst::{
    animation::*,
    assets::{Handle, Prefab},
    assets::{PrefabData, ProgressCounter},
    audio::{output::Output, Source, SourceHandle},
    core::{bundle::SystemBundle, timing::Time, Transform},
    derive::PrefabData,
    ecs::world::LazyBuilder,
    ecs::*,
    error::Error,
    prelude::*,
    renderer::{
        camera::*,
        debug_drawing::DebugLines,
        palette::{Hsl, RgbHue, Srgba},
        plugins::{RenderDebugLines, RenderFlat2D, RenderToWindow},
        resources::Tint,
        sprite::SpriteSheetHandle,
        types::{DefaultBackend, Texture},
        ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat,
    },
};
use amethyst::{
    assets::AssetStorage,
    shred::{ResourceId, SystemData},
};
pub use rand::prelude::*;
pub use serde::{Deserialize, Serialize};

pub fn get_active_animation<T: amethyst::animation::AnimationSampling>(
    control_set: &AnimationControlSet<AnimationId, T>,
) -> Option<AnimationId> {
    for (id, animation) in control_set.animations.iter() {
        if animation.state.is_running() {
            return Some(*id);
        }
    }
    None
}

pub fn set_active_animation<T: amethyst::animation::AnimationSampling>(
    control_set: &mut AnimationControlSet<AnimationId, T>,
    id: AnimationId,
    animation_set: &AnimationSet<AnimationId, T>,
    end: EndControl,
    rate_multiplier: f32,
) {
    let mut actives = Vec::new();
    for (active_id, animation) in control_set.animations.iter() {
        if animation.state.is_running() && *active_id != id {
            actives.push(*active_id);
        }
    }
    for active in actives {
        control_set.abort(active);
    }
    control_set.add_animation(
        id,
        &animation_set.get(&id).unwrap(),
        end,
        rate_multiplier,
        AnimationCommand::Start,
    );
}

#[derive(SystemData)]
pub struct SoundPlayer<'a> {
    storage: Option<Read<'a, SoundStorage>>,
    output: Option<Read<'a, Output>>,
    sources: Read<'a, AssetStorage<Source>>,
}

impl<'a> SoundPlayer<'a> {
    pub fn play_normal(&self, get_sound: impl Fn(&SoundStorage) -> &SourceHandle) {
        if let Some(ref output) = self.output.as_ref() {
            if let Some(ref sounds) = self.storage.as_ref() {
                if let Some(sound) = self.sources.get(get_sound(&sounds)) {
                    output.play_once(sound, 0.75);
                }
            }
        }
    }
}

#[derive(SystemData)]
pub struct PrefabSpawner<'a> {
    sprites: Option<Read<'a, SpriteStorage>>,
    prefabs: Option<Read<'a, PrefabStorage>>,
    lazy: Read<'a, LazyUpdate>,
    pub entities: Entities<'a>,
}

impl<'a> PrefabSpawner<'a> {
    pub fn spawn_decor(
        &self,
        transform: Transform,
        get_sprite: impl Fn(&SpriteStorage) -> &SpriteSheetHandle,
        sprite_number: usize,
        modify: impl FnOnce(LazyBuilder) -> LazyBuilder,
    ) -> Option<Entity> {
        if let Some(sprites) = &self.sprites {
            Some(
                modify(
                    self.lazy
                        .create_entity(&self.entities)
                        .with(transform)
                        .with(SpriteRender::new(
                            get_sprite(sprites).clone(),
                            sprite_number,
                        )),
                )
                .build(),
            )
        } else {
            None
        }
    }

    pub fn spawn_prefab(
        &self,
        get_prefab: impl Fn(&PrefabStorage) -> &Handle<Prefab<SpriteEntityPrefabData>>,
        modify: impl FnOnce(LazyBuilder) -> LazyBuilder,
    ) -> Option<Entity> {
        if let Some(prefabs) = &self.prefabs {
            Some(
                modify(
                    self.lazy
                        .create_entity(&self.entities)
                        .with(get_prefab(&prefabs).clone()),
                )
                .build(),
            )
        } else {
            None
        }
    }
}

pub fn distance_2d_iso(va: &na19::Vector3<f32>, vb: &na19::Vector3<f32>) -> f32 {
    let dx = (va.x - vb.x) / 2.0;
    let dy = va.y - vb.y;
    f32::sqrt((dx * dx) + (dy * dy))
}

pub fn normalize_iso(direction: &mut na19::Vector3<f32>) {
    direction.x *= 0.5;
    *direction = direction.normalize();
    direction.x *= 2.0;
}

pub fn tether_at(mover: &mut na19::Vector3<f32>, tether: &na19::Vector3<f32>, length: f32) {
    let distance = distance_2d_iso(mover, tether);
    if distance > length {
        let mut direction = (*mover - *tether);
        normalize_iso(&mut direction);
        mover.x = tether.x + (direction.x * length);
        mover.y = tether.y + (direction.y * length);
    }
}

pub fn lerp(progress: f32, v1: f32, v2: f32) -> f32 {
    let diff = v2 - v1;
    v1 + (progress * diff)
}

pub fn rand_in<T>(vec: &Vec<T>) -> &T {
    vec.get(thread_rng().gen_range(0, vec.len()))
        .expect("Nothing in vector")
}

pub fn note_color(note: Note) -> Srgba {
    Hsl::new(
        RgbHue::from_degrees(360.0 / (NOTE_COUNT as f32) * (note as f32)),
        1.,
        0.5,
    )
    .into()
}

pub fn rand_color() -> Srgba {
    Hsl::new(
        RgbHue::from_radians(thread_rng().gen_range(0., std::f32::consts::PI * 2.0)),
        1.,
        0.5,
    )
    .into()
}

pub fn black() -> Srgba {
    Hsl::new(RgbHue::from_radians(0.0), 1., 0.).into()
}

pub fn rand_upto(max: usize) -> usize {
    thread_rng().gen_range(0, max)
}

pub fn rand_chance(chance: f32) -> bool {
    thread_rng().gen_range(0.0, 1.0) < chance
}

pub const C4: usize = 0;
pub const D4: usize = 2;
pub const E4: usize = 4;
pub const F4: usize = 5;
pub const G4: usize = 7;
pub const A4: usize = 9;
pub const B4: usize = 11;
pub const C5: usize = 12;
pub const D5: usize = 14;
pub const E5: usize = 16;
pub const F5: usize = 17;
pub const G5: usize = 19;
