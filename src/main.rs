use core::f64::consts::TAU;
use std::path::Path;

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy_motiongfx::manager::TimelineComplete;
use bevy_motiongfx::{BevyMotionGfxPlugin, MotionGfxSet, prelude::*};
use bevy_vello::{VelloPlugin, prelude::*};
use velyst::{VelystPlugin, prelude::*};

use pipelines_ready::*;

const EXIT_TIME: f32 = 0.5;
const SAVE_DIR: &str = "frames/";

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins,
            VelloPlugin::default(),
            VelystPlugin,
            BevyMotionGfxPlugin,
            PipelinesReadyPlugin,
        ))
        .register_typst_func::<WaveFunc>()
        .configure_sets(
            PostUpdate,
            (MotionGfxSet::Controller, MotionGfxSet::Sample)
                .chain()
                .before(VelystSet::PrepareFunc),
        )
        .add_systems(PostUpdate, sync_wave.in_set(VelystSet::PrepareFunc))
        .add_systems(Startup, (setup, build_timeline, setup_save_path).chain())
        .add_systems(OnEnter(PipelineState::Ready), start_recording)
        .add_systems(Update, screenshot.run_if(in_state(PipelineState::Ready)))
        .add_systems(Update, check_final_frame)
        .insert_resource(ExitDelayTimer(Timer::from_seconds(
            EXIT_TIME,
            TimerMode::Once,
        )))
        .run()
}

#[derive(Component, Default)]
struct WaveTime {
    t: f64,
}

fn sync_wave(mut q: Query<(&WaveTime, &mut VelystFunc<WaveFunc>)>) {
    for (time, mut func) in q.iter_mut() {
        func.data.animate = time.t;
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: Color::BLACK.into(),
            ..default()
        },
        VelloView,
    ));

    commands.spawn((
        WaveTime::default(),
        VelystFunc::new(
            asset_server.load("typst/hello_world.typ"),
            WaveFunc::default(),
        ),
        UiScene,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
    ));
}

fn build_timeline(
    mut commands: Commands,
    mut motiongfx: ResMut<MotionGfxManager>,
    entity: Single<Entity, With<WaveTime>>,
) {
    let mut b = motiongfx.create_builder();

    let slide0_frag = b.act(*entity, path!(<WaveTime>::t), |_| TAU).play(4.0);
    let slide1_frag = b.act(*entity, path!(<WaveTime>::t), |_| 0.0).play(2.0);

    let track = [slide0_frag, slide1_frag].ord_chain().compile();
    b.add_tracks(track);
    let timeline = b.compile();

    commands.spawn((motiongfx.add_timeline(timeline), FixedRatePlayer::new(60)));
}

fn start_recording(mut player: Single<&mut FixedRatePlayer>) {
    player.set_playing(true);
}

fn screenshot(mut commands: Commands, player: Single<&FixedRatePlayer, Without<TimelineComplete>>) {
    if !player.is_playing {
        return;
    }

    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(format!(
            "{}frame_{:05}.png",
            SAVE_DIR, player.curr_frame
        )));
}

#[derive(Resource)]
struct ExitDelayTimer(Timer);

fn check_final_frame(
    incomplete_players: Query<&FixedRatePlayer, Without<TimelineComplete>>,
    mut exit_timer: ResMut<ExitDelayTimer>,
    time: Res<Time>,
    mut app_exit: MessageWriter<AppExit>,
) {
    if incomplete_players.is_empty() && exit_timer.0.tick(time.delta()).is_finished() {
        app_exit.write(AppExit::Success);
    }
}

fn setup_save_path() {
    let path = Path::new(SAVE_DIR);
    if !path.exists() {
        std::fs::create_dir_all(path).expect("Should have been able to create frame directory");
    }
}

typst_func!(
    "main",
    #[derive(Default)]
    struct WaveFunc {},
    positional_args { animate: f64 },
);

mod pipelines_ready {
    use bevy::{
        prelude::*,
        render::{render_resource::*, *},
    };

    #[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum PipelineState {
        #[default]
        Loading,
        Ready,
    }

    pub struct PipelinesReadyPlugin;
    impl Plugin for PipelinesReadyPlugin {
        fn build(&self, app: &mut App) {
            app.init_state::<PipelineState>();
            app.sub_app_mut(RenderApp)
                .add_systems(ExtractSchedule, update_pipelines_ready);
        }
    }

    fn update_pipelines_ready(mut main_world: ResMut<MainWorld>, pipelines: Res<PipelineCache>) {
        let curr_state = main_world.resource::<State<PipelineState>>();
        if *curr_state.get() == PipelineState::Ready {
            return;
        }

        let mut state = main_world.resource_mut::<NextState<PipelineState>>();

        if pipelines.pipelines().count() > 0 && pipelines.waiting_pipelines().count() == 0 {
            state.set(PipelineState::Ready);
        }
    }
}
