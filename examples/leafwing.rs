use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_touch_stick::{prelude::*, TouchStickUiKnob, TouchStickUiOutline};
use leafwing_input_manager::prelude::*;

/// Marker type for our touch stick
#[derive(Default, Reflect, Hash, Clone, PartialEq, Eq)]
enum Stick {
    #[default]
    Left,
    Right,
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
enum Action {
    Move,
    Look,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((
            DefaultPlugins,
            // add an inspector for easily changing settings at runtime
            WorldInspectorPlugin::default(),
            // add the plugin
            TouchStickPlugin::<Stick>::default(),
            // add leafwing plugin
            InputManagerPlugin::<Action>::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, move_player)
        .run();
}

#[derive(Component)]
struct Player {
    max_speed: f32,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(0., 0., 5.0),
        ..default()
    });

    // spawn a player
    commands
        .spawn((
            Player { max_speed: 50. },
            InputManagerBundle::<Action> {
                // Stores "which actions are currently activated"
                action_state: ActionState::default(),
                // Describes how to convert from player inputs into those actions
                input_map: InputMap::default()
                    .insert(Action::Move, DualAxis::left_stick())
                    .insert(Action::Look, DualAxis::right_stick())
                    .build(),
            },
            SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(0., 0., 0.),
                    ..default()
                },
                sprite: Sprite {
                    color: Color::ORANGE,
                    custom_size: Some(Vec2::new(30., 50.)),
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            // pointy "nose" for player
            parent.spawn(SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(15., 0., 0.),
                    rotation: Quat::from_rotation_z(PI / 4.),
                    ..default()
                },
                sprite: Sprite {
                    color: Color::ORANGE,
                    custom_size: Some(Vec2::splat(50. / f32::sqrt(2.))),
                    ..default()
                },
                ..default()
            });
        });

    // spawn a move stick
    commands
        .spawn((
            // map this stick as a left gamepad stick (through bevy_input)
            // leafwing will register this as a normal gamepad
            TouchStickGamepadMapping::LEFT_STICK,
            TouchStickUiBundle {
                stick: TouchStick {
                    id: Stick::Left,
                    stick_type: TouchStickType::Fixed,
                    ..default()
                },
                // configure the interactable area through bevy_ui
                style: Style {
                    width: Val::Px(150.),
                    height: Val::Px(150.),
                    position_type: PositionType::Absolute,
                    left: Val::Percent(15.),
                    bottom: Val::Percent(5.),
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                TouchStickUiKnob,
                ImageBundle {
                    image: asset_server.load("knob.png").into(),
                    style: Style {
                        width: Val::Px(75.),
                        height: Val::Px(75.),
                        ..default()
                    },
                    ..default()
                },
            ));
            parent.spawn((
                TouchStickUiOutline,
                ImageBundle {
                    image: asset_server.load("outline.png").into(),
                    style: Style {
                        width: Val::Px(150.),
                        height: Val::Px(150.),
                        ..default()
                    },
                    ..default()
                },
            ));
        });

    // spawn a look stick
    commands
        .spawn((
            // map this stick as a right gamepad stick (through bevy_input)
            // leafwing will register this as a normal gamepad
            TouchStickGamepadMapping::RIGHT_STICK,
            TouchStickUiBundle {
                stick: TouchStick {
                    id: Stick::Right,
                    stick_type: TouchStickType::Floating,
                    ..default()
                },
                // configure the interactable area through bevy_ui
                style: Style {
                    width: Val::Px(150.),
                    height: Val::Px(150.),
                    position_type: PositionType::Absolute,
                    right: Val::Percent(15.),
                    bottom: Val::Percent(5.),
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                TouchStickUiKnob,
                ImageBundle {
                    image: asset_server.load("knob.png").into(),
                    style: Style {
                        width: Val::Px(75.),
                        height: Val::Px(75.),
                        ..default()
                    },
                    ..default()
                },
            ));
            parent.spawn((
                TouchStickUiOutline,
                ImageBundle {
                    image: asset_server.load("outline.png").into(),
                    style: Style {
                        width: Val::Px(150.),
                        height: Val::Px(150.),
                        ..default()
                    },
                    ..default()
                },
            ));
        });
}

fn move_player(
    mut players: Query<(&mut Transform, &ActionState<Action>, &Player)>,
    time: Res<Time>,
) {
    let (mut player_transform, action_state, player) = players.single_mut();

    if action_state.pressed(&Action::Move) {
        let axis_value = action_state.clamped_axis_pair(&Action::Move).unwrap().xy();

        info!("moving: {axis_value}");

        let move_delta = axis_value * player.max_speed * time.delta_seconds();
        player_transform.translation += move_delta.extend(0.);
    }

    if action_state.pressed(&Action::Look) {
        let axis_value = action_state.clamped_axis_pair(&Action::Look).unwrap().xy();

        if axis_value != Vec2::ZERO {
            let dir = Vec2::angle_between(Vec2::X, axis_value.normalize());
            player_transform.rotation = Quat::from_rotation_z(dir);
        }
    }
}
