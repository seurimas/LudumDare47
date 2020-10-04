use crate::prelude::*;
use amethyst::renderer::{palette::Srgba, resources::Tint};

#[derive(Component, Debug, Copy, Clone)]
#[storage(VecStorage)]
pub struct NotePickup {
    color: Option<Srgba>,
    platform: Entity,
    value: Note,
    ttl: f32,
}

impl NotePickup {
    pub fn new(platform: Entity, value: Note, ttl: f32) -> Self {
        NotePickup {
            color: None,
            platform,
            value,
            ttl,
        }
    }
}

pub struct NotePickupSystem;
impl<'s> System<'s> for NotePickupSystem {
    type SystemData = (
        WriteStorage<'s, NotePickup>,
        WriteStorage<'s, Player>,
        Write<'s, StageState>,
        Entities<'s>,
        Read<'s, Time>,
    );
    fn run(&mut self, (mut notes, players, mut stage_state, entities, time): Self::SystemData) {
        for (mut note, note_entity) in (&mut notes, &entities).join() {
            for (player) in (&players).join() {
                if Some(note.platform) == player.platform && !player.state.is_airborne() {
                    entities.delete(note_entity);
                    stage_state.notes_found.push(note.value);
                }
            }
            note.ttl -= time.delta_seconds();
            if note.ttl < 0.0 {
                entities.delete(note_entity);
            }
        }
    }
}

pub struct NoteAnimationSystem;
impl<'s> System<'s> for NoteAnimationSystem {
    type SystemData = (
        WriteStorage<'s, NotePickup>,
        WriteStorage<'s, SpriteRender>,
        WriteStorage<'s, Tint>,
        ReadStorage<'s, AnimationSet<AnimationId, Transform>>,
        WriteStorage<'s, AnimationControlSet<AnimationId, Transform>>,
        Entities<'s>,
    );
    fn run(
        &mut self,
        (
            mut notes,
            mut sprites,
            mut tints,
            t_animation_sets,
            mut t_control_sets,
            entities,
        ): Self::SystemData,
    ) {
        for (mut note, t_animation_set, mut sprite, entity) in
            (&mut notes, &t_animation_sets, &mut sprites, &entities).join()
        {
            if let Some(t_control_set) = get_animation_set(&mut t_control_sets, entity) {
                if get_active_animation(t_control_set).is_none() {
                    set_active_animation(
                        t_control_set,
                        AnimationId::Beat,
                        &t_animation_set,
                        EndControl::Loop(None),
                        1.0,
                    );
                }
            }
            if note.color.is_none() {
                let color = note_color(note.value);
                note.color = Some(color);
                tints.insert(entity, Tint(color));
                sprite.sprite_number = rand_upto(3);
            }
        }
    }
}
