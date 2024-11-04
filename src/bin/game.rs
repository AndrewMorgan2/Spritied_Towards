// In src/bin/game.rs
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup_game)
        .add_systems(Update, game_logic)
        .run();
}

fn setup_game(mut commands: Commands) {
    // Initialize your game here
    commands.spawn(Camera2dBundle::default());
    
    // Add your game's initial entities
    commands.spawn(
        TextBundle::from_section(
            "Game Started!",
            TextStyle {
                font_size: 60.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            align_self: AlignSelf::Center,
            margin: UiRect::all(Val::Px(50.0)),
            ..default()
        }),
    );
}

fn game_logic() {
    // Add your game logic here
}