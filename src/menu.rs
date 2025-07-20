use bevy::prelude::*;

use crate::{AppState, PlayerClass, SelectedClass};

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .add_systems(OnEnter(AppState::Menu), setup_menu)
            .add_systems(Update, menu.run_if(in_state(AppState::Menu)))
            .add_systems(OnExit(AppState::Menu), cleanup_menu);
    }
}

#[derive(Resource)]
struct MenuData {
    root_entity: Entity,
}

fn setup_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");

    let root_entity = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            for (label, class) in [
                ("Warrior", PlayerClass::Warrior),
                ("Mage", PlayerClass::Mage),
                ("Ranger", PlayerClass::Ranger),
            ] {
                parent
                    .spawn((
                        ButtonBundle {
                            style: Style {
                                width: Val::Px(200.),
                                height: Val::Px(65.),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            background_color: NORMAL_BUTTON.into(),
                            ..default()
                        },
                        class,
                    ))
                    .with_children(|parent| {
                        parent.spawn(TextBundle::from_section(
                            label,
                            TextStyle {
                                font: font.clone(),
                                font_size: 30.,
                                color: Color::WHITE,
                            },
                        ));
                    });
            }
        })
        .id();

    commands.insert_resource(MenuData { root_entity });
}

fn menu(
    mut next_state: ResMut<NextState<AppState>>,
    mut selected_class: ResMut<SelectedClass>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &PlayerClass),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, class) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                selected_class.0 = Some(*class);
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
    commands.entity(menu_data.root_entity).despawn_recursive();
}
