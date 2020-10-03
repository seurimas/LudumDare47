pub use crate::assets::{AnimationId, PrefabStorage, SoundStorage, SpriteStorage};
pub use crate::stage::{Platform, StageState};
pub use amethyst::{
    animation::*,
    audio::{output::Output, Source, SourceHandle},
    core::{bundle::SystemBundle, timing::Time, Transform},
    ecs::world::LazyBuilder,
    ecs::*,
    error::Error,
    prelude::*,
    renderer::{
        camera::*,
        debug_drawing::DebugLines,
        palette::Srgba,
        plugins::{RenderDebugLines, RenderFlat2D, RenderToWindow},
        sprite::SpriteSheetHandle,
        types::{DefaultBackend, Texture},
        ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat,
    },
};
use amethyst::{
    assets::AssetStorage,
    shred::{ResourceId, SystemData},
};

pub fn get_active_animation(
    control_set: &AnimationControlSet<AnimationId, SpriteRender>,
) -> Option<AnimationId> {
    for (id, animation) in control_set.animations.iter() {
        if animation.state.is_running() {
            return Some(*id);
        }
    }
    None
}

pub fn set_active_animation(
    control_set: &mut AnimationControlSet<AnimationId, SpriteRender>,
    id: AnimationId,
    animation_set: &AnimationSet<AnimationId, SpriteRender>,
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
    pub fn play_normal(&self, get_sound: fn(&SoundStorage) -> SourceHandle) {
        if let Some(ref output) = self.output.as_ref() {
            if let Some(ref sounds) = self.storage.as_ref() {
                if let Some(sound) = self.sources.get(&get_sound(&sounds)) {
                    output.play_once(sound, 0.75);
                }
            }
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
