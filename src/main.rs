use bevy::{color::palettes::css, prelude::*};
use bevy_rapier2d::prelude::*;

use crate::player::{Player, PlayerClass, SelectedClass};

mod player;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .init_state::<AppState>() // Alternatively we could use .insert_state(AppState::Menu)
        .insert_resource(SelectedClass(None))
        .add_systems(Startup, setup)
        // This system runs when we enter `AppState::Menu`, during the `StateTransition` schedule.
        // All systems from the exit schedule of the state we're leaving are run first,
        // and then all systems from the enter schedule of the state we're entering are run second.
        .add_systems(OnEnter(AppState::Menu), setup_menu)
        // By contrast, update systems are stored in the `Update` schedule. They simply
        // check the value of the `State<T>` resource to see if they should run each frame.
        .add_systems(Update, menu.run_if(in_state(AppState::Menu)))
        .add_systems(OnExit(AppState::Menu), cleanup_menu)
        .add_systems(OnEnter(AppState::InGame), setup_game)
        .add_systems(
            Update,
            (
                player::rotate_player_to_mouse,
                player::player_movement_input,
            )
                .run_if(in_state(AppState::InGame)),
        )
        .run();
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    Menu,
    InGame,
}

#[derive(Resource)]
struct MenuData {
    button_entity: Entity,
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn setup_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let button_font = asset_server.load("fonts/FiraSans-Bold.ttf"); // optional if you want custom font

    let container = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            for (class, label) in [
                (PlayerClass::Warrior, "Warrior"),
                (PlayerClass::Archer, "Archer"),
                (PlayerClass::Mage, "Mage"),
            ] {
                parent
                    .spawn((
                        ButtonBundle {
                            style: Style {
                                width: Val::Px(200.),
                                height: Val::Px(50.),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            background_color: NORMAL_BUTTON.into(),
                            ..default()
                        },
                        class, // Attach the enum directly as a component
                    ))
                    .with_children(|parent| {
                        parent.spawn(TextBundle::from_section(
                            label,
                            TextStyle {
                                font: button_font.clone(),
                                font_size: 30.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        ));
                    });
            }
        })
        .id();

    commands.insert_resource(MenuData {
        button_entity: container,
    });
}

fn menu(
    mut next_state: ResMut<NextState<AppState>>,
    mut selected_class: ResMut<SelectedClass>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &PlayerClass),
        Changed<Interaction>,
    >,
) {
    for (interaction, mut color, class) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                selected_class.0 = Some(*class); // Store selected class
                next_state.set(AppState::InGame);
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

fn cleanup_menu(mut commands: Commands, menu_data: Res<MenuData>) {
    commands.entity(menu_data.button_entity).despawn_recursive();
}

fn setup_game(
    mut commands: Commands,
    selected_class: Res<SelectedClass>,
    asset_server: Res<AssetServer>,
) {
    let Some(class) = selected_class.0 else {
        error!("No class selected!");
        return;
    };

    // Example: spawn colored square for now
    let (color, name) = match class {
        PlayerClass::Warrior => (css::RED, "Warrior"),
        PlayerClass::Archer => (css::GREEN, "Archer"),
        PlayerClass::Mage => (css::BLUE, "Mage"),
    };

    println!("Spawning player of class: {name}");

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::from(color),
                custom_size: Some(Vec2::splat(40.0)),
                ..default()
            },
            transform: Transform::from_xyz(0., 0., 0.),
            ..default()
        },
        Player,
        RigidBody::Dynamic,
        Collider::ball(20.0),
        GravityScale(0.0),
        Velocity::zero(),
        Damping {
            linear_damping: 5.0,
            ..default()
        },
    ));
}
