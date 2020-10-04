#![allow(warnings)]
extern crate nalgebra as na;
extern crate nalgebra19 as na19;
mod assets;
mod hazards;
mod music;
mod pickups;
mod player;
mod prelude;
mod stage;
use amethyst::{
    animation::AnimationBundle,
    assets::*,
    audio::{output::init_output, AudioBundle, SourceHandle, WavFormat},
    core::transform::*,
    ecs::*,
    input::{is_close_requested, is_key_down},
    prelude::*,
    renderer::{
        bundle::RenderingBundle,
        camera::*,
        debug_drawing::DebugLines,
        palette::Srgba,
        plugins::{RenderDebugLines, RenderFlat2D, RenderToWindow},
        sprite::SpriteSheetHandle,
        types::{DefaultBackend, Texture},
        ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat,
    },
    tiles::{MortonEncoder, RenderTiles2D},
    ui::{RenderUi, UiBundle, UiCreator, UiEventType, UiFinder},
    utils::{
        application_root_dir,
        fps_counter::{FpsCounter, FpsCounterBundle},
    },
    window::ScreenDimensions,
    winit::VirtualKeyCode,
};
use amethyst_imgui::RenderImgui;
use assets::*;
use hazards::*;
use imgui::*;
use player::*;
use stage::*;

struct ImguiDebugSystem {
    listbox_item_current: i32,
    box_current: i32,
}

impl Default for ImguiDebugSystem {
    fn default() -> Self {
        ImguiDebugSystem {
            listbox_item_current: 0,
            box_current: 0,
        }
    }
}
impl<'s> amethyst::ecs::System<'s> for ImguiDebugSystem {
    type SystemData = (
        Read<'s, FpsCounter>,
        Option<Read<'s, SpriteStorage>>,
        Entities<'s>,
        Read<'s, Transform>,
    );
    fn run(&mut self, (fps, sprites, entities, transforms): Self::SystemData) {
        amethyst_imgui::with(|ui: &imgui::Ui| {
            let mut window = imgui::Window::new(im_str!("Test"));
            window.build(ui, || {
                ui.text(im_str!("This is a test!"));
                ui.text(im_str!("FPS: {}", fps.sampled_fps()));
            });
        });
    }
}

struct GameplayState {
    assets: GameAssets,
    stage_desc: StageDescription,
}
impl SimpleState for GameplayState {
    fn on_start(&mut self, mut data: StateData<'_, GameData<'_, '_>>) {
        data.world.delete_all();
        data.world.insert(self.assets.0.clone());
        data.world.insert(self.assets.1.clone());
        data.world.insert(self.assets.2.clone());

        let dimensions = (*data.world.read_resource::<ScreenDimensions>()).clone();

        initialize_camera(&mut data.world, &dimensions);
        initialize_stage(&mut data.world, self.stage_desc.clone());
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        /*let (entities, names): (Entities<'_>, ReadStorage<'_, Named>) = data.world.system_data();
        if get_named_entity(&entities, &names, "player").is_none() {
            return SimpleTrans::Switch(Box::new(MenuState {
                assets: self.assets.clone(),
                menu: "game_over.ron",
            }));
        }
        if get_named_entity(&entities, &names, "pylon").is_none() {
            return SimpleTrans::Switch(Box::new(MenuState {
                assets: self.assets.clone(),
                menu: "game_over.ron",
            }));
        }
        if data.world.read_resource::<WaveState>().wave_num == SPAWNS.len() {
            return SimpleTrans::Switch(Box::new(MenuState {
                assets: self.assets.clone(),
                menu: "game_over.ron",
            }));
        }*/
        SimpleTrans::None
    }
}

struct MenuState {
    assets: GameAssets,
    menu: &'static str,
}
impl SimpleState for MenuState {
    fn on_start(&mut self, mut data: StateData<'_, GameData<'_, '_>>) {
        data.world.delete_all();
        data.world.exec(|mut creator: UiCreator<'_>| {
            creator.create(format!("{}", self.menu), ());
        });
    }

    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        match &event {
            StateEvent::Window(event) => {
                if is_close_requested(&event) {
                    Trans::Quit
                } else {
                    Trans::None
                }
            }
            StateEvent::Ui(ui_event) => data.world.exec(|finder: UiFinder<'_>| {
                if ui_event.event_type == UiEventType::Click {
                    if let Some(start) = finder.find("play") {
                        if start == ui_event.target {
                            return Trans::Push(Box::new(GameplayState {
                                assets: self.assets.clone(),
                                stage_desc: StageDescription::default(),
                            }));
                        }
                    }
                    if let Some(exit) = finder.find("exit") {
                        if exit == ui_event.target {
                            return Trans::Quit;
                        }
                    }
                }
                Trans::None
            }),
            _ => Trans::None,
        }
    }
}

#[derive(Default)]
struct LoadingState {
    progress: Option<ProgressCounter>,
    assets: Option<GameAssets>,
}

impl SimpleState for LoadingState {
    fn on_start(&mut self, mut data: StateData<'_, GameData<'_, '_>>) {
        //        data.world.register::<PhysicsHandle>();
        //        data.world.insert(AssetStorage::<TiledMap>::default());

        println!("Starting loading");
        init_output(data.world);
        let mut progress_counter = ProgressCounter::new();

        let tiles = load_spritesheet(data.world, "Tiles".to_string(), &mut progress_counter);

        let player = load_prefab(data.world, "Player.ron".to_string(), &mut progress_counter);
        let notes = load_prefab(data.world, "Notes.ron".to_string(), &mut progress_counter);
        let ball = load_prefab(
            data.world,
            "BallDrop.ron".to_string(),
            &mut progress_counter,
        );
        let shadow = load_prefab(data.world, "Shadow.ron".to_string(), &mut progress_counter);
        let platform = load_prefab(data.world, "Drops.ron".to_string(), &mut progress_counter);

        let jump = load_sound_file(data.world, "hup.wav".to_string(), &mut progress_counter);
        let tap = load_sound_file(data.world, "tap.wav".to_string(), &mut progress_counter);

        let foo_scale = SCALE
            .iter()
            .map(|note| {
                load_sound_file(
                    data.world,
                    format!("foo/{}.wav", note),
                    &mut progress_counter,
                )
            })
            .collect::<Vec<SourceHandle>>();

        self.progress = Some(progress_counter);
        self.assets = Some((
            SpriteStorage { tiles },
            PrefabStorage {
                ball,
                shadow,
                notes,
                platform,
                player,
            },
            SoundStorage {
                jump,
                tap,
                foo_scale,
            },
        ));
    }

    fn update(&mut self, data: &mut StateData<GameData>) -> SimpleTrans {
        if let Some(progress) = &self.progress {
            if progress.errors().len() > 0 {
                println!("{:?}", progress);
                return SimpleTrans::Quit;
            }
            if progress.is_complete() {
                return SimpleTrans::Switch(Box::new(GameplayState {
                    stage_desc: StageDescription::default(),
                    assets: self.assets.clone().unwrap(),
                }));
                /*return SimpleTrans::Switch(Box::new(MenuState {
                    assets: self.assets.clone().unwrap(),
                    menu: "main_menu.ron",
                }));*/
            }
        }
        SimpleTrans::None
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;
    let resources = app_root.join("assets");

    let display_config_path = "assets/display_config.ron";
    let input_path = "assets/input.ron";

    let game_data = GameDataBuilder::default()
        .with_system_desc(
            PrefabLoaderSystemDesc::<SpriteEntityPrefabData>::default(),
            "scene_loader",
            &[],
        )
        .with_bundle(AnimationBundle::<AnimationId, Transform>::new(
            "animation_control_system",
            "sampler_interpolation_system",
        ))?
        .with_bundle(AnimationBundle::<AnimationId, SpriteRender>::new(
            "sprite_animation_control",
            "sprite_sampler_interpolation",
        ))?
        //.with(DjSystem, "dj", &[])
        .with_bundle(TransformBundle::new().with_dep(&[]))?
        .with_bundle(
            amethyst::input::InputBundle::<amethyst::input::StringBindings>::new()
                .with_bindings_from_file(input_path)?,
        )?
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?
                        .with_clear([0.0, 0.0, 0.0, 1.0]),
                )
                .with_plugin(RenderFlat2D::default())
                .with_plugin(RenderDebugLines::default())
                .with_plugin(RenderUi::default())
                .with_plugin(RenderImgui::<amethyst::input::StringBindings>::default()),
        )?
        .with_bundle(AudioBundle::default())?
        .with_bundle(FpsCounterBundle)?
        .with_bundle(PlayerBundle)?
        .with_bundle(StageBundle)?
        .with_bundle(HazardsBundle)?
        .with_bundle(UiBundle::<amethyst::input::StringBindings>::new())?
        //.with(DebugDrawShapes, "debug_shapes", &[])
        ;

    let mut game = Application::new(resources, LoadingState::default(), game_data)?;
    game.run();

    Ok(())
}
struct DebugDrawShapes;

impl<'s> System<'s> for DebugDrawShapes {
    type SystemData = (Write<'s, DebugLines>, ReadStorage<'s, Transform>);

    fn run(&mut self, (mut debugLines, transforms): Self::SystemData) {
        for (transform) in (&transforms).join() {
            debugLines.draw_circle(
                na19::geometry::Point3::<f32>::new(
                    transform.translation().x,
                    transform.translation().y,
                    0.0,
                ),
                8.0,
                16,
                Srgba::new(1.0, 1.0, 1.0, 1.0),
            );
        }
    }
}
