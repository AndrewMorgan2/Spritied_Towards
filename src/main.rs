//! This example will display a simple menu using Bevy UI where you can start a new game,
//! change some settings or quit. There is no actual game, it will just display the current
//! settings for 5 seconds before going back to the menu.
// Add these imports at the top level
use bevy::audio::*;

// Create a component to mark the background music entity
#[derive(Component)]
struct BackgroundMusic;

// Create a plugin for the audio system
pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_background_music);
    }
}

// System to set up the background music
fn setup_background_music(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        AudioBundle {
            source: asset_server.load("sounds/Shadowy Whispers.ogg"), // Replace with your music file
            settings: PlaybackSettings {
                // volume: Volume::0.5, // Volume between 0.0 and 1.0
                mode: PlaybackMode::Loop,
                ..default()
            },
            ..default()
        },
        BackgroundMusic,
    ));
}

use bevy::prelude::*;
const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);

// Enum that will be used as a global state for the game
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum GameState {
    #[default]
    Splash,
    Menu,
    Game,
    Game2,
    Game3,
    Game4,
    Chapter1,
    Chapter2,
    Chapter3,
    Chapter4,
}

// One of the two settings that can be set through the menu. It will be a resource in the app
#[derive(Resource, Debug, Component, PartialEq, Eq, Clone, Copy)]
enum DisplayQuality {
    Low,
    Medium,
    High,
}

// One of the two settings that can be set through the menu. It will be a resource in the app
#[derive(Resource, Debug, Component, PartialEq, Eq, Clone, Copy)]
struct Volume(u32);

#[derive(Resource, Default)]
struct PendingAirCards {
    to_add: i32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AudioPlugin) // Add this line
        // Insert as resource the initial value for the settings resources
        .insert_resource(DisplayQuality::Medium)
        .insert_resource(Volume(7))
        .insert_resource(PendingAirCards::default()) // Add this line
        // Declare the game state, whose starting value is determined by the `Default` trait
        .init_state::<GameState>()
        .add_systems(Startup, setup)
        // Adds the plugins for each state
        .add_plugins((
            splash::splash_plugin,
            menu::menu_plugin,
            game::game_plugin,
            game2::game_plugin_2,
            game3::game_plugin_3,
            game4::game_plugin_3,
            chapter1::chapter1_plugin,
            chapter2::chapter2_plugin,
            chapter3::chapter3_plugin,
            chapter4::chapter3_plugin,
        ))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

mod splash {
    use bevy::prelude::*;

    use super::{despawn_screen, GameState};

    // This plugin will display a splash screen with Bevy logo for 1 second before switching to the menu
    pub fn splash_plugin(app: &mut App) {
        // As this plugin is managing the splash screen, it will focus on the state `GameState::Splash`
        app
            // When entering the state, spawn everything needed for this screen
            .add_systems(OnEnter(GameState::Splash), splash_setup)
            // While in this state, run the `countdown` system
            .add_systems(Update, countdown.run_if(in_state(GameState::Splash)))
            // When exiting the state, despawn everything that was spawned for this screen
            .add_systems(OnExit(GameState::Splash), despawn_screen::<OnSplashScreen>);
    }

    // Tag component used to tag entities added on the splash screen
    #[derive(Component)]
    struct OnSplashScreen;

    // Newtype to use a `Timer` for this screen as a resource
    #[derive(Resource, Deref, DerefMut)]
    struct SplashTimer(Timer);

    fn splash_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
        let icon = asset_server.load("branding/icon.png");
        // Display the logo
        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    ..default()
                },
                OnSplashScreen,
            ))
            .with_children(|parent| {
                parent.spawn(ImageBundle {
                    style: Style {
                        // This will set the logo to be 200px wide, and auto adjust its height
                        width: Val::Px(200.0),
                        ..default()
                    },
                    image: UiImage::new(icon),
                    ..default()
                });
            });
        // Insert the timer as a resource
        commands.insert_resource(SplashTimer(Timer::from_seconds(1.0, TimerMode::Once)));
    }

    // Tick the timer, and change state when finished
    fn countdown(
        mut game_state: ResMut<NextState<GameState>>,
        time: Res<Time>,
        mut timer: ResMut<SplashTimer>,
    ) {
        if timer.tick(time.delta()).finished() {
            game_state.set(GameState::Menu);
        }
    }
}

mod game {
    use super::{despawn_screen, DisplayQuality, GameState, Volume, TEXT_COLOR};
    use bevy::prelude::*;

    // Add this new resource to handle the custom font
    #[derive(Resource)]
    struct GameFont(Handle<Font>);

    #[derive(Component)]
    struct OnGameScreen;

    #[derive(Component)]
    struct AnimationTimer(Timer);

    #[derive(Component)]
    struct AnimationIndices {
        first: usize,
        last: usize,
    }

    #[derive(Component)]
    struct TextContainer;

    #[derive(Component)]
    struct TypingText {
        full_text: String,
        current_index: usize,
        timer: Timer,
        completed: bool,
    }

    #[derive(Component)]
    struct TextSequence {
        sequence_index: usize,
    }

    #[derive(Resource)]
    struct TextSequenceState {
        current_sequence: usize,
        texts: Vec<String>,
        delay_timer: Timer,
        ready_for_next: bool,
    }

    #[derive(Resource)]
    struct TypewriterSound(Handle<AudioSource>);

    #[derive(Resource, Deref, DerefMut)]
    struct GameTimer(Timer);

    fn game_setup(
        mut commands: Commands,
        display_quality: Res<DisplayQuality>,
        volume: Res<Volume>,
        asset_server: Res<AssetServer>,
        mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
        windows: Query<&Window>,
    ) {
        let window = windows.single();

        // Load custom font and create resource
        let custom_font = asset_server.load("joystix monospace.otf");
        commands.insert_resource(GameFont(custom_font));
        commands.init_resource::<GameSetupComplete>();

        // Load typewriter sound
        let typewriter_sound = asset_server.load("sounds/typewriter.ogg");
        commands.insert_resource(TypewriterSound(typewriter_sound));

        // Initialize text sequence
        commands.insert_resource(TextSequenceState {
            current_sequence: 0,
            texts: vec![
                "  ".to_string(),
                "Strange... the patterns are shifting...".to_string(),
                "You awake in a magic forest".to_string(),
                "Something breaks a twig nearby...".to_string(),
            ],
            delay_timer: Timer::from_seconds(4.0, TimerMode::Once), // 4 second delay between texts
            ready_for_next: true,
        });

        // Load the sprite sheet
        let texture_handle = asset_server.load("textures/intro_game_sprite.png");

        let layout = TextureAtlasLayout::from_grid(UVec2::new(576, 324), 5, 64, None, None);

        let atlas_layout = atlas_layouts.add(layout);

        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                    ..default()
                },
                OnGameScreen,
            ))
            .with_children(|parent| {
                // Text container at the bottom
                parent.spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Percent(20.0),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::FlexEnd,
                            padding: UiRect::all(Val::Px(20.0)),
                            position_type: PositionType::Absolute,
                            bottom: Val::Px(0.0),
                            ..default()
                        },
                        background_color: Color::srgba(0.0, 0.0, 0.0, 0.7).into(),
                        ..default()
                    },
                    TextContainer,
                ));

                // Sprite container
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            width: Val::Vw(100.0),
                            height: Val::Vh(100.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|parent| {
                        parent.spawn((
                            SpriteSheetBundle {
                                texture: texture_handle,
                                atlas: TextureAtlas {
                                    layout: atlas_layout,
                                    index: 0,
                                },
                                transform: Transform::from_xyz(
                                    -window.width() / 2.0,
                                    -window.height() / 2.0 + 60.0,
                                    1.0,
                                ),
                                sprite: Sprite {
                                    custom_size: Some(Vec2::new(1920.0, 1080.0)),
                                    anchor: bevy::sprite::Anchor::Center,
                                    ..default()
                                },
                                ..default()
                            },
                            AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)), // for (entity, _, sequence) in typing_query.iter() {
                            //     if sequence.sequence_index <= sequence_state.current_sequence {
                            //         commands.entity(entity).despawn();
                            //     }
                            // }
                            AnimationIndices {
                                first: 0,
                                last: 320,
                            },
                        ));
                    });
            });

        commands.insert_resource(GameTimer(Timer::from_seconds(14.0, TimerMode::Once)));
        // Immediately despawn all previous text when we're ready for the next one
        // for (entity, _, sequence) in typing_query.iter() {
        //     if sequence.sequence_index <= sequence_state.current_sequence {
        //         commands.entity(entity).despawn();
        //     }
        // }
        // make liek 20 s when proper

        // We can't spawn the text in game_setup because we need to wait for the GameFont resource to be available
        // Instead, we'll create a new system to handle the initial text spawn
    }

    // Add marker resource to ensure proper setup order
    #[derive(Resource, Default)]
    struct GameSetupComplete;

    // Modify the spawn_initial_text system to run after setup is complete
    fn spawn_initial_text(
        mut commands: Commands,
        game_font: Res<GameFont>,
        setup_complete: Res<GameSetupComplete>,
    ) {
        spawn_text_entity(&mut commands, 0, &game_font);
    }

    // Update the game plugin with proper system ordering
    pub fn game_plugin(app: &mut App) {
        app.init_resource::<GameSetupComplete>() // Initialize the marker resource
            .add_systems(OnEnter(GameState::Game), game_setup)
            .add_systems(
                OnEnter(GameState::Game),
                spawn_initial_text.after(game_setup),
            )
            .add_systems(
                Update,
                (game, animate_sprite, manage_text_sequence, type_text)
                    .run_if(in_state(GameState::Game)),
            )
            .add_systems(OnExit(GameState::Game), despawn_screen::<OnGameScreen>);
    }

    // Make sure spawn_text_entity uses the correct font parameter
    fn spawn_text_entity(
        commands: &mut Commands,
        sequence_index: usize,
        game_font: &Res<GameFont>,
    ) {
        commands.spawn((
            TextBundle::from_section(
                "",
                TextStyle {
                    font: game_font.0.clone(),
                    font_size: 40.0,
                    color: Color::WHITE,
                    ..default()
                },
            )
            .with_style(Style {
                margin: UiRect::bottom(Val::Px(20.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                bottom: Val::Px(80.0),
                left: Val::Px(20.0),
                right: Val::Px(20.0),
                ..default()
            }),
            TypingText {
                full_text: String::new(),
                current_index: 0,
                timer: Timer::from_seconds(0.08, TimerMode::Repeating),
                completed: false,
            },
            TextSequence { sequence_index },
        ));
    }

    fn manage_text_sequence(
        mut commands: Commands,
        mut sequence_state: ResMut<TextSequenceState>,
        time: Res<Time>,
        typing_query: Query<(Entity, &TypingText, &TextSequence)>,
        game_font: Res<GameFont>,
    ) {
        if !sequence_state.ready_for_next {
            sequence_state.delay_timer.tick(time.delta());
            if sequence_state.delay_timer.finished() {
                sequence_state.ready_for_next = true;
                // Immediately despawn all previous text when we're ready for the next one
                for (entity, _, sequence) in typing_query.iter() {
                    if sequence.sequence_index <= sequence_state.current_sequence {
                        commands.entity(entity).despawn();
                    }
                }
            }
            return;
        }

        let mut all_completed = true;
        for (_, typing_text, sequence) in typing_query.iter() {
            if sequence.sequence_index == sequence_state.current_sequence {
                if !typing_text.completed {
                    all_completed = false;
                    break;
                }
            }
        }

        if all_completed && sequence_state.current_sequence < sequence_state.texts.len() - 1 {
            // Clear any existing text before spawning new one
            for (entity, _, _) in typing_query.iter() {
                commands.entity(entity).despawn();
            }

            sequence_state.current_sequence += 1;
            sequence_state.ready_for_next = false;
            sequence_state.delay_timer.reset();

            spawn_text_entity(&mut commands, sequence_state.current_sequence, &game_font);
        }
    }

    fn type_text(
        time: Res<Time>,
        sequence_state: Res<TextSequenceState>,
        mut query: Query<(&mut TypingText, &mut Text, &TextSequence)>,
        typewriter_sound: Res<TypewriterSound>,
        mut commands: Commands,
    ) {
        for (mut typing_text, mut text, sequence) in query.iter_mut() {
            if typing_text.completed || sequence.sequence_index != sequence_state.current_sequence {
                continue;
            }

            if typing_text.full_text.is_empty() {
                typing_text.full_text = sequence_state.texts[sequence.sequence_index].clone();
            }

            typing_text.timer.tick(time.delta());

            if typing_text.timer.just_finished()
                && typing_text.current_index < typing_text.full_text.len()
            {
                let next_char = typing_text
                    .full_text
                    .chars()
                    .nth(typing_text.current_index)
                    .unwrap();
                typing_text.current_index += 1;

                if let Some(section) = text.sections.first_mut() {
                    section.value = typing_text.full_text[..typing_text.current_index].to_string();
                }

                if next_char != ' ' {
                    commands.spawn(AudioBundle {
                        source: typewriter_sound.0.clone(),
                        settings: PlaybackSettings::DESPAWN,
                        ..default()
                    });
                }

                if typing_text.current_index == typing_text.full_text.len() {
                    typing_text.completed = true;
                }
            }
        }
    }

    fn game(
        time: Res<Time>,
        mut game_state: ResMut<NextState<GameState>>,
        mut timer: ResMut<GameTimer>,
    ) {
        if timer.tick(time.delta()).finished() {
            game_state.set(GameState::Chapter1);
        }
    }

    fn animate_sprite(
        time: Res<Time>,
        mut query: Query<(&mut TextureAtlas, &mut AnimationTimer, &AnimationIndices)>,
    ) {
        for (mut atlas, mut timer, indices) in &mut query {
            timer.0.tick(time.delta());
            if timer.0.just_finished() {
                atlas.index = if atlas.index == indices.last {
                    indices.first
                } else {
                    atlas.index + 1
                };
            }
        }
    }
}

mod game2 {
    use super::{despawn_screen, DisplayQuality, GameState, Volume, TEXT_COLOR};
    use bevy::prelude::*;

    // Add this new resource to handle the custom font
    #[derive(Resource)]
    struct GameFont(Handle<Font>);

    #[derive(Component)]
    struct OnGameScreen;

    #[derive(Component)]
    struct AnimationTimer(Timer);

    #[derive(Component)]
    struct AnimationIndices {
        first: usize,
        last: usize,
    }

    #[derive(Component)]
    struct TextContainer;

    #[derive(Component)]
    struct TypingText {
        full_text: String,
        current_index: usize,
        timer: Timer,
        completed: bool,
    }

    #[derive(Component)]
    struct TextSequence {
        sequence_index: usize,
    }

    #[derive(Resource)]
    struct TextSequenceState {
        current_sequence: usize,
        texts: Vec<String>,
        delay_timer: Timer,
        ready_for_next: bool,
    }

    #[derive(Resource)]
    struct TypewriterSound(Handle<AudioSource>);

    #[derive(Resource, Deref, DerefMut)]
    struct GameTimer(Timer);

    fn game_setup2(
        mut commands: Commands,
        display_quality: Res<DisplayQuality>,
        volume: Res<Volume>,
        asset_server: Res<AssetServer>,
        mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
        windows: Query<&Window>,
    ) {
        let window = windows.single();

        // Load custom font and create resource
        let custom_font = asset_server.load("joystix monospace.otf");
        commands.insert_resource(GameFont(custom_font));
        commands.init_resource::<GameSetupComplete>();

        // Load typewriter sound
        let typewriter_sound = asset_server.load("sounds/typewriter.ogg");
        commands.insert_resource(TypewriterSound(typewriter_sound));

        // Initialize text sequence
        commands.insert_resource(TextSequenceState {
            current_sequence: 0,
            texts: vec![
                "  ".to_string(),
                "As you walk you come across a fort...".to_string(),
                "The door shudders in the wind".to_string(),
                "Suddenly the door swings open...".to_string(),
            ],
            delay_timer: Timer::from_seconds(4.0, TimerMode::Once), // 4 second delay between texts
            ready_for_next: true,
        });

        // Load the sprite sheet
        let texture_handle = asset_server.load("textures/forest_fort.png");

        let layout = TextureAtlasLayout::from_grid(UVec2::new(576, 324), 5, 64, None, None);

        let atlas_layout = atlas_layouts.add(layout);

        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                    ..default()
                },
                OnGameScreen,
            ))
            .with_children(|parent| {
                // Text container at the bottom
                parent.spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Percent(20.0),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::FlexEnd,
                            padding: UiRect::all(Val::Px(20.0)),
                            position_type: PositionType::Absolute,
                            bottom: Val::Px(0.0),
                            ..default()
                        },
                        background_color: Color::srgba(0.0, 0.0, 0.0, 0.7).into(),
                        ..default()
                    },
                    TextContainer,
                ));

                // Sprite container
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            width: Val::Vw(100.0),
                            height: Val::Vh(100.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|parent| {
                        parent.spawn((
                            SpriteSheetBundle {
                                texture: texture_handle,
                                atlas: TextureAtlas {
                                    layout: atlas_layout,
                                    index: 0,
                                },
                                transform: Transform::from_xyz(
                                    -window.width() / 2.0,
                                    -window.height() / 2.0 + 60.0,
                                    1.0,
                                ),
                                sprite: Sprite {
                                    custom_size: Some(Vec2::new(1920.0, 1080.0)),
                                    anchor: bevy::sprite::Anchor::Center,
                                    ..default()
                                },
                                ..default()
                            },
                            AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)), // for (entity, _, sequence) in typing_query.iter() {
                            //     if sequence.sequence_index <= sequence_state.current_sequence {
                            //         commands.entity(entity).despawn();
                            //     }
                            // }
                            AnimationIndices {
                                first: 0,
                                last: 320,
                            },
                        ));
                    });
            });

        commands.insert_resource(GameTimer(Timer::from_seconds(14.0, TimerMode::Once)));
        // Immediately despawn all previous text when we're ready for the next one
        // for (entity, _, sequence) in typing_query.iter() {
        //     if sequence.sequence_index <= sequence_state.current_sequence {
        //         commands.entity(entity).despawn();
        //     }
        // }
        // make liek 20 s when proper

        // We can't spawn the text in game_setup because we need to wait for the GameFont resource to be available
        // Instead, we'll create a new system to handle the initial text spawn
    }

    // Add marker resource to ensure proper setup order
    #[derive(Resource, Default)]
    struct GameSetupComplete;

    // Modify the spawn_initial_text system to run after setup is complete
    fn spawn_initial_text(
        mut commands: Commands,
        game_font: Res<GameFont>,
        setup_complete: Res<GameSetupComplete>,
    ) {
        spawn_text_entity(&mut commands, 0, &game_font);
    }

    // Update the game plugin with proper system ordering
    pub fn game_plugin_2(app: &mut App) {
        app.init_resource::<GameSetupComplete>() // Initialize the marker resource
            .add_systems(OnEnter(GameState::Game2), game_setup2)
            .add_systems(
                OnEnter(GameState::Game2),
                spawn_initial_text.after(game_setup2),
            )
            .add_systems(
                Update,
                (game2, animate_sprite, manage_text_sequence, type_text)
                    .run_if(in_state(GameState::Game2)),
            )
            .add_systems(OnExit(GameState::Game2), despawn_screen::<OnGameScreen>);
    }

    // Make sure spawn_text_entity uses the correct font parameter
    fn spawn_text_entity(
        commands: &mut Commands,
        sequence_index: usize,
        game_font: &Res<GameFont>,
    ) {
        commands.spawn((
            TextBundle::from_section(
                "",
                TextStyle {
                    font: game_font.0.clone(),
                    font_size: 40.0,
                    color: Color::WHITE,
                    ..default()
                },
            )
            .with_style(Style {
                margin: UiRect::bottom(Val::Px(20.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                bottom: Val::Px(80.0),
                left: Val::Px(20.0),
                right: Val::Px(20.0),
                ..default()
            }),
            TypingText {
                full_text: String::new(),
                current_index: 0,
                timer: Timer::from_seconds(0.08, TimerMode::Repeating),
                completed: false,
            },
            TextSequence { sequence_index },
        ));
    }

    fn manage_text_sequence(
        mut commands: Commands,
        mut sequence_state: ResMut<TextSequenceState>,
        time: Res<Time>,
        typing_query: Query<(Entity, &TypingText, &TextSequence)>,
        game_font: Res<GameFont>,
    ) {
        if !sequence_state.ready_for_next {
            sequence_state.delay_timer.tick(time.delta());
            if sequence_state.delay_timer.finished() {
                sequence_state.ready_for_next = true;
                // Immediately despawn all previous text when we're ready for the next one
                for (entity, _, sequence) in typing_query.iter() {
                    if sequence.sequence_index <= sequence_state.current_sequence {
                        commands.entity(entity).despawn();
                    }
                }
            }
            return;
        }

        let mut all_completed = true;
        for (_, typing_text, sequence) in typing_query.iter() {
            if sequence.sequence_index == sequence_state.current_sequence {
                if !typing_text.completed {
                    all_completed = false;
                    break;
                }
            }
        }

        if all_completed && sequence_state.current_sequence < sequence_state.texts.len() - 1 {
            // Clear any existing text before spawning new one
            for (entity, _, _) in typing_query.iter() {
                commands.entity(entity).despawn();
            }

            sequence_state.current_sequence += 1;
            sequence_state.ready_for_next = false;
            sequence_state.delay_timer.reset();

            spawn_text_entity(&mut commands, sequence_state.current_sequence, &game_font);
        }
    }

    fn type_text(
        time: Res<Time>,
        sequence_state: Res<TextSequenceState>,
        mut query: Query<(&mut TypingText, &mut Text, &TextSequence)>,
        typewriter_sound: Res<TypewriterSound>,
        mut commands: Commands,
    ) {
        for (mut typing_text, mut text, sequence) in query.iter_mut() {
            if typing_text.completed || sequence.sequence_index != sequence_state.current_sequence {
                continue;
            }

            if typing_text.full_text.is_empty() {
                typing_text.full_text = sequence_state.texts[sequence.sequence_index].clone();
            }

            typing_text.timer.tick(time.delta());

            if typing_text.timer.just_finished()
                && typing_text.current_index < typing_text.full_text.len()
            {
                let next_char = typing_text
                    .full_text
                    .chars()
                    .nth(typing_text.current_index)
                    .unwrap();
                typing_text.current_index += 1;

                if let Some(section) = text.sections.first_mut() {
                    section.value = typing_text.full_text[..typing_text.current_index].to_string();
                }

                if next_char != ' ' {
                    commands.spawn(AudioBundle {
                        source: typewriter_sound.0.clone(),
                        settings: PlaybackSettings::DESPAWN,
                        ..default()
                    });
                }

                if typing_text.current_index == typing_text.full_text.len() {
                    typing_text.completed = true;
                }
            }
        }
    }

    fn game2(
        time: Res<Time>,
        mut game_state: ResMut<NextState<GameState>>,
        mut timer: ResMut<GameTimer>,
    ) {
        if timer.tick(time.delta()).finished() {
            game_state.set(GameState::Chapter2);
        }
    }

    fn animate_sprite(
        time: Res<Time>,
        mut query: Query<(&mut TextureAtlas, &mut AnimationTimer, &AnimationIndices)>,
    ) {
        for (mut atlas, mut timer, indices) in &mut query {
            timer.0.tick(time.delta());
            if timer.0.just_finished() {
                atlas.index = if atlas.index == indices.last {
                    indices.first
                } else {
                    atlas.index + 1
                };
            }
        }
    }
}

mod game3 {
    use super::{despawn_screen, DisplayQuality, GameState, Volume, TEXT_COLOR};
    use bevy::prelude::*;

    // Add this new resource to handle the custom font
    #[derive(Resource)]
    struct GameFont(Handle<Font>);

    #[derive(Component)]
    struct OnGameScreen;

    #[derive(Component)]
    struct AnimationTimer(Timer);

    #[derive(Component)]
    struct AnimationIndices {
        first: usize,
        last: usize,
    }

    #[derive(Component)]
    struct TextContainer;

    #[derive(Component)]
    struct TypingText {
        full_text: String,
        current_index: usize,
        timer: Timer,
        completed: bool,
    }

    #[derive(Component)]
    struct TextSequence {
        sequence_index: usize,
    }

    #[derive(Resource)]
    struct TextSequenceState {
        current_sequence: usize,
        texts: Vec<String>,
        delay_timer: Timer,
        ready_for_next: bool,
    }

    #[derive(Resource)]
    struct TypewriterSound(Handle<AudioSource>);

    #[derive(Resource, Deref, DerefMut)]
    struct GameTimer(Timer);

    fn game_setup3(
        mut commands: Commands,
        display_quality: Res<DisplayQuality>,
        volume: Res<Volume>,
        asset_server: Res<AssetServer>,
        mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
        windows: Query<&Window>,
    ) {
        let window = windows.single();

        // Load custom font and create resource
        let custom_font = asset_server.load("joystix monospace.otf");
        commands.insert_resource(GameFont(custom_font));
        commands.init_resource::<GameSetupComplete>();

        // Load typewriter sound
        let typewriter_sound = asset_server.load("sounds/typewriter.ogg");
        commands.insert_resource(TypewriterSound(typewriter_sound));

        // Initialize text sequence
        commands.insert_resource(TextSequenceState {
            current_sequence: 0,
            texts: vec![
                "  ".to_string(),
                "Clearing the fort you hear running water".to_string(),
                "Did the statue rotate...".to_string(),
                "It's probably an illusion...".to_string(),
            ],
            delay_timer: Timer::from_seconds(4.0, TimerMode::Once), // 4 second delay between texts
            ready_for_next: true,
        });

        // Load the sprite sheet
        let texture_handle = asset_server.load("textures/pool.png");

        let layout = TextureAtlasLayout::from_grid(UVec2::new(576, 324), 5, 64, None, None);

        let atlas_layout = atlas_layouts.add(layout);

        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                    ..default()
                },
                OnGameScreen,
            ))
            .with_children(|parent| {
                // Text container at the bottom
                parent.spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Percent(20.0),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::FlexEnd,
                            padding: UiRect::all(Val::Px(20.0)),
                            position_type: PositionType::Absolute,
                            bottom: Val::Px(0.0),
                            ..default()
                        },
                        background_color: Color::srgba(0.0, 0.0, 0.0, 0.7).into(),
                        ..default()
                    },
                    TextContainer,
                ));

                // Sprite container
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            width: Val::Vw(100.0),
                            height: Val::Vh(100.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|parent| {
                        parent.spawn((
                            SpriteSheetBundle {
                                texture: texture_handle,
                                atlas: TextureAtlas {
                                    layout: atlas_layout,
                                    index: 0,
                                },
                                transform: Transform::from_xyz(
                                    -window.width() / 2.0,
                                    -window.height() / 2.0 + 60.0,
                                    1.0,
                                ),
                                sprite: Sprite {
                                    custom_size: Some(Vec2::new(1920.0, 1080.0)),
                                    anchor: bevy::sprite::Anchor::Center,
                                    ..default()
                                },
                                ..default()
                            },
                            AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)), // for (entity, _, sequence) in typing_query.iter() {
                            //     if sequence.sequence_index <= sequence_state.current_sequence {
                            //         commands.entity(entity).despawn();
                            //     }
                            // }
                            AnimationIndices {
                                first: 0,
                                last: 320,
                            },
                        ));
                    });
            });

        commands.insert_resource(GameTimer(Timer::from_seconds(14.0, TimerMode::Once)));
        // Immediately despawn all previous text when we're ready for the next one
        // for (entity, _, sequence) in typing_query.iter() {
        //     if sequence.sequence_index <= sequence_state.current_sequence {
        //         commands.entity(entity).despawn();
        //     }
        // }
        // make liek 20 s when proper

        // We can't spawn the text in game_setup because we need to wait for the GameFont resource to be available
        // Instead, we'll create a new system to handle the initial text spawn
    }

    // Add marker resource to ensure proper setup order
    #[derive(Resource, Default)]
    struct GameSetupComplete;

    // Modify the spawn_initial_text system to run after setup is complete
    fn spawn_initial_text(
        mut commands: Commands,
        game_font: Res<GameFont>,
        setup_complete: Res<GameSetupComplete>,
    ) {
        spawn_text_entity(&mut commands, 0, &game_font);
    }

    // Update the game plugin with proper system ordering
    pub fn game_plugin_3(app: &mut App) {
        app.init_resource::<GameSetupComplete>() // Initialize the marker resource
            .add_systems(OnEnter(GameState::Game3), game_setup3)
            .add_systems(
                OnEnter(GameState::Game3),
                spawn_initial_text.after(game_setup3),
            )
            .add_systems(
                Update,
                (game3, animate_sprite, manage_text_sequence, type_text)
                    .run_if(in_state(GameState::Game3)),
            )
            .add_systems(OnExit(GameState::Game3), despawn_screen::<OnGameScreen>);
    }

    // Make sure spawn_text_entity uses the correct font parameter
    fn spawn_text_entity(
        commands: &mut Commands,
        sequence_index: usize,
        game_font: &Res<GameFont>,
    ) {
        commands.spawn((
            TextBundle::from_section(
                "",
                TextStyle {
                    font: game_font.0.clone(),
                    font_size: 40.0,
                    color: Color::WHITE,
                    ..default()
                },
            )
            .with_style(Style {
                margin: UiRect::bottom(Val::Px(20.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                bottom: Val::Px(80.0),
                left: Val::Px(20.0),
                right: Val::Px(20.0),
                ..default()
            }),
            TypingText {
                full_text: String::new(),
                current_index: 0,
                timer: Timer::from_seconds(0.08, TimerMode::Repeating),
                completed: false,
            },
            TextSequence { sequence_index },
        ));
    }

    fn manage_text_sequence(
        mut commands: Commands,
        mut sequence_state: ResMut<TextSequenceState>,
        time: Res<Time>,
        typing_query: Query<(Entity, &TypingText, &TextSequence)>,
        game_font: Res<GameFont>,
    ) {
        if !sequence_state.ready_for_next {
            sequence_state.delay_timer.tick(time.delta());
            if sequence_state.delay_timer.finished() {
                sequence_state.ready_for_next = true;
                // Immediately despawn all previous text when we're ready for the next one
                for (entity, _, sequence) in typing_query.iter() {
                    if sequence.sequence_index <= sequence_state.current_sequence {
                        commands.entity(entity).despawn();
                    }
                }
            }
            return;
        }

        let mut all_completed = true;
        for (_, typing_text, sequence) in typing_query.iter() {
            if sequence.sequence_index == sequence_state.current_sequence {
                if !typing_text.completed {
                    all_completed = false;
                    break;
                }
            }
        }

        if all_completed && sequence_state.current_sequence < sequence_state.texts.len() - 1 {
            // Clear any existing text before spawning new one
            for (entity, _, _) in typing_query.iter() {
                commands.entity(entity).despawn();
            }

            sequence_state.current_sequence += 1;
            sequence_state.ready_for_next = false;
            sequence_state.delay_timer.reset();

            spawn_text_entity(&mut commands, sequence_state.current_sequence, &game_font);
        }
    }

    fn type_text(
        time: Res<Time>,
        sequence_state: Res<TextSequenceState>,
        mut query: Query<(&mut TypingText, &mut Text, &TextSequence)>,
        typewriter_sound: Res<TypewriterSound>,
        mut commands: Commands,
    ) {
        for (mut typing_text, mut text, sequence) in query.iter_mut() {
            if typing_text.completed || sequence.sequence_index != sequence_state.current_sequence {
                continue;
            }

            if typing_text.full_text.is_empty() {
                typing_text.full_text = sequence_state.texts[sequence.sequence_index].clone();
            }

            typing_text.timer.tick(time.delta());

            if typing_text.timer.just_finished()
                && typing_text.current_index < typing_text.full_text.len()
            {
                let next_char = typing_text
                    .full_text
                    .chars()
                    .nth(typing_text.current_index)
                    .unwrap();
                typing_text.current_index += 1;

                if let Some(section) = text.sections.first_mut() {
                    section.value = typing_text.full_text[..typing_text.current_index].to_string();
                }

                if next_char != ' ' {
                    commands.spawn(AudioBundle {
                        source: typewriter_sound.0.clone(),
                        settings: PlaybackSettings::DESPAWN,
                        ..default()
                    });
                }

                if typing_text.current_index == typing_text.full_text.len() {
                    typing_text.completed = true;
                }
            }
        }
    }

    fn game3(
        time: Res<Time>,
        mut game_state: ResMut<NextState<GameState>>,
        mut timer: ResMut<GameTimer>,
    ) {
        if timer.tick(time.delta()).finished() {
            game_state.set(GameState::Chapter3);
        }
    }

    fn animate_sprite(
        time: Res<Time>,
        mut query: Query<(&mut TextureAtlas, &mut AnimationTimer, &AnimationIndices)>,
    ) {
        for (mut atlas, mut timer, indices) in &mut query {
            timer.0.tick(time.delta());
            if timer.0.just_finished() {
                atlas.index = if atlas.index == indices.last {
                    indices.first
                } else {
                    atlas.index + 1
                };
            }
        }
    }
}

mod game4 {
    use super::{despawn_screen, DisplayQuality, GameState, Volume, TEXT_COLOR};
    use bevy::prelude::*;

    // Add this new resource to handle the custom font
    #[derive(Resource)]
    struct GameFont(Handle<Font>);

    #[derive(Component)]
    struct OnGameScreen;

    #[derive(Component)]
    struct AnimationTimer(Timer);

    #[derive(Component)]
    struct AnimationIndices {
        first: usize,
        last: usize,
    }

    #[derive(Component)]
    struct TextContainer;

    #[derive(Component)]
    struct TypingText {
        full_text: String,
        current_index: usize,
        timer: Timer,
        completed: bool,
    }

    #[derive(Component)]
    struct TextSequence {
        sequence_index: usize,
    }

    #[derive(Resource)]
    struct TextSequenceState {
        current_sequence: usize,
        texts: Vec<String>,
        delay_timer: Timer,
        ready_for_next: bool,
    }

    #[derive(Resource)]
    struct TypewriterSound(Handle<AudioSource>);

    #[derive(Resource, Deref, DerefMut)]
    struct GameTimer(Timer);

    fn game_setup3(
        mut commands: Commands,
        display_quality: Res<DisplayQuality>,
        volume: Res<Volume>,
        asset_server: Res<AssetServer>,
        mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
        windows: Query<&Window>,
    ) {
        let window = windows.single();

        // Load custom font and create resource
        let custom_font = asset_server.load("joystix monospace.otf");
        commands.insert_resource(GameFont(custom_font));
        commands.init_resource::<GameSetupComplete>();

        // Load typewriter sound
        let typewriter_sound = asset_server.load("sounds/typewriter.ogg");
        commands.insert_resource(TypewriterSound(typewriter_sound));

        // Initialize text sequence
        commands.insert_resource(TextSequenceState {
            current_sequence: 0,
            texts: vec![
                "  ".to_string(),
                "A pile of rubble lies at your feet".to_string(),
                "You hear voices chanting...".to_string(),
                "Stella luminara, verita serena...".to_string(),
            ],
            delay_timer: Timer::from_seconds(4.0, TimerMode::Once), // 4 second delay between texts
            ready_for_next: true,
        });

        // Load the sprite sheet
        let texture_handle = asset_server.load("textures/summoning.png");

        let layout = TextureAtlasLayout::from_grid(UVec2::new(576, 324), 5, 64, None, None);

        let atlas_layout = atlas_layouts.add(layout);

        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                    ..default()
                },
                OnGameScreen,
            ))
            .with_children(|parent| {
                // Text container at the bottom
                parent.spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Percent(20.0),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::FlexEnd,
                            padding: UiRect::all(Val::Px(20.0)),
                            position_type: PositionType::Absolute,
                            bottom: Val::Px(0.0),
                            ..default()
                        },
                        background_color: Color::srgba(0.0, 0.0, 0.0, 0.7).into(),
                        ..default()
                    },
                    TextContainer,
                ));

                // Sprite container
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            width: Val::Vw(100.0),
                            height: Val::Vh(100.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|parent| {
                        parent.spawn((
                            SpriteSheetBundle {
                                texture: texture_handle,
                                atlas: TextureAtlas {
                                    layout: atlas_layout,
                                    index: 0,
                                },
                                transform: Transform::from_xyz(
                                    -window.width() / 2.0,
                                    -window.height() / 2.0 + 60.0,
                                    1.0,
                                ),
                                sprite: Sprite {
                                    custom_size: Some(Vec2::new(1920.0, 1080.0)),
                                    anchor: bevy::sprite::Anchor::Center,
                                    ..default()
                                },
                                ..default()
                            },
                            AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)), // for (entity, _, sequence) in typing_query.iter() {
                            //     if sequence.sequence_index <= sequence_state.current_sequence {
                            //         commands.entity(entity).despawn();
                            //     }
                            // }
                            AnimationIndices {
                                first: 0,
                                last: 320,
                            },
                        ));
                    });
            });

        commands.insert_resource(GameTimer(Timer::from_seconds(14.0, TimerMode::Once)));
        // Immediately despawn all previous text when we're ready for the next one
        // for (entity, _, sequence) in typing_query.iter() {
        //     if sequence.sequence_index <= sequence_state.current_sequence {
        //         commands.entity(entity).despawn();
        //     }
        // }
        // make liek 20 s when proper

        // We can't spawn the text in game_setup because we need to wait for the GameFont resource to be available
        // Instead, we'll create a new system to handle the initial text spawn
    }

    // Add marker resource to ensure proper setup order
    #[derive(Resource, Default)]
    struct GameSetupComplete;

    // Modify the spawn_initial_text system to run after setup is complete
    fn spawn_initial_text(
        mut commands: Commands,
        game_font: Res<GameFont>,
        setup_complete: Res<GameSetupComplete>,
    ) {
        spawn_text_entity(&mut commands, 0, &game_font);
    }

    // Update the game plugin with proper system ordering
    pub fn game_plugin_3(app: &mut App) {
        app.init_resource::<GameSetupComplete>() // Initialize the marker resource
            .add_systems(OnEnter(GameState::Game4), game_setup3)
            .add_systems(
                OnEnter(GameState::Game4),
                spawn_initial_text.after(game_setup3),
            )
            .add_systems(
                Update,
                (game3, animate_sprite, manage_text_sequence, type_text)
                    .run_if(in_state(GameState::Game4)),
            )
            .add_systems(OnExit(GameState::Game4), despawn_screen::<OnGameScreen>);
    }

    // Make sure spawn_text_entity uses the correct font parameter
    fn spawn_text_entity(
        commands: &mut Commands,
        sequence_index: usize,
        game_font: &Res<GameFont>,
    ) {
        commands.spawn((
            TextBundle::from_section(
                "",
                TextStyle {
                    font: game_font.0.clone(),
                    font_size: 40.0,
                    color: Color::WHITE,
                    ..default()
                },
            )
            .with_style(Style {
                margin: UiRect::bottom(Val::Px(20.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                bottom: Val::Px(80.0),
                left: Val::Px(20.0),
                right: Val::Px(20.0),
                ..default()
            }),
            TypingText {
                full_text: String::new(),
                current_index: 0,
                timer: Timer::from_seconds(0.08, TimerMode::Repeating),
                completed: false,
            },
            TextSequence { sequence_index },
        ));
    }

    fn manage_text_sequence(
        mut commands: Commands,
        mut sequence_state: ResMut<TextSequenceState>,
        time: Res<Time>,
        typing_query: Query<(Entity, &TypingText, &TextSequence)>,
        game_font: Res<GameFont>,
    ) {
        if !sequence_state.ready_for_next {
            sequence_state.delay_timer.tick(time.delta());
            if sequence_state.delay_timer.finished() {
                sequence_state.ready_for_next = true;
                // Immediately despawn all previous text when we're ready for the next one
                for (entity, _, sequence) in typing_query.iter() {
                    if sequence.sequence_index <= sequence_state.current_sequence {
                        commands.entity(entity).despawn();
                    }
                }
            }
            return;
        }

        let mut all_completed = true;
        for (_, typing_text, sequence) in typing_query.iter() {
            if sequence.sequence_index == sequence_state.current_sequence {
                if !typing_text.completed {
                    all_completed = false;
                    break;
                }
            }
        }

        if all_completed && sequence_state.current_sequence < sequence_state.texts.len() - 1 {
            // Clear any existing text before spawning new one
            for (entity, _, _) in typing_query.iter() {
                commands.entity(entity).despawn();
            }

            sequence_state.current_sequence += 1;
            sequence_state.ready_for_next = false;
            sequence_state.delay_timer.reset();

            spawn_text_entity(&mut commands, sequence_state.current_sequence, &game_font);
        }
    }

    fn type_text(
        time: Res<Time>,
        sequence_state: Res<TextSequenceState>,
        mut query: Query<(&mut TypingText, &mut Text, &TextSequence)>,
        typewriter_sound: Res<TypewriterSound>,
        mut commands: Commands,
    ) {
        for (mut typing_text, mut text, sequence) in query.iter_mut() {
            if typing_text.completed || sequence.sequence_index != sequence_state.current_sequence {
                continue;
            }

            if typing_text.full_text.is_empty() {
                typing_text.full_text = sequence_state.texts[sequence.sequence_index].clone();
            }

            typing_text.timer.tick(time.delta());

            if typing_text.timer.just_finished()
                && typing_text.current_index < typing_text.full_text.len()
            {
                let next_char = typing_text
                    .full_text
                    .chars()
                    .nth(typing_text.current_index)
                    .unwrap();
                typing_text.current_index += 1;

                if let Some(section) = text.sections.first_mut() {
                    section.value = typing_text.full_text[..typing_text.current_index].to_string();
                }

                if next_char != ' ' {
                    commands.spawn(AudioBundle {
                        source: typewriter_sound.0.clone(),
                        settings: PlaybackSettings::DESPAWN,
                        ..default()
                    });
                }

                if typing_text.current_index == typing_text.full_text.len() {
                    typing_text.completed = true;
                }
            }
        }
    }

    fn game3(
        time: Res<Time>,
        mut game_state: ResMut<NextState<GameState>>,
        mut timer: ResMut<GameTimer>,
    ) {
        if timer.tick(time.delta()).finished() {
            game_state.set(GameState::Chapter4);
        }
    }

    fn animate_sprite(
        time: Res<Time>,
        mut query: Query<(&mut TextureAtlas, &mut AnimationTimer, &AnimationIndices)>,
    ) {
        for (mut atlas, mut timer, indices) in &mut query {
            timer.0.tick(time.delta());
            if timer.0.just_finished() {
                atlas.index = if atlas.index == indices.last {
                    indices.first
                } else {
                    atlas.index + 1
                };
            }
        }
    }
}

mod menu {
    use bevy::{
        app::AppExit,
        color::palettes::css::{CRIMSON, GREEN},
        prelude::*,
    };

    use super::{despawn_screen, DisplayQuality, GameState, Volume, TEXT_COLOR};

    // This plugin manages the menu, with 5 different screens:
    // - a main menu with "New Game", "Settings", "Quit"
    // - a settings menu with two submenus and a back button
    // - two settings screen with a setting that can be set and a back button
    pub fn menu_plugin(app: &mut App) {
        app
            // At start, the menu is not enabled. This will be changed in `menu_setup` when
            // entering the `GameState::Menu` state.
            // Current screen in the menu is handled by an independent state from `GameState`
            .init_state::<MenuState>()
            .add_systems(OnEnter(GameState::Menu), menu_setup)
            // Systems to handle the main menu screen
            .add_systems(OnEnter(MenuState::Main), main_menu_setup)
            .add_systems(OnExit(MenuState::Main), despawn_screen::<OnMainMenuScreen>)
            // Systems to handle the settings menu screen
            .add_systems(OnEnter(MenuState::Settings), settings_menu_setup)
            .add_systems(
                OnExit(MenuState::Settings),
                despawn_screen::<OnSettingsMenuScreen>,
            )
            // Systems to handle the display settings screen
            .add_systems(
                OnEnter(MenuState::SettingsDisplay),
                display_settings_menu_setup,
            )
            .add_systems(
                Update,
                (setting_button::<DisplayQuality>.run_if(in_state(MenuState::SettingsDisplay)),),
            )
            .add_systems(
                OnExit(MenuState::SettingsDisplay),
                despawn_screen::<OnDisplaySettingsMenuScreen>,
            )
            // Systems to handle the sound settings screen
            .add_systems(OnEnter(MenuState::SettingsSound), sound_settings_menu_setup)
            .add_systems(
                Update,
                setting_button::<Volume>.run_if(in_state(MenuState::SettingsSound)),
            )
            .add_systems(
                OnExit(MenuState::SettingsSound),
                despawn_screen::<OnSoundSettingsMenuScreen>,
            )
            // Common systems to all screens that handles buttons behavior
            .add_systems(
                Update,
                (menu_action, button_system).run_if(in_state(GameState::Menu)),
            );
    }

    // State used for the current menu screen
    #[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
    enum MenuState {
        Main,
        Settings,
        SettingsDisplay,
        SettingsSound,
        #[default]
        Disabled,
    }

    // Tag component used to tag entities added on the main menu screen
    #[derive(Component)]
    struct OnMainMenuScreen;

    // Tag component used to tag entities added on the settings menu screen
    #[derive(Component)]
    struct OnSettingsMenuScreen;

    // Tag component used to tag entities added on the display settings menu screen
    #[derive(Component)]
    struct OnDisplaySettingsMenuScreen;

    // Tag component used to tag entities added on the sound settings menu screen
    #[derive(Component)]
    struct OnSoundSettingsMenuScreen;

    const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
    const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
    const HOVERED_PRESSED_BUTTON: Color = Color::srgb(0.25, 0.65, 0.25);
    const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

    // Tag component used to mark which setting is currently selected
    #[derive(Component)]
    struct SelectedOption;

    // All actions that can be triggered from a button click
    #[derive(Component)]
    enum MenuButtonAction {
        Play,
        Settings,
        SettingsDisplay,
        SettingsSound,
        BackToMainMenu,
        BackToSettings,
        Quit,
    }

    // This system handles changing all buttons color based on mouse interaction
    fn button_system(
        mut interaction_query: Query<
            (&Interaction, &mut UiImage, Option<&SelectedOption>),
            (Changed<Interaction>, With<Button>),
        >,
    ) {
        for (interaction, mut image, selected) in &mut interaction_query {
            image.color = match (*interaction, selected) {
                (Interaction::Pressed, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON,
                (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON,
                (Interaction::Hovered, None) => HOVERED_BUTTON,
                (Interaction::None, None) => NORMAL_BUTTON,
            }
        }
    }

    // This system updates the settings when a new value for a setting is selected, and marks
    // the button as the one currently selected
    fn setting_button<T: Resource + Component + PartialEq + Copy>(
        interaction_query: Query<(&Interaction, &T, Entity), (Changed<Interaction>, With<Button>)>,
        mut selected_query: Query<(Entity, &mut UiImage), With<SelectedOption>>,
        mut commands: Commands,
        mut setting: ResMut<T>,
    ) {
        for (interaction, button_setting, entity) in &interaction_query {
            if *interaction == Interaction::Pressed && *setting != *button_setting {
                let (previous_button, mut previous_image) = selected_query.single_mut();
                previous_image.color = NORMAL_BUTTON;
                commands.entity(previous_button).remove::<SelectedOption>();
                commands.entity(entity).insert(SelectedOption);
                *setting = *button_setting;
            }
        }
    }

    fn menu_setup(mut menu_state: ResMut<NextState<MenuState>>) {
        menu_state.set(MenuState::Main);
    }

    fn main_menu_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
        // Common style for all buttons on the screen
        let button_style = Style {
            width: Val::Px(250.0),
            height: Val::Px(65.0),
            margin: UiRect::all(Val::Px(20.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        };
        let button_icon_style = Style {
            width: Val::Px(30.0),
            position_type: PositionType::Absolute,
            left: Val::Px(10.0),
            ..default()
        };
        let button_text_style = TextStyle {
            font_size: 40.0,
            color: TEXT_COLOR,
            ..default()
        };

        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ..default()
                },
                OnMainMenuScreen,
            ))
            .with_children(|parent| {
                // Background image
                parent.spawn(ImageBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    image: UiImage::new(asset_server.load("textures/Game Icons/1.png")),
                    ..default()
                });

                parent
                    .spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|parent| {
                        // Top logo/title image
                        parent.spawn(ImageBundle {
                            style: Style {
                                width: Val::Px(800.0),              // Adjust size as needed
                                height: Val::Px(600.0),             // Adjust size as needed
                                margin: UiRect::all(Val::Px(50.0)), // Add some space between logo and buttons
                                ..default()
                            },
                            // Replace with your actual logo image path
                            image: UiImage::new(asset_server.load("textures/logo.png")),
                            ..default()
                        });

                        // New Game button
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: button_style.clone(),
                                    background_color: NORMAL_BUTTON.into(),
                                    ..default()
                                },
                                MenuButtonAction::Play,
                            ))
                            .with_children(|parent| {
                                let icon = asset_server.load("textures/Game Icons/right.png");
                                parent.spawn(ImageBundle {
                                    style: button_icon_style.clone(),
                                    image: UiImage::new(icon),
                                    ..default()
                                });
                                parent.spawn(TextBundle::from_section(
                                    "New Game",
                                    button_text_style.clone(),
                                ));
                            });

                        // Quit button
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: button_style,
                                    background_color: NORMAL_BUTTON.into(),
                                    ..default()
                                },
                                MenuButtonAction::Quit,
                            ))
                            .with_children(|parent| {
                                let icon = asset_server.load("textures/Game Icons/exitRight.png");
                                parent.spawn(ImageBundle {
                                    style: button_icon_style,
                                    image: UiImage::new(icon),
                                    ..default()
                                });
                                parent.spawn(TextBundle::from_section("Quit", button_text_style));
                            });
                    });
            });
    }

    fn settings_menu_setup(mut commands: Commands) {
        let button_style = Style {
            width: Val::Px(200.0),
            height: Val::Px(65.0),
            margin: UiRect::all(Val::Px(20.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        };

        let button_text_style = TextStyle {
            font_size: 40.0,
            color: TEXT_COLOR,
            ..default()
        };

        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ..default()
                },
                OnSettingsMenuScreen,
            ))
            .with_children(|parent| {
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: CRIMSON.into(),
                        ..default()
                    })
                    .with_children(|parent| {
                        for (action, text) in [
                            (MenuButtonAction::SettingsDisplay, "Display"),
                            (MenuButtonAction::SettingsSound, "Sound"),
                            (MenuButtonAction::BackToMainMenu, "Back"),
                        ] {
                            parent
                                .spawn((
                                    ButtonBundle {
                                        style: button_style.clone(),
                                        background_color: NORMAL_BUTTON.into(),
                                        ..default()
                                    },
                                    action,
                                ))
                                .with_children(|parent| {
                                    parent.spawn(TextBundle::from_section(
                                        text,
                                        button_text_style.clone(),
                                    ));
                                });
                        }
                    });
            });
    }

    fn display_settings_menu_setup(mut commands: Commands, display_quality: Res<DisplayQuality>) {
        let button_style = Style {
            width: Val::Px(200.0),
            height: Val::Px(65.0),
            margin: UiRect::all(Val::Px(20.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        };
        let button_text_style = TextStyle {
            font_size: 40.0,
            color: TEXT_COLOR,
            ..default()
        };

        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ..default()
                },
                OnDisplaySettingsMenuScreen,
            ))
            .with_children(|parent| {
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: CRIMSON.into(),
                        ..default()
                    })
                    .with_children(|parent| {
                        // Create a new `NodeBundle`, this time not setting its `flex_direction`. It will
                        // use the default value, `FlexDirection::Row`, from left to right.
                        parent
                            .spawn(NodeBundle {
                                style: Style {
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                background_color: CRIMSON.into(),
                                ..default()
                            })
                            .with_children(|parent| {
                                // Display a label for the current setting
                                parent.spawn(TextBundle::from_section(
                                    "Display Quality",
                                    button_text_style.clone(),
                                ));
                                // Display a button for each possible value
                                for quality_setting in [
                                    DisplayQuality::Low,
                                    DisplayQuality::Medium,
                                    DisplayQuality::High,
                                ] {
                                    let mut entity = parent.spawn((
                                        ButtonBundle {
                                            style: Style {
                                                width: Val::Px(150.0),
                                                height: Val::Px(65.0),
                                                ..button_style.clone()
                                            },
                                            background_color: NORMAL_BUTTON.into(),
                                            ..default()
                                        },
                                        quality_setting,
                                    ));
                                    entity.with_children(|parent| {
                                        parent.spawn(TextBundle::from_section(
                                            format!("{quality_setting:?}"),
                                            button_text_style.clone(),
                                        ));
                                    });
                                    if *display_quality == quality_setting {
                                        entity.insert(SelectedOption);
                                    }
                                }
                            });
                        // Display the back button to return to the settings screen
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: button_style,
                                    background_color: NORMAL_BUTTON.into(),
                                    ..default()
                                },
                                MenuButtonAction::BackToSettings,
                            ))
                            .with_children(|parent| {
                                parent.spawn(TextBundle::from_section("Back", button_text_style));
                            });
                    });
            });
    }

    fn sound_settings_menu_setup(mut commands: Commands, volume: Res<Volume>) {
        let button_style = Style {
            width: Val::Px(200.0),
            height: Val::Px(65.0),
            margin: UiRect::all(Val::Px(20.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        };
        let button_text_style = TextStyle {
            font_size: 40.0,
            color: TEXT_COLOR,
            ..default()
        };

        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ..default()
                },
                OnSoundSettingsMenuScreen,
            ))
            .with_children(|parent| {
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: CRIMSON.into(),
                        ..default()
                    })
                    .with_children(|parent| {
                        parent
                            .spawn(NodeBundle {
                                style: Style {
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                background_color: CRIMSON.into(),
                                ..default()
                            })
                            .with_children(|parent| {
                                parent.spawn(TextBundle::from_section(
                                    "Volume",
                                    button_text_style.clone(),
                                ));
                                for volume_setting in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9] {
                                    let mut entity = parent.spawn((
                                        ButtonBundle {
                                            style: Style {
                                                width: Val::Px(30.0),
                                                height: Val::Px(65.0),
                                                ..button_style.clone()
                                            },
                                            background_color: NORMAL_BUTTON.into(),
                                            ..default()
                                        },
                                        Volume(volume_setting),
                                    ));
                                    if *volume == Volume(volume_setting) {
                                        entity.insert(SelectedOption);
                                    }
                                }
                            });
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: button_style,
                                    background_color: NORMAL_BUTTON.into(),
                                    ..default()
                                },
                                MenuButtonAction::BackToSettings,
                            ))
                            .with_children(|parent| {
                                parent.spawn(TextBundle::from_section("Back", button_text_style));
                            });
                    });
            });
    }

    fn menu_action(
        interaction_query: Query<
            (&Interaction, &MenuButtonAction),
            (Changed<Interaction>, With<Button>),
        >,
        mut app_exit_events: EventWriter<AppExit>,
        mut menu_state: ResMut<NextState<MenuState>>,
        mut game_state: ResMut<NextState<GameState>>,
    ) {
        for (interaction, menu_button_action) in &interaction_query {
            if *interaction == Interaction::Pressed {
                match menu_button_action {
                    MenuButtonAction::Quit => {
                        app_exit_events.send(AppExit::Success);
                    }
                    MenuButtonAction::Play => {
                        // game_state.set(GameState::Chapter3);
                        game_state.set(GameState::Game);

                        menu_state.set(MenuState::Disabled);
                    }
                    MenuButtonAction::Settings => menu_state.set(MenuState::Settings),
                    MenuButtonAction::SettingsDisplay => {
                        menu_state.set(MenuState::SettingsDisplay);
                    }
                    MenuButtonAction::SettingsSound => {
                        menu_state.set(MenuState::SettingsSound);
                    }
                    MenuButtonAction::BackToMainMenu => menu_state.set(MenuState::Main),
                    MenuButtonAction::BackToSettings => {
                        menu_state.set(MenuState::Settings);
                    }
                }
            }
        }
    }
}

// Generic system that takes a component as a parameter, and will despawn all entities with that component
fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}

mod chapter1 {
    use crate::game2;

    use super::GameState;
    use bevy::app::AppExit;
    use bevy::ecs::system::ParamSet;
    use bevy::prelude::*;

    #[derive(Component, Copy, Clone, Debug, PartialEq)]
    enum CardType {
        Fire,
        Ice,
        Air,
        Earth,
        Crystal,
        // Add other types as needed
    }
    // Components
    #[derive(Component)]
    struct AnimationTimer(Timer);

    #[derive(Component)]
    struct AnimationIndices {
        first: usize,
        last: usize,
    }

    #[derive(Component)]
    struct Card;

    #[derive(Component)]
    struct OriginalPosition(Vec2);

    #[derive(Component)]
    struct OnChapterOneScreen;

    #[derive(Component)]
    struct SideCharacter;

    #[derive(Component)]
    struct Monster;

    #[derive(Component)]
    struct Health {
        current: f32,
        maximum: f32,
    }

    // Add this to your existing components if not already present
    #[derive(Component)]
    struct HealthBarContainer;

    #[derive(Component)]
    struct HealthBar;

    // Add these new components in the chapter1 mod
    #[derive(Component)]
    struct EndTurnButton;

    #[derive(Component)]
    struct ButtonText;

    #[derive(Component)]
    struct Damage(f32);

    #[derive(Component)]
    struct DeathScreen;

    #[derive(Component)]
    struct DeathText;

    #[derive(Component)]
    struct FadeInEffect {
        timer: Timer,
    }

    #[derive(Component)]
    struct ReturnToMenuTimer {
        timer: Timer,
    }

    #[derive(Component)]
    struct DamageDisplay;

    #[derive(Resource, Default)]
    struct PendingAirCards(i32);

    //CHANGE
    #[derive(Component)]
    struct CardPlayAnimation {
        elapsed_time: f32,
        duration: f32,
    }

    fn animate_card_play(
        mut commands: Commands,
        time: Res<Time>,
        windows: Query<&Window>,
        mut animation_query: Query<(Entity, &mut Style, &mut CardPlayAnimation)>,
        monster_query: Query<&Transform, With<Monster>>,
    ) {
        let window = windows.single();

        // Get monster position once
        if let Ok(monster_transform) = monster_query.get_single() {
            for (entity, mut style, mut animation) in animation_query.iter_mut() {
                animation.elapsed_time += time.delta_seconds();
                let progress = (animation.elapsed_time / animation.duration).min(1.0);

                // Scale down the card as it moves
                let scale = 1.0 - (progress * 0.9); // Scale to 10% of original size
                style.width = Val::Px(180.0 * scale);
                style.height = Val::Px(250.0 * scale);

                // Move card towards monster
                style.top = Val::Px(monster_transform.translation.y);

                // Remove card when animation is done
                if progress >= 1.0 {
                    commands.entity(entity).despawn_recursive();
                }
            }
        }
    }
    //END CAHNGE

    fn spawn_death_screen(commands: &mut Commands, asset_server: &AssetServer) {
        // Main container
        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        position_type: PositionType::Absolute,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.0)),
                    ..default()
                },
                DeathScreen,
                FadeInEffect {
                    timer: Timer::from_seconds(2.0, TimerMode::Once),
                },
                // Add a timer to return to menu after 5 seconds
                ReturnToMenuTimer {
                    timer: Timer::from_seconds(5.0, TimerMode::Once),
                },
            ))
            .with_children(|parent| {
                // "YOU DIED" text
                parent.spawn((
                    TextBundle::from_section(
                        "YOU DIED",
                        TextStyle {
                            font_size: 120.0,
                            color: Color::rgba(0.8, 0.0, 0.0, 0.0), // Start transparent
                            ..default()
                        },
                    ),
                    DeathText,
                    FadeInEffect {
                        timer: Timer::from_seconds(2.0, TimerMode::Once),
                    },
                ));
            });
    }

    // Update the death screen system to handle both fade-in and menu transition
    fn update_death_screen(
        mut commands: Commands,
        time: Res<Time>,
        mut query: Query<
            (
                Entity,
                &mut BackgroundColor,
                &mut FadeInEffect,
                &mut ReturnToMenuTimer,
            ),
            With<DeathScreen>,
        >,
        mut text_query: Query<&mut Text, With<DeathText>>,
        mut game_state: ResMut<NextState<GameState>>,
        mut app_exit_events: EventWriter<AppExit>,
    ) {
        for (entity, mut bg_color, mut fade, mut return_timer) in query.iter_mut() {
            // Update fade effect
            fade.timer.tick(time.delta());
            let alpha = fade.timer.fraction();
            bg_color.0 = Color::rgba(0.0, 0.0, 0.0, alpha * 0.7);

            // Update text color
            if let Ok(mut text) = text_query.get_single_mut() {
                text.sections[0].style.color = Color::rgba(0.8, 0.0, 0.0, alpha);
            }

            // Update return timer
            return_timer.timer.tick(time.delta());
            if return_timer.timer.finished() {
                app_exit_events.send(AppExit::Success);
            }
        }
    }
    // Modify TurnState to include pending air cards
    #[derive(Resource)]
    struct TurnState {
        first_card_played: bool,
        cards_played_this_turn: Vec<CardType>,
        crystal_power: i32,
        turn_count: i32,
        pending_air_cards: i32,
    }

    impl Default for TurnState {
        fn default() -> Self {
            Self {
                first_card_played: true,
                cards_played_this_turn: Vec::new(),
                crystal_power: 0,
                turn_count: 0,
                pending_air_cards: 0, // Initialize pending_air_cards to 0
            }
        }
    }

    // Constants for base damage values
    const FIRE_BASE_DAMAGE: f32 = 8.0;
    const FIRE_FIRST_CARD_BONUS: f32 = 7.0;
    const ICE_BASE_DAMAGE: f32 = 6.0;
    const CRYSTAL_BASE_DAMAGE: f32 = 4.0;
    const AIR_BASE_DAMAGE: f32 = 2.0;
    const EARTH_BASE_DAMAGE: f32 = 5.0;
    const HEAL_BASE_DAMAGE: f32 = 5.0;

    fn update_health_bars(
        query: Query<(&Health, &Children), Without<HealthBar>>,
        mut health_bar_query: Query<&mut Sprite, With<HealthBar>>,
    ) {
        for (health, children) in query.iter() {
            for child in children.iter() {
                if let Ok(mut bar_sprite) = health_bar_query.get_mut(*child) {
                    // Update health bar width based on current health
                    let bar_width = 100.0;
                    let health_percentage = health.current / health.maximum;

                    bar_sprite.custom_size = Some(Vec2::new(
                        bar_width * health_percentage,
                        bar_sprite.custom_size.unwrap_or(Vec2::ZERO).y,
                    ));

                    // Update color based on health percentage
                    bar_sprite.color = if health_percentage > 0.5 {
                        Color::srgb(0.0, 1.0, 0.0) // Green: rgb(0, 255, 0)
                    } else if health_percentage > 0.25 {
                        Color::srgb(1.0, 0.65, 0.0) // Orange: rgb(255, 165, 0)
                    } else {
                        Color::srgb(1.0, 0.0, 0.0) // Red: rgb(255, 0, 0)
                    };
                }
            }
        }
    }
    #[derive(Resource)]
    struct FightState {
        current_turn: Turn,
        selected_card: Option<usize>,
    }

    #[derive(PartialEq)]
    enum Turn {
        Player,
        Enemy,
    }

    impl Default for FightState {
        fn default() -> Self {
            Self {
                current_turn: Turn::Player,
                selected_card: None,
            }
        }
    }

    // Update the card hover system to use FightState
    fn update_card_hover(
        mut card_query: Query<
            (
                &Interaction,
                &mut Transform,
                &OriginalPosition,
                &mut Style,
                Entity,
            ),
            (With<Card>, Changed<Interaction>),
        >,
        mut commands: Commands,
        fight_state: Res<FightState>,
    ) {
        for (interaction, mut transform, original_pos, mut style, entity) in card_query.iter_mut() {
            match *interaction {
                Interaction::Hovered => {
                    if fight_state.current_turn == Turn::Player {
                        transform.translation.y = original_pos.0.y + 30.0;
                        style.width = Val::Px(200.0);
                        style.height = Val::Px(280.0);
                    }
                }
                _ => {
                    transform.translation.y = original_pos.0.y;
                    style.width = Val::Px(180.0);
                    style.height = Val::Px(250.0);
                }
            }
        }
    }

    fn handle_card_click(
        mut commands: Commands,
        mut card_query: Query<
            (&Interaction, Entity, &CardType),
            (Changed<Interaction>, With<Card>),
        >,
        cards_in_hand: Query<Entity, With<Card>>, // Query to count cards
        mut fight_state: ResMut<FightState>,
        mut turn_state: ResMut<TurnState>,
        mut monster_query: Query<(Entity, &mut Health, &Children), With<Monster>>,
        health_container_query: Query<&Children, With<HealthBarContainer>>,
        mut health_bar_query: Query<&mut Sprite, With<HealthBar>>,
        asset_server: Res<AssetServer>,
    ) {
        if fight_state.current_turn != Turn::Player {
            return;
        }

        for (interaction, card_entity, card_type) in card_query.iter() {
            if *interaction == Interaction::Pressed {
                println!("First card played status: {}", turn_state.first_card_played);
                // Add animation component
                commands
                    .entity(card_entity)
                    .insert(CardPlayAnimation {
                        elapsed_time: 0.0,
                        duration: 3.0, // Animation duration in seconds
                    })
                    .remove::<Interaction>();
                // Calculate damage based on whether this is the first card
                let is_first = turn_state.first_card_played;
                let cards_in_hand_count = cards_in_hand.iter().count() as f32; // Get count here

                let damage = if *card_type == CardType::Fire && is_first {
                    println!("Fire card played as first card! Enhanced damage!");
                    FIRE_BASE_DAMAGE + FIRE_FIRST_CARD_BONUS
                } else {
                    match card_type {
                        CardType::Fire => {
                            println!("Fire card played but not first");
                            FIRE_BASE_DAMAGE
                        }
                        CardType::Ice => {
                            let mut damage = ICE_BASE_DAMAGE;

                            if let Some(last_card) = turn_state.cards_played_this_turn.last() {
                                if matches!(last_card, CardType::Fire) {
                                    damage *= 2.0;
                                }
                            }

                            if turn_state
                                .cards_played_this_turn
                                .iter()
                                .any(|c| matches!(c, CardType::Earth))
                            {
                                damage = 0.0;
                            }

                            damage
                        }
                        CardType::Crystal => {
                            let effects_bonus =
                                (turn_state.cards_played_this_turn.len() as f32) * 2.0;
                            let turn_bonus = turn_state.crystal_power as f32;
                            CRYSTAL_BASE_DAMAGE + effects_bonus + turn_bonus
                        }
                        CardType::Air => AIR_BASE_DAMAGE,
                        CardType::Earth => {
                            let turn_bonus = turn_state.turn_count as f32;
                            EARTH_BASE_DAMAGE + cards_in_hand_count + turn_bonus
                            // Use the count here
                        }
                    }
                };

                // Deal damage
                for (entity, mut monster_health, children) in monster_query.iter_mut() {
                    monster_health.current = (monster_health.current - damage).max(0.0);
                    println!("Dealing {} damage. First card: {}", damage, is_first);
                    spawn_damage_text(&mut commands, damage, &asset_server);
                    // Update monster's health bar
                    for child in children.iter() {
                        if let Ok(container_children) = health_container_query.get(*child) {
                            for health_bar_entity in container_children.iter() {
                                if let Ok(mut bar_sprite) =
                                    health_bar_query.get_mut(*health_bar_entity)
                                {
                                    let bar_width = 150.0; // Match the width set in chapter1_setup
                                    let health_percentage =
                                        monster_health.current / monster_health.maximum;

                                    bar_sprite.custom_size = Some(Vec2::new(
                                        bar_width * health_percentage,
                                        bar_sprite.custom_size.unwrap_or(Vec2::ZERO).y,
                                    ));

                                    // Update color based on health percentage
                                    bar_sprite.color = if health_percentage > 0.5 {
                                        Color::srgb(0.0, 1.0, 0.0) // Green
                                    } else if health_percentage > 0.25 {
                                        Color::srgb(1.0, 0.65, 0.0) // Orange
                                    } else {
                                        Color::srgb(1.0, 0.0, 0.0) // Red
                                    };
                                }
                            }
                        }
                    }

                    // If monster dies, despawn it
                    if monster_health.current <= 0.0 {
                        commands.entity(entity).despawn_recursive();
                    }
                }

                // Handle special card effects and cleanup
                if matches!(card_type, CardType::Air) {
                    turn_state.pending_air_cards += 2;
                }

                // Update turn state BEFORE destroying the card
                turn_state.cards_played_this_turn.push(*card_type);
                turn_state.first_card_played = false;
                println!("Set first_card_played to false");

                // Destroy the played card
                commands.entity(card_entity).despawn_recursive();

                break;
            }
        }
    }

    // Add this system to help debug the turn state
    // Add this system to help debug the turn state
    fn debug_turn_state(turn_state: Res<TurnState>, fight_state: Res<FightState>) {
        println!(
            "Turn State - First card: {}, Cards played: {}",
            turn_state.first_card_played,
            turn_state.cards_played_this_turn.len(),
        );
    }

    // Add this new component for the jump animation
    #[derive(Component)]
    struct JumpAnimation {
        start_y: f32,
        elapsed_time: f32,
        duration: f32,
        height: f32,
    }

    fn process_turn(
        mut fight_state: ResMut<FightState>,
        mut query_set: ParamSet<(
            Query<(&mut Health, &Children), With<SideCharacter>>,
            Query<(&Health, &Damage), With<Monster>>,
        )>,
        health_container_query: Query<&Children, With<HealthBarContainer>>,
        mut health_bar_query: Query<&mut Sprite, With<HealthBar>>,
        asset_server: Res<AssetServer>,
        mut commands: Commands,
    ) {
        if fight_state.current_turn == Turn::Enemy {
            // First, collect all living monsters and their damage
            let monster_attacks: Vec<f32> = query_set
                .p1()
                .iter()
                .filter(|(health, _)| health.current > 0.0)
                .map(|(_, damage)| damage.0)
                .collect();

            // Then apply damage to the player
            if let Ok((mut character_health, children)) = query_set.p0().get_single_mut() {
                for damage in monster_attacks {
                    character_health.current = (character_health.current - damage).max(0.0);
                    println!(
                        "Player health: {}/{}",
                        character_health.current, character_health.maximum
                    );

                    // Health bar update logic using nested queries
                    for child in children.iter() {
                        if let Ok(container_children) = health_container_query.get(*child) {
                            for health_bar_entity in container_children.iter() {
                                if let Ok(mut bar_sprite) =
                                    health_bar_query.get_mut(*health_bar_entity)
                                {
                                    let bar_width = 150.0;
                                    let health_percentage =
                                        character_health.current / character_health.maximum;

                                    bar_sprite.custom_size = Some(Vec2::new(
                                        bar_width * health_percentage,
                                        bar_sprite.custom_size.unwrap_or(Vec2::ZERO).y,
                                    ));

                                    bar_sprite.color = if health_percentage > 0.5 {
                                        Color::srgb(0.0, 1.0, 0.0)
                                    } else if health_percentage > 0.25 {
                                        Color::srgb(1.0, 0.65, 0.0)
                                    } else {
                                        Color::srgb(1.0, 0.0, 0.0)
                                    };
                                }
                            }
                        }
                    }

                    spawn_damage_text(&mut commands, damage, &asset_server);

                    // Check for player death
                    if character_health.current <= 0.0 {
                        spawn_death_screen(&mut commands, &asset_server);
                    }
                }

                // Switch back to player turn
                fight_state.current_turn = Turn::Player;
            }
        }
    }

    // Add a component for the damage text effect
    #[derive(Component)]
    struct DamageText {
        timer: Timer,
    }

    // The spawn_damage_text and animate_damage_text functions remain the same
    fn spawn_damage_text(commands: &mut Commands, damage: f32, asset_server: &Res<AssetServer>) {
        let mut color = Color::srgb(1.0, 0.0, 0.0);

        commands.spawn((
            Text2dBundle {
                text: Text::from_section(
                    format!("-{}", damage),
                    TextStyle {
                        font_size: 30.0,
                        color,
                        ..default()
                    },
                ),
                transform: Transform::from_xyz(0.0, 0.0, 10.0),
                ..default()
            },
            DamageText {
                timer: Timer::from_seconds(1.0, TimerMode::Once),
            },
        ));
    }

    fn animate_damage_text(
        mut commands: Commands,
        time: Res<Time>,
        mut query: Query<(Entity, &mut Transform, &mut Text, &mut DamageText)>,
    ) {
        for (entity, mut transform, mut text, mut damage_text) in query.iter_mut() {
            damage_text.timer.tick(time.delta());

            // Move the text upward
            transform.translation.y += 100.0 * time.delta_seconds();

            // Fade out the text
            let alpha =
                1.0 - damage_text.timer.elapsed_secs() / damage_text.timer.duration().as_secs_f32();

            // Remove the text when the timer is finished
            if damage_text.timer.finished() {
                commands.entity(entity).despawn();
            }
        }
    }
    fn handle_end_turn_button(
        mut interaction_query: Query<
            (&Interaction, &mut BackgroundColor),
            (Changed<Interaction>, With<EndTurnButton>),
        >,
        mut fight_state: ResMut<FightState>,
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        turn_state: Res<TurnState>,
    ) {
        for (interaction, mut color) in &mut interaction_query {
            match *interaction {
                Interaction::Pressed => {
                    if fight_state.current_turn == Turn::Player {
                        // Add air cards before changing turn
                        for _ in 0..turn_state.pending_air_cards {
                            spawn_card(&mut commands, CardType::Air, &asset_server);
                        }

                        fight_state.current_turn = Turn::Enemy;
                        *color = Color::srgb(0.35, 0.35, 0.35).into();
                    }
                }
                Interaction::Hovered => {
                    if fight_state.current_turn == Turn::Player {
                        *color = Color::srgb(0.25, 0.25, 0.25).into();
                    }
                }
                Interaction::None => {
                    *color = Color::srgb(0.15, 0.15, 0.15).into();
                }
            }
        }
    }

    // Add this system to update the button's appearance based on turn state
    fn update_end_turn_button(
        fight_state: Res<FightState>,
        mut button_query: Query<&mut BackgroundColor, With<EndTurnButton>>,
        mut text_query: Query<&mut Text, With<ButtonText>>,
    ) {
        if let Ok(mut color) = button_query.get_single_mut() {
            if fight_state.current_turn == Turn::Player {
                *color = Color::srgb(0.15, 0.15, 0.15).into();
            } else {
                *color = Color::srgb(0.5, 0.5, 0.5).into();
            }
        }

        if let Ok(mut text) = text_query.get_single_mut() {
            if fight_state.current_turn == Turn::Player {
                text.sections[0].value = "End Turn".to_string();
            } else {
                text.sections[0].value = "Enemy Turn".to_string();
            }
        }
    }
    // Update the chapter1_plugin to include debug system
    pub fn chapter1_plugin(app: &mut App) {
        app.init_resource::<FightState>()
            .init_resource::<TurnState>() // This line was already correct
            .add_systems(OnEnter(GameState::Chapter1), (chapter1_setup,))
            .add_systems(
                Update,
                (
                    animate_sprite,
                    update_card_hover,
                    handle_card_click,
                    process_turn,
                    update_health_bars,
                    handle_end_turn_button,
                    update_end_turn_button,
                    animate_damage_text,
                    update_death_screen,
                    process_pending_cards,
                    update_turn_state,
                    check_victory_condition, // Add this
                    update_victory_screen,
                    animate_card_play,
                    //debug_turn_state,
                )
                    .chain()
                    .run_if(in_state(GameState::Chapter1)),
            )
            .add_systems(
                OnExit(GameState::Chapter1),
                super::despawn_screen::<OnChapterOneScreen>,
            );
    }

    #[derive(Component)]
    struct PendingCards {
        card_type: CardType,
        amount: i32,
    }

    fn process_pending_cards(
        mut commands: Commands,
        pending_query: Query<(Entity, &PendingCards)>,
        mut turn_state: ResMut<TurnState>,
        asset_server: Res<AssetServer>,
    ) {
        for (entity, pending) in pending_query.iter() {
            for _ in 0..pending.amount {
                spawn_card(&mut commands, pending.card_type, &asset_server);
            }
            commands.entity(entity).despawn();
        }
    }

    fn spawn_card(commands: &mut Commands, card_type: CardType, asset_server: &Res<AssetServer>) {
        let texture = match card_type {
            CardType::Fire => asset_server.load("textures/Game Icons/Fire.png"),
            CardType::Ice => asset_server.load("textures/Game Icons/Frost.png"),
            CardType::Air => asset_server.load("textures/Game Icons/air.png"),
            CardType::Earth => asset_server.load("textures/Game Icons/Earth.png"),
            CardType::Crystal => asset_server.load("textures/Game Icons/Crystal.png"),
        };

        commands.spawn((
            ImageBundle {
                style: Style {
                    width: Val::Px(180.0),
                    height: Val::Px(250.0),
                    margin: UiRect::horizontal(Val::Px(10.0)),
                    ..default()
                },
                image: UiImage::new(texture),
                background_color: Color::WHITE.into(),
                ..default()
            },
            Interaction::None,
            Card,
            card_type,
            OriginalPosition(Vec2::new(0.0, 0.0)), // Position will need to be adjusted
            OnChapterOneScreen,
        ));
    }

    fn update_turn_state(mut fight_state: ResMut<FightState>, mut turn_state: ResMut<TurnState>) {
        // if fight_state.current_turn == Turn::Player {
        //     turn_state.cards_played_this_turn.clear();
        //     turn_state.crystal_power += 1;
        //     turn_state.turn_count += 1;
        //     turn_state.pending_air_cards = 0; // Reset pending air cards
        // }
    }

    fn chapter1_setup(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
        windows: Query<&Window>,
    ) {
        commands.insert_resource(TurnState {
            first_card_played: true,
            cards_played_this_turn: Vec::new(),
            crystal_power: 0,
            turn_count: 0,
            pending_air_cards: 0,
        });
        let window = windows.single();

        // Calculate positions
        let char_x = window.width() * -0.25;
        let char_y = window.height() * -0.75;

        // Load textures
        let texture_handle: Handle<Image> = asset_server.load("textures/intro_game_sprite.png");
        let fire_card_texture: Handle<Image> = asset_server.load("textures/Game Icons/Fire.png");
        let ice_card_texture: Handle<Image> = asset_server.load("textures/Game Icons/Frost.png");
        let air_card_texture: Handle<Image> = asset_server.load("textures/Game Icons/air.png");
        let earth_card_texture: Handle<Image> = asset_server.load("textures/Game Icons/Earth.png");
        let crystal_card_texture: Handle<Image> =
            asset_server.load("textures/Game Icons/Crystal.png");
        let forest: Handle<Image> = asset_server.load("textures/1.png");

        let side_character_texture = asset_server.load("textures/character.png");
        let monster_texture: Handle<Image> = asset_server.load("textures/monster.png");
        let monster_texture_2 = asset_server.load("textures/monster_2.png");

        let layout = TextureAtlasLayout::from_grid(UVec2::new(576, 324), 5, 64, None, None);
        let atlas_layout = atlas_layouts.add(layout);

        // Spawn main container
        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                    ..default()
                },
                OnChapterOneScreen,
            ))
            // .with_children(|parent| {
            //     // Background animation (same as before)
            //     parent
            //         .spawn(NodeBundle {
            //             style: Style {
            //                 width: Val::Vw(100.0),
            //                 height: Val::Vh(100.0),
            //                 align_items: AlignItems::Center,
            //                 justify_content: JustifyContent::Center,
            //                 ..default()
            //             },
            //             ..default()
            //         })
            //         .with_children(|parent| {
            //             parent.spawn((
            //                 SpriteSheetBundle {
            //                     texture: texture_handle,
            //                     atlas: TextureAtlas {
            //                         layout: atlas_layout,
            //                         index: 0,
            //                     },
            //                     transform: Transform::from_xyz(
            //                         -window.width() / 2.0,
            //                         -window.height() / 2.0,
            //                         1.0,
            //                     ),
            //                     sprite: Sprite {
            //                         custom_size: Some(Vec2::new(1920.0, 1080.0)),
            //                         anchor: bevy::sprite::Anchor::Center,
            //                         ..default()
            //                     },
            //                     ..default()
            //                 },
            //                 AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
            //                 AnimationIndices {
            //                     first: 0,
            //                     last: 320,
            //                 },
            //             ));
            //         });
            .with_children(|parent| {
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            width: Val::Vw(100.0),
                            height: Val::Vh(100.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|parent| {
                        parent.spawn(SpriteBundle {
                            texture: forest,
                            transform: Transform::from_xyz(
                                -window.width() / 2.0,
                                -window.height() / 2.0,
                                1.0,
                            ),
                            sprite: Sprite {
                                custom_size: Some(Vec2::new(1920.0, 1080.0)),
                                anchor: bevy::sprite::Anchor::Center,
                                ..default()
                            },
                            ..default()
                        });
                    });

                // Side character with health
                parent
                    .spawn((
                        SpriteBundle {
                            texture: side_character_texture,
                            transform: Transform::from_xyz(
                                -window.width() / 2.0 + char_x,
                                char_y,
                                2.0,
                            ),
                            sprite: Sprite {
                                custom_size: Some(Vec2::new(400.0, 400.0)),
                                anchor: bevy::sprite::Anchor::Center,
                                ..default()
                            },
                            ..default()
                        },
                        SideCharacter,
                        Health {
                            current: 100.0,
                            maximum: 100.0,
                        },
                    ))
                    .with_children(|monster| {
                        // Health bar background
                        monster
                            .spawn((
                                SpriteBundle {
                                    sprite: Sprite {
                                        color: Color::srgb(0.2, 0.2, 0.2),
                                        custom_size: Some(Vec2::new(150.0, 10.0)),
                                        ..default()
                                    },
                                    transform: Transform::from_xyz(
                                        0.0,    // Centered horizontally relative to parent
                                        -175.0, // Below sprite with 20px padding
                                        0.1,
                                    ),
                                    ..default()
                                },
                                HealthBarContainer,
                                OnChapterOneScreen,
                            ))
                            .with_children(|container| {
                                // Actual health bar
                                container.spawn((
                                    SpriteBundle {
                                        sprite: Sprite {
                                            color: Color::srgb(0.0, 1.0, 0.0),
                                            custom_size: Some(Vec2::new(150.0, 10.0)),
                                            anchor: bevy::sprite::Anchor::CenterLeft,
                                            ..default()
                                        },
                                        transform: Transform::from_xyz(-75.0, 0.0, 0.2),
                                        ..default()
                                    },
                                    HealthBar,
                                ));
                            });
                    });
                let monster1_damage = 15.0;
                let monster2_damage = 10.0;
                // Monster 1 with health
                parent
                    .spawn((
                        SpriteBundle {
                            texture: monster_texture,
                            transform: Transform::from_xyz(
                                char_x + window.width() / 8.0,
                                char_y - 75.0,
                                2.0,
                            ),
                            sprite: Sprite {
                                custom_size: Some(Vec2::new(250.0, 250.0)),
                                anchor: bevy::sprite::Anchor::Center,
                                ..default()
                            },
                            ..default()
                        },
                        Monster,
                        Health {
                            current: 40.0,
                            maximum: 40.0,
                        },
                        Damage(monster1_damage), // This monster deals 15 damage
                    ))
                    .with_children(|monster| {
                        // Spawn the black background sprite
                        monster.spawn(SpriteBundle {
                            sprite: Sprite {
                                color: Color::BLACK,
                                custom_size: Some(Vec2::new(50.0, 30.0)), // Adjust size as needed
                                ..default()
                            },
                            transform: Transform::from_xyz(0.0, 120.0, 0.0),
                            ..default()
                        });
                        // Spawn damage text above monster
                        monster.spawn((
                            Text2dBundle {
                                text: Text::from_section(
                                    format!("{}", monster1_damage),
                                    TextStyle {
                                        font_size: 24.0,
                                        color: Color::rgb(1.0, 0.0, 0.0),
                                        ..default()
                                    },
                                ),
                                transform: Transform::from_xyz(0.0, 120.0, 0.1), // Position above monster
                                ..default()
                            },
                            DamageDisplay,
                        ));
                        // Health bar background
                        monster
                            .spawn((
                                SpriteBundle {
                                    sprite: Sprite {
                                        color: Color::srgb(0.2, 0.2, 0.2),
                                        custom_size: Some(Vec2::new(150.0, 10.0)),
                                        ..default()
                                    },
                                    transform: Transform::from_xyz(
                                        0.0,    // Centered horizontally relative to parent
                                        -100.0, // Below sprite with 20px padding
                                        0.1,
                                    ),
                                    ..default()
                                },
                                HealthBarContainer,
                                OnChapterOneScreen,
                            ))
                            .with_children(|container| {
                                // Actual health bar
                                container.spawn((
                                    SpriteBundle {
                                        sprite: Sprite {
                                            color: Color::srgb(0.0, 1.0, 0.0),
                                            custom_size: Some(Vec2::new(150.0, 10.0)),
                                            anchor: bevy::sprite::Anchor::CenterLeft,
                                            ..default()
                                        },
                                        transform: Transform::from_xyz(-75.0, 0.0, 0.2),
                                        ..default()
                                    },
                                    HealthBar,
                                ));
                            });
                    });
                // Monster 2 with healthcurrent:
                parent
                    .spawn((
                        SpriteBundle {
                            texture: monster_texture_2,
                            transform: Transform::from_xyz(
                                char_x - window.width() / 8.0,
                                char_y - 75.0,
                                2.0,
                            ),
                            sprite: Sprite {
                                custom_size: Some(Vec2::new(250.0, 250.0)),
                                anchor: bevy::sprite::Anchor::Center,
                                ..default()
                            },
                            ..default()
                        },
                        Monster,
                        Health {
                            current: 40.0,
                            maximum: 40.0,
                        },
                        Damage(monster2_damage), // This monster deals 10 damage
                    ))
                    .with_children(|monster| {
                        // Spawn the black background sprite
                        monster.spawn(SpriteBundle {
                            sprite: Sprite {
                                color: Color::BLACK,
                                custom_size: Some(Vec2::new(50.0, 30.0)), // Adjust size as needed
                                ..default()
                            },
                            transform: Transform::from_xyz(0.0, 120.0, 0.0),
                            ..default()
                        });
                        // Spawn damage text above monster
                        monster.spawn((
                            Text2dBundle {
                                text: Text::from_section(
                                    format!("{}", monster2_damage),
                                    TextStyle {
                                        font_size: 24.0,
                                        color: Color::srgb(1.0, 0.0, 0.0),
                                        ..default()
                                    },
                                ),
                                transform: Transform::from_xyz(0.0, 120.0, 0.1), // Position above monster
                                ..default()
                            },
                            DamageDisplay,
                        ));
                        // Health bar background
                        monster
                            .spawn((
                                SpriteBundle {
                                    sprite: Sprite {
                                        color: Color::srgb(0.2, 0.2, 0.2),
                                        custom_size: Some(Vec2::new(150.0, 10.0)),
                                        ..default()
                                    },
                                    transform: Transform::from_xyz(
                                        0.0,    // Centered horizontally relative to parent
                                        -100.0, // Below sprite with 20px padding
                                        0.1,
                                    ),
                                    ..default()
                                },
                                HealthBarContainer,
                                OnChapterOneScreen,
                            ))
                            .with_children(|container| {
                                // Actual health bar
                                container.spawn((
                                    SpriteBundle {
                                        sprite: Sprite {
                                            color: Color::srgb(0.0, 1.0, 0.0),
                                            custom_size: Some(Vec2::new(150.0, 10.0)),
                                            anchor: bevy::sprite::Anchor::CenterLeft,
                                            ..default()
                                        },
                                        transform: Transform::from_xyz(-75.0, 0.0, 0.2),
                                        ..default()
                                    },
                                    HealthBar,
                                ));
                            });
                    });

                // Add this to the chapter1_setup function after spawning the cards container
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            right: Val::Px(20.0),
                            top: Val::Px(20.0), // Changed from top to bottom
                            width: Val::Px(150.0),
                            height: Val::Px(50.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        background_color: Color::srgb(0.15, 0.15, 0.15).into(),
                        ..default()
                    })
                    .with_children(|parent| {
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: Style {
                                        width: Val::Percent(100.0),
                                        height: Val::Percent(100.0),
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                    background_color: Color::srgb(0.15, 0.15, 0.15).into(),
                                    ..default()
                                },
                                EndTurnButton,
                            ))
                            .with_children(|parent| {
                                parent.spawn((
                                    TextBundle::from_section(
                                        "End Turn",
                                        TextStyle {
                                            font_size: 20.0,
                                            color: Color::WHITE,
                                            ..default()
                                        },
                                    ),
                                    ButtonText,
                                ));
                            });
                    });
                // Cards container
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Px(200.0),
                            position_type: PositionType::Absolute,
                            top: Val::Px(20.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(20.0),
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|parent| {
                        // Define card types and their corresponding textures
                        let cards = vec![
                            //(CardType::Air, air_card_texture.clone()),
                            (CardType::Earth, earth_card_texture.clone()),
                            (CardType::Crystal, crystal_card_texture.clone()),
                            (CardType::Fire, fire_card_texture.clone()),
                            (CardType::Ice, ice_card_texture.clone()),
                        ];

                        // Spawn three cards
                        for (i, (card_type, card_texture)) in cards.into_iter().enumerate() {
                            // Changed to into_iter()
                            let x_position = (i as f32 - 1.0) * 220.0;

                            parent.spawn((
                                ImageBundle {
                                    style: Style {
                                        width: Val::Px(180.0),
                                        height: Val::Px(250.0),
                                        margin: UiRect::horizontal(Val::Px(10.0)),
                                        ..default()
                                    },
                                    image: UiImage::new(card_texture),
                                    background_color: Color::WHITE.into(),
                                    transform: Transform::from_xyz(x_position, 0.0, 0.0),
                                    ..default()
                                },
                                Interaction::None,
                                Card,
                                card_type, // No longer a reference
                                OriginalPosition(Vec2::new(x_position, 0.0)),
                            ));
                        }
                    });
            });
    }

    fn animate_sprite(
        time: Res<Time>,
        mut query: Query<(&mut TextureAtlas, &mut AnimationTimer, &AnimationIndices)>,
    ) {
        for (mut atlas, mut timer, indices) in &mut query {
            timer.0.tick(time.delta());
            if timer.0.just_finished() {
                atlas.index = if atlas.index == indices.last {
                    indices.first
                } else {
                    atlas.index + 1
                };
            }
        }
    }

    // Add these new components and structs in the chapter1 mod
    #[derive(Component)]
    struct VictoryScreen;

    #[derive(Component)]
    struct VictoryText;

    #[derive(Component)]
    struct ReturnToMenuVictoryTimer {
        timer: Timer,
    }

    fn spawn_victory_screen(commands: &mut Commands, asset_server: &AssetServer) {
        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        position_type: PositionType::Absolute,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.0)),
                    ..default()
                },
                VictoryScreen,
                FadeInEffect {
                    timer: Timer::from_seconds(2.0, TimerMode::Once),
                },
                ReturnToMenuVictoryTimer {
                    timer: Timer::from_seconds(5.0, TimerMode::Once),
                },
            ))
            .with_children(|parent| {
                parent.spawn((
                    TextBundle::from_section(
                        "VICTORY!",
                        TextStyle {
                            font_size: 120.0,
                            color: Color::rgba(0.0, 0.8, 0.0, 0.0), // Start transparent, but green
                            ..default()
                        },
                    ),
                    VictoryText,
                    FadeInEffect {
                        timer: Timer::from_seconds(2.0, TimerMode::Once),
                    },
                ));
            });
    }

    fn update_victory_screen(
        mut commands: Commands,
        time: Res<Time>,
        mut query: Query<
            (
                Entity,
                &mut BackgroundColor,
                &mut FadeInEffect,
                &mut ReturnToMenuVictoryTimer,
            ),
            With<VictoryScreen>,
        >,
        mut text_query: Query<&mut Text, With<VictoryText>>,
        mut game_state: ResMut<NextState<GameState>>,
        mut app_exit_events: EventWriter<AppExit>,
    ) {
        for (entity, mut bg_color, mut fade, mut return_timer) in query.iter_mut() {
            fade.timer.tick(time.delta());
            let alpha = fade.timer.fraction();
            bg_color.0 = Color::rgba(0.0, 0.0, 0.0, alpha * 0.7);

            if let Ok(mut text) = text_query.get_single_mut() {
                text.sections[0].style.color = Color::rgba(0.0, 0.8, 0.0, alpha);
            }

            return_timer.timer.tick(time.delta());
            if return_timer.timer.finished() {
                game_state.set(GameState::Game2); // Change this line to transition to Chapter2
                commands.entity(entity).despawn_recursive(); // Clean up victory screen
                                                             //app_exit_events.send(AppExit::Success);
            }
        }
    }

    fn check_victory_condition(
        monster_query: Query<&Health, With<Monster>>,
        victory_screen_query: Query<(), With<VictoryScreen>>,
        mut commands: Commands,
        asset_server: Res<AssetServer>,
    ) {
        if victory_screen_query.is_empty() {
            // Only check if victory screen isn't already shown
            let all_monsters_dead = monster_query.iter().all(|health| health.current <= 0.0);

            if all_monsters_dead {
                spawn_victory_screen(&mut commands, &asset_server);
            }
        }
    }
}

mod chapter2 {
    use super::GameState;
    use bevy::app::AppExit;
    use bevy::ecs::system::ParamSet;
    use bevy::prelude::*;

    #[derive(Component, Copy, Clone, Debug, PartialEq)]
    enum CardType {
        Fire,
        Ice,
        Air,
        Earth,
        Crystal,
        // Add other types as needed
    }
    // Components
    #[derive(Component)]
    struct AnimationTimer(Timer);

    #[derive(Component)]
    struct AnimationIndices {
        first: usize,
        last: usize,
    }

    #[derive(Component)]
    struct Card;

    #[derive(Component)]
    struct OriginalPosition(Vec2);

    #[derive(Component)]
    struct OnChapterOneScreen;

    #[derive(Component)]
    struct SideCharacter;

    #[derive(Component)]
    struct Monster;

    #[derive(Component)]
    struct Health {
        current: f32,
        maximum: f32,
    }

    // Add this to your existing components if not already present
    #[derive(Component)]
    struct HealthBarContainer;

    #[derive(Component)]
    struct HealthBar;

    // Add these new components in the chapter1 mod
    #[derive(Component)]
    struct EndTurnButton;

    #[derive(Component)]
    struct ButtonText;

    #[derive(Component)]
    struct Damage(f32);

    #[derive(Component)]
    struct DeathScreen;

    #[derive(Component)]
    struct DeathText;

    #[derive(Component)]
    struct FadeInEffect {
        timer: Timer,
    }

    #[derive(Component)]
    struct ReturnToMenuTimer {
        timer: Timer,
    }

    #[derive(Component)]
    struct DamageDisplay;

    #[derive(Resource, Default)]
    struct PendingAirCards(i32);

    fn spawn_death_screen(commands: &mut Commands, asset_server: &AssetServer) {
        // Main container
        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        position_type: PositionType::Absolute,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.0)),
                    ..default()
                },
                DeathScreen,
                FadeInEffect {
                    timer: Timer::from_seconds(2.0, TimerMode::Once),
                },
                // Add a timer to return to menu after 5 seconds
                ReturnToMenuTimer {
                    timer: Timer::from_seconds(5.0, TimerMode::Once),
                },
            ))
            .with_children(|parent| {
                // "YOU DIED" text
                parent.spawn((
                    TextBundle::from_section(
                        "YOU DIED",
                        TextStyle {
                            font_size: 120.0,
                            color: Color::rgba(0.8, 0.0, 0.0, 0.0), // Start transparent
                            ..default()
                        },
                    ),
                    DeathText,
                    FadeInEffect {
                        timer: Timer::from_seconds(2.0, TimerMode::Once),
                    },
                ));
            });
    }

    // Update the death screen system to handle both fade-in and menu transition
    fn update_death_screen(
        mut commands: Commands,
        time: Res<Time>,
        mut query: Query<
            (
                Entity,
                &mut BackgroundColor,
                &mut FadeInEffect,
                &mut ReturnToMenuTimer,
            ),
            With<DeathScreen>,
        >,
        mut text_query: Query<&mut Text, With<DeathText>>,
        mut game_state: ResMut<NextState<GameState>>,
        mut app_exit_events: EventWriter<AppExit>,
    ) {
        for (entity, mut bg_color, mut fade, mut return_timer) in query.iter_mut() {
            // Update fade effect
            fade.timer.tick(time.delta());
            let alpha = fade.timer.fraction();
            bg_color.0 = Color::rgba(0.0, 0.0, 0.0, alpha * 0.7);

            // Update text color
            if let Ok(mut text) = text_query.get_single_mut() {
                text.sections[0].style.color = Color::rgba(0.8, 0.0, 0.0, alpha);
            }

            // Update return timer
            return_timer.timer.tick(time.delta());
            if return_timer.timer.finished() {
                app_exit_events.send(AppExit::Success);
            }
        }
    }
    // Modify TurnState to include pending air cards
    #[derive(Resource)]
    struct TurnState {
        first_card_played: bool,
        cards_played_this_turn: Vec<CardType>,
        crystal_power: i32,
        turn_count: i32,
        pending_air_cards: i32,
    }

    impl Default for TurnState {
        fn default() -> Self {
            Self {
                first_card_played: true,
                cards_played_this_turn: Vec::new(),
                crystal_power: 0,
                turn_count: 0,
                pending_air_cards: 0, // Initialize pending_air_cards to 0
            }
        }
    }

    // Constants for base damage values
    const FIRE_BASE_DAMAGE: f32 = 8.0;
    const FIRE_FIRST_CARD_BONUS: f32 = 7.0;
    const ICE_BASE_DAMAGE: f32 = 6.0;
    const CRYSTAL_BASE_DAMAGE: f32 = 4.0;
    const AIR_BASE_DAMAGE: f32 = 2.0;
    const EARTH_BASE_DAMAGE: f32 = 5.0;

    fn update_health_bars(
        query: Query<(&Health, &Children), Without<HealthBar>>,
        mut health_bar_query: Query<&mut Sprite, With<HealthBar>>,
    ) {
        for (health, children) in query.iter() {
            for child in children.iter() {
                if let Ok(mut bar_sprite) = health_bar_query.get_mut(*child) {
                    // Update health bar width based on current health
                    let bar_width = 100.0;
                    let health_percentage = health.current / health.maximum;

                    bar_sprite.custom_size = Some(Vec2::new(
                        bar_width * health_percentage,
                        bar_sprite.custom_size.unwrap_or(Vec2::ZERO).y,
                    ));

                    // Update color based on health percentage
                    bar_sprite.color = if health_percentage > 0.5 {
                        Color::srgb(0.0, 1.0, 0.0) // Green: rgb(0, 255, 0)
                    } else if health_percentage > 0.25 {
                        Color::srgb(1.0, 0.65, 0.0) // Orange: rgb(255, 165, 0)
                    } else {
                        Color::srgb(1.0, 0.0, 0.0) // Red: rgb(255, 0, 0)
                    };
                }
            }
        }
    }
    #[derive(Resource)]
    struct FightState {
        current_turn: Turn,
        selected_card: Option<usize>,
    }

    #[derive(PartialEq)]
    enum Turn {
        Player,
        Enemy,
    }

    impl Default for FightState {
        fn default() -> Self {
            Self {
                current_turn: Turn::Player,
                selected_card: None,
            }
        }
    }

    // Update the card hover system to use FightState
    fn update_card_hover(
        mut card_query: Query<
            (
                &Interaction,
                &mut Transform,
                &OriginalPosition,
                &mut Style,
                Entity,
            ),
            (With<Card>, Changed<Interaction>),
        >,
        mut commands: Commands,
        fight_state: Res<FightState>,
    ) {
        for (interaction, mut transform, original_pos, mut style, entity) in card_query.iter_mut() {
            match *interaction {
                Interaction::Hovered => {
                    if fight_state.current_turn == Turn::Player {
                        transform.translation.y = original_pos.0.y + 30.0;
                        style.width = Val::Px(200.0);
                        style.height = Val::Px(280.0);
                    }
                }
                _ => {
                    transform.translation.y = original_pos.0.y;
                    style.width = Val::Px(180.0);
                    style.height = Val::Px(250.0);
                }
            }
        }
    }

    fn handle_card_click(
        mut commands: Commands,
        mut card_query: Query<
            (&Interaction, Entity, &CardType),
            (Changed<Interaction>, With<Card>),
        >,
        cards_in_hand: Query<Entity, With<Card>>, // Query to count cards
        mut fight_state: ResMut<FightState>,
        mut turn_state: ResMut<TurnState>,
        mut monster_query: Query<(Entity, &mut Health, &Children), With<Monster>>,
        health_container_query: Query<&Children, With<HealthBarContainer>>,
        mut health_bar_query: Query<&mut Sprite, With<HealthBar>>,
        asset_server: Res<AssetServer>,
    ) {
        if fight_state.current_turn != Turn::Player {
            return;
        }

        for (interaction, card_entity, card_type) in card_query.iter() {
            if *interaction == Interaction::Pressed {
                println!("First card played status: {}", turn_state.first_card_played);

                // Calculate damage based on whether this is the first card
                let is_first = turn_state.first_card_played;
                let cards_in_hand_count = cards_in_hand.iter().count() as f32; // Get count here

                let damage = if *card_type == CardType::Fire && is_first {
                    println!("Fire card played as first card! Enhanced damage!");
                    FIRE_BASE_DAMAGE + FIRE_FIRST_CARD_BONUS
                } else {
                    match card_type {
                        CardType::Fire => {
                            println!("Fire card played but not first");
                            FIRE_BASE_DAMAGE
                        }
                        CardType::Ice => {
                            let mut damage = ICE_BASE_DAMAGE;

                            if let Some(last_card) = turn_state.cards_played_this_turn.last() {
                                if matches!(last_card, CardType::Fire) {
                                    damage *= 2.0;
                                }
                            }

                            if turn_state
                                .cards_played_this_turn
                                .iter()
                                .any(|c| matches!(c, CardType::Earth))
                            {
                                damage = 0.0;
                            }

                            damage
                        }
                        CardType::Crystal => {
                            let effects_bonus =
                                (turn_state.cards_played_this_turn.len() as f32) * 2.0;
                            let turn_bonus = turn_state.crystal_power as f32;
                            CRYSTAL_BASE_DAMAGE + effects_bonus + turn_bonus
                        }
                        CardType::Air => AIR_BASE_DAMAGE,
                        CardType::Earth => {
                            let turn_bonus = turn_state.turn_count as f32;
                            EARTH_BASE_DAMAGE + cards_in_hand_count + turn_bonus
                            // Use the count here
                        }
                    }
                };

                // Deal damage
                for (entity, mut monster_health, children) in monster_query.iter_mut() {
                    monster_health.current = (monster_health.current - damage).max(0.0);
                    println!("Dealing {} damage. First card: {}", damage, is_first);
                    spawn_damage_text(&mut commands, damage, &asset_server);
                    // Update monster's health bar
                    for child in children.iter() {
                        if let Ok(container_children) = health_container_query.get(*child) {
                            for health_bar_entity in container_children.iter() {
                                if let Ok(mut bar_sprite) =
                                    health_bar_query.get_mut(*health_bar_entity)
                                {
                                    let bar_width = 150.0; // Match the width set in chapter1_setup
                                    let health_percentage =
                                        monster_health.current / monster_health.maximum;

                                    bar_sprite.custom_size = Some(Vec2::new(
                                        bar_width * health_percentage,
                                        bar_sprite.custom_size.unwrap_or(Vec2::ZERO).y,
                                    ));

                                    // Update color based on health percentage
                                    bar_sprite.color = if health_percentage > 0.5 {
                                        Color::srgb(0.0, 1.0, 0.0) // Green
                                    } else if health_percentage > 0.25 {
                                        Color::srgb(1.0, 0.65, 0.0) // Orange
                                    } else {
                                        Color::srgb(1.0, 0.0, 0.0) // Red
                                    };
                                }
                            }
                        }
                    }

                    // If monster dies, despawn it
                    if monster_health.current <= 0.0 {
                        commands.entity(entity).despawn_recursive();
                    }
                }

                // Handle special card effects and cleanup
                if matches!(card_type, CardType::Air) {
                    turn_state.pending_air_cards += 2;
                }

                // Update turn state BEFORE destroying the card
                turn_state.cards_played_this_turn.push(*card_type);
                turn_state.first_card_played = false;
                println!("Set first_card_played to false");

                // Destroy the played card
                commands.entity(card_entity).despawn_recursive();

                break;
            }
        }
    }

    // Add this system to help debug the turn state
    // Add this system to help debug the turn state
    fn debug_turn_state(turn_state: Res<TurnState>, fight_state: Res<FightState>) {
        println!(
            "Turn State - First card: {}, Cards played: {}",
            turn_state.first_card_played,
            turn_state.cards_played_this_turn.len(),
        );
    }

    fn process_turn(
        mut fight_state: ResMut<FightState>,
        mut query_set: ParamSet<(
            Query<(&mut Health, &Children), With<SideCharacter>>,
            Query<(&Health, &Damage), With<Monster>>,
        )>,
        health_container_query: Query<&Children, With<HealthBarContainer>>,
        mut health_bar_query: Query<&mut Sprite, With<HealthBar>>,
        asset_server: Res<AssetServer>,
        mut commands: Commands,
    ) {
        if fight_state.current_turn == Turn::Enemy {
            // First, collect all living monsters and their damage
            let monster_attacks: Vec<f32> = query_set
                .p1()
                .iter()
                .filter(|(health, _)| health.current > 0.0)
                .map(|(_, damage)| damage.0)
                .collect();

            // Then apply damage to the player
            if let Ok((mut character_health, children)) = query_set.p0().get_single_mut() {
                for damage in monster_attacks {
                    character_health.current = (character_health.current - damage).max(0.0);
                    println!(
                        "Player health: {}/{}",
                        character_health.current, character_health.maximum
                    );

                    // Health bar update logic using nested queries
                    for child in children.iter() {
                        if let Ok(container_children) = health_container_query.get(*child) {
                            for health_bar_entity in container_children.iter() {
                                if let Ok(mut bar_sprite) =
                                    health_bar_query.get_mut(*health_bar_entity)
                                {
                                    let bar_width = 150.0;
                                    let health_percentage =
                                        character_health.current / character_health.maximum;

                                    bar_sprite.custom_size = Some(Vec2::new(
                                        bar_width * health_percentage,
                                        bar_sprite.custom_size.unwrap_or(Vec2::ZERO).y,
                                    ));

                                    bar_sprite.color = if health_percentage > 0.5 {
                                        Color::srgb(0.0, 1.0, 0.0)
                                    } else if health_percentage > 0.25 {
                                        Color::srgb(1.0, 0.65, 0.0)
                                    } else {
                                        Color::srgb(1.0, 0.0, 0.0)
                                    };
                                }
                            }
                        }
                    }

                    spawn_damage_text(&mut commands, damage, &asset_server);

                    // Check for player death
                    if character_health.current <= 0.0 {
                        spawn_death_screen(&mut commands, &asset_server);
                    }
                }

                // Switch back to player turn
                fight_state.current_turn = Turn::Player;
            }
        }
    }

    // Add a component for the damage text effect
    #[derive(Component)]
    struct DamageText {
        timer: Timer,
    }

    // The spawn_damage_text and animate_damage_text functions remain the same
    fn spawn_damage_text(commands: &mut Commands, damage: f32, asset_server: &Res<AssetServer>) {
        let mut color = Color::srgb(1.0, 0.0, 0.0);

        commands.spawn((
            Text2dBundle {
                text: Text::from_section(
                    format!("-{}", damage),
                    TextStyle {
                        font_size: 30.0,
                        color,
                        ..default()
                    },
                ),
                transform: Transform::from_xyz(0.0, 0.0, 10.0),
                ..default()
            },
            DamageText {
                timer: Timer::from_seconds(1.0, TimerMode::Once),
            },
        ));
    }

    fn animate_damage_text(
        mut commands: Commands,
        time: Res<Time>,
        mut query: Query<(Entity, &mut Transform, &mut Text, &mut DamageText)>,
    ) {
        for (entity, mut transform, mut text, mut damage_text) in query.iter_mut() {
            damage_text.timer.tick(time.delta());

            // Move the text upward
            transform.translation.y += 100.0 * time.delta_seconds();

            // Fade out the text
            let alpha =
                1.0 - damage_text.timer.elapsed_secs() / damage_text.timer.duration().as_secs_f32();

            // Remove the text when the timer is finished
            if damage_text.timer.finished() {
                commands.entity(entity).despawn();
            }
        }
    }
    fn handle_end_turn_button(
        mut interaction_query: Query<
            (&Interaction, &mut BackgroundColor),
            (Changed<Interaction>, With<EndTurnButton>),
        >,
        mut fight_state: ResMut<FightState>,
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        turn_state: Res<TurnState>,
    ) {
        for (interaction, mut color) in &mut interaction_query {
            match *interaction {
                Interaction::Pressed => {
                    if fight_state.current_turn == Turn::Player {
                        // Add air cards before changing turn
                        for _ in 0..turn_state.pending_air_cards {
                            spawn_card(&mut commands, CardType::Air, &asset_server);
                        }

                        fight_state.current_turn = Turn::Enemy;
                        *color = Color::srgb(0.35, 0.35, 0.35).into();
                    }
                }
                Interaction::Hovered => {
                    if fight_state.current_turn == Turn::Player {
                        *color = Color::srgb(0.25, 0.25, 0.25).into();
                    }
                }
                Interaction::None => {
                    *color = Color::srgb(0.15, 0.15, 0.15).into();
                }
            }
        }
    }

    // Add this system to update the button's appearance based on turn state
    fn update_end_turn_button(
        fight_state: Res<FightState>,
        mut button_query: Query<&mut BackgroundColor, With<EndTurnButton>>,
        mut text_query: Query<&mut Text, With<ButtonText>>,
    ) {
        if let Ok(mut color) = button_query.get_single_mut() {
            if fight_state.current_turn == Turn::Player {
                *color = Color::srgb(0.15, 0.15, 0.15).into();
            } else {
                *color = Color::srgb(0.5, 0.5, 0.5).into();
            }
        }

        if let Ok(mut text) = text_query.get_single_mut() {
            if fight_state.current_turn == Turn::Player {
                text.sections[0].value = "End Turn".to_string();
            } else {
                text.sections[0].value = "Enemy Turn".to_string();
            }
        }
    }
    // Update the chapter1_plugin to include debug system
    pub fn chapter2_plugin(app: &mut App) {
        app.init_resource::<FightState>()
            .init_resource::<TurnState>() // This line was already correct
            .add_systems(OnEnter(GameState::Chapter2), (chapter1_setup,))
            .add_systems(
                Update,
                (
                    animate_sprite,
                    update_card_hover,
                    handle_card_click,
                    process_turn,
                    update_health_bars,
                    handle_end_turn_button,
                    update_end_turn_button,
                    animate_damage_text,
                    update_death_screen,
                    process_pending_cards,
                    update_turn_state,
                    check_victory_condition, // Add this
                    update_victory_screen,
                    //debug_turn_state,
                )
                    .chain()
                    .run_if(in_state(GameState::Chapter2)),
            )
            .add_systems(
                OnExit(GameState::Chapter2),
                super::despawn_screen::<OnChapterOneScreen>,
            );
    }

    #[derive(Component)]
    struct PendingCards {
        card_type: CardType,
        amount: i32,
    }

    fn process_pending_cards(
        mut commands: Commands,
        pending_query: Query<(Entity, &PendingCards)>,
        mut turn_state: ResMut<TurnState>,
        asset_server: Res<AssetServer>,
    ) {
        for (entity, pending) in pending_query.iter() {
            for _ in 0..pending.amount {
                spawn_card(&mut commands, pending.card_type, &asset_server);
            }
            commands.entity(entity).despawn();
        }
    }

    fn spawn_card(commands: &mut Commands, card_type: CardType, asset_server: &Res<AssetServer>) {
        let texture = match card_type {
            CardType::Fire => asset_server.load("textures/Game Icons/Fire.png"),
            CardType::Ice => asset_server.load("textures/Game Icons/Frost.png"),
            CardType::Air => asset_server.load("textures/Game Icons/air.png"),
            CardType::Earth => asset_server.load("textures/Game Icons/Earth.png"),
            CardType::Crystal => asset_server.load("textures/Game Icons/Crystal.png"),
        };

        commands.spawn((
            ImageBundle {
                style: Style {
                    width: Val::Px(180.0),
                    height: Val::Px(250.0),
                    margin: UiRect::horizontal(Val::Px(10.0)),
                    ..default()
                },
                image: UiImage::new(texture),
                background_color: Color::WHITE.into(),
                ..default()
            },
            Interaction::None,
            Card,
            card_type,
            OriginalPosition(Vec2::new(0.0, 0.0)), // Position will need to be adjusted
            OnChapterOneScreen,
        ));
    }

    fn update_turn_state(mut fight_state: ResMut<FightState>, mut turn_state: ResMut<TurnState>) {
        // if fight_state.current_turn == Turn::Player {
        //     turn_state.cards_played_this_turn.clear();
        //     turn_state.crystal_power += 1;
        //     turn_state.turn_count += 1;
        //     turn_state.pending_air_cards = 0; // Reset pending air cards
        // }
    }

    fn chapter1_setup(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
        windows: Query<&Window>,
    ) {
        commands.insert_resource(TurnState {
            first_card_played: true,
            cards_played_this_turn: Vec::new(),
            crystal_power: 0,
            turn_count: 0,
            pending_air_cards: 0,
        });
        let window = windows.single();

        // Calculate positions
        let char_x = window.width() * -0.25;
        let char_y = window.height() * -0.75;

        // Load textures
        let texture_handle: Handle<Image> = asset_server.load("textures/intro_game_sprite.png");
        let fire_card_texture: Handle<Image> = asset_server.load("textures/Game Icons/Fire.png");
        let ice_card_texture: Handle<Image> = asset_server.load("textures/Game Icons/Frost.png");
        let air_card_texture: Handle<Image> = asset_server.load("textures/Game Icons/air.png");
        let earth_card_texture: Handle<Image> = asset_server.load("textures/Game Icons/Earth.png");
        let crystal_card_texture: Handle<Image> =
            asset_server.load("textures/Game Icons/Crystal.png");
        let forest: Handle<Image> = asset_server.load("textures/2.png");

        let side_character_texture = asset_server.load("textures/character.png");
        let monster_texture: Handle<Image> = asset_server.load("textures/knight.png");
        let monster_texture_2 = asset_server.load("textures/knight.png");

        let layout = TextureAtlasLayout::from_grid(UVec2::new(576, 324), 5, 64, None, None);
        let atlas_layout = atlas_layouts.add(layout);

        // Spawn main container
        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                    ..default()
                },
                OnChapterOneScreen,
            ))
            // .with_children(|parent| {
            //     // Background animation (same as before)
            //     parent
            //         .spawn(NodeBundle {
            //             style: Style {
            //                 width: Val::Vw(100.0),
            //                 height: Val::Vh(100.0),
            //                 align_items: AlignItems::Center,
            //                 justify_content: JustifyContent::Center,
            //                 ..default()
            //             },
            //             ..default()
            //         })
            //         .with_children(|parent| {
            //             parent.spawn((
            //                 SpriteSheetBundle {
            //                     texture: texture_handle,
            //                     atlas: TextureAtlas {
            //                         layout: atlas_layout,
            //                         index: 0,
            //                     },
            //                     transform: Transform::from_xyz(
            //                         -window.width() / 2.0,
            //                         -window.height() / 2.0,
            //                         1.0,
            //                     ),
            //                     sprite: Sprite {
            //                         custom_size: Some(Vec2::new(1920.0, 1080.0)),
            //                         anchor: bevy::sprite::Anchor::Center,
            //                         ..default()
            //                     },
            //                     ..default()
            //                 },
            //                 AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
            //                 AnimationIndices {
            //                     first: 0,
            //                     last: 320,
            //                 },
            //             ));
            //         });
            .with_children(|parent| {
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            width: Val::Vw(100.0),
                            height: Val::Vh(100.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|parent| {
                        parent.spawn(SpriteBundle {
                            texture: forest,
                            transform: Transform::from_xyz(
                                -window.width() / 2.0,
                                -window.height() / 2.0,
                                1.0,
                            ),
                            sprite: Sprite {
                                custom_size: Some(Vec2::new(1920.0, 1080.0)),
                                anchor: bevy::sprite::Anchor::Center,
                                ..default()
                            },
                            ..default()
                        });
                    });

                // Side character with health
                parent
                    .spawn((
                        SpriteBundle {
                            texture: side_character_texture,
                            transform: Transform::from_xyz(
                                -window.width() / 2.0 + char_x,
                                char_y,
                                2.0,
                            ),
                            sprite: Sprite {
                                custom_size: Some(Vec2::new(400.0, 400.0)),
                                anchor: bevy::sprite::Anchor::Center,
                                ..default()
                            },
                            ..default()
                        },
                        SideCharacter,
                        Health {
                            current: 100.0,
                            maximum: 100.0,
                        },
                    ))
                    .with_children(|monster| {
                        // Health bar background
                        monster
                            .spawn((
                                SpriteBundle {
                                    sprite: Sprite {
                                        color: Color::srgb(0.2, 0.2, 0.2),
                                        custom_size: Some(Vec2::new(150.0, 10.0)),
                                        ..default()
                                    },
                                    transform: Transform::from_xyz(
                                        0.0,    // Centered horizontally relative to parent
                                        -175.0, // Below sprite with 20px padding
                                        0.1,
                                    ),
                                    ..default()
                                },
                                HealthBarContainer,
                                OnChapterOneScreen,
                            ))
                            .with_children(|container| {
                                // Actual health bar
                                container.spawn((
                                    SpriteBundle {
                                        sprite: Sprite {
                                            color: Color::srgb(0.0, 1.0, 0.0),
                                            custom_size: Some(Vec2::new(150.0, 10.0)),
                                            anchor: bevy::sprite::Anchor::CenterLeft,
                                            ..default()
                                        },
                                        transform: Transform::from_xyz(-75.0, 0.0, 0.2),
                                        ..default()
                                    },
                                    HealthBar,
                                ));
                            });
                    });
                let monster1_damage = 25.0;
                let monster2_damage = 10.0;
                // Monster 1 with health
                parent
                    .spawn((
                        SpriteBundle {
                            texture: monster_texture,
                            transform: Transform::from_xyz(
                                char_x + window.width() / 8.0,
                                char_y,
                                2.0,
                            ),
                            sprite: Sprite {
                                custom_size: Some(Vec2::new(400.0, 400.0)),
                                anchor: bevy::sprite::Anchor::Center,
                                ..default()
                            },
                            ..default()
                        },
                        Monster,
                        Health {
                            current: 21.0,
                            maximum: 21.0,
                        },
                        Damage(monster1_damage), // This monster deals 15 damage
                    ))
                    .with_children(|monster| {
                        // Spawn the black background sprite
                        monster.spawn(SpriteBundle {
                            sprite: Sprite {
                                color: Color::BLACK,
                                custom_size: Some(Vec2::new(50.0, 30.0)), // Adjust size as needed
                                ..default()
                            },
                            transform: Transform::from_xyz(0.0, 120.0, 0.0),
                            ..default()
                        });
                        // Spawn damage text above monster
                        monster.spawn((
                            Text2dBundle {
                                text: Text::from_section(
                                    format!("{}", monster1_damage),
                                    TextStyle {
                                        font_size: 24.0,
                                        color: Color::rgb(1.0, 0.0, 0.0),
                                        ..default()
                                    },
                                ),
                                transform: Transform::from_xyz(0.0, 120.0, 0.1), // Position above monster
                                ..default()
                            },
                            DamageDisplay,
                        ));
                        // Health bar background
                        monster
                            .spawn((
                                SpriteBundle {
                                    sprite: Sprite {
                                        color: Color::srgb(0.2, 0.2, 0.2),
                                        custom_size: Some(Vec2::new(150.0, 10.0)),
                                        ..default()
                                    },
                                    transform: Transform::from_xyz(
                                        0.0,    // Centered horizontally relative to parent
                                        -170.0, // Below sprite with 20px padding
                                        0.1,
                                    ),
                                    ..default()
                                },
                                HealthBarContainer,
                                OnChapterOneScreen,
                            ))
                            .with_children(|container| {
                                // Actual health bar
                                container.spawn((
                                    SpriteBundle {
                                        sprite: Sprite {
                                            color: Color::srgb(0.0, 1.0, 0.0),
                                            custom_size: Some(Vec2::new(150.0, 10.0)),
                                            anchor: bevy::sprite::Anchor::CenterLeft,
                                            ..default()
                                        },
                                        transform: Transform::from_xyz(-75.0, 0.0, 0.2),
                                        ..default()
                                    },
                                    HealthBar,
                                ));
                            });
                    });
                // Monster 2 with healthcurrent:
                parent
                    .spawn((
                        SpriteBundle {
                            texture: monster_texture_2,
                            transform: Transform::from_xyz(
                                char_x - window.width() / 8.0,
                                char_y,
                                2.0,
                            ),
                            sprite: Sprite {
                                custom_size: Some(Vec2::new(400.0, 400.0)),
                                anchor: bevy::sprite::Anchor::Center,
                                ..default()
                            },
                            ..default()
                        },
                        Monster,
                        Health {
                            current: 21.0,
                            maximum: 21.0,
                        },
                        Damage(monster2_damage), // This monster deals 10 damage
                    ))
                    .with_children(|monster| {
                        // Spawn the black background sprite
                        monster.spawn(SpriteBundle {
                            sprite: Sprite {
                                color: Color::BLACK,
                                custom_size: Some(Vec2::new(50.0, 30.0)), // Adjust size as needed
                                ..default()
                            },
                            transform: Transform::from_xyz(0.0, 120.0, 0.0),
                            ..default()
                        });
                        // Spawn damage text above monster
                        monster.spawn((
                            Text2dBundle {
                                text: Text::from_section(
                                    format!("{}", monster2_damage),
                                    TextStyle {
                                        font_size: 24.0,
                                        color: Color::srgb(1.0, 0.0, 0.0),
                                        ..default()
                                    },
                                ),
                                transform: Transform::from_xyz(0.0, 120.0, 0.1), // Position above monster
                                ..default()
                            },
                            DamageDisplay,
                        ));
                        // Health bar background
                        monster
                            .spawn((
                                SpriteBundle {
                                    sprite: Sprite {
                                        color: Color::srgb(0.2, 0.2, 0.2),
                                        custom_size: Some(Vec2::new(150.0, 10.0)),
                                        ..default()
                                    },
                                    transform: Transform::from_xyz(
                                        0.0,    // Centered horizontally relative to parent
                                        -170.0, // Below sprite with 20px padding
                                        0.1,
                                    ),
                                    ..default()
                                },
                                HealthBarContainer,
                                OnChapterOneScreen,
                            ))
                            .with_children(|container| {
                                // Actual health bar
                                container.spawn((
                                    SpriteBundle {
                                        sprite: Sprite {
                                            color: Color::srgb(0.0, 1.0, 0.0),
                                            custom_size: Some(Vec2::new(150.0, 10.0)),
                                            anchor: bevy::sprite::Anchor::CenterLeft,
                                            ..default()
                                        },
                                        transform: Transform::from_xyz(-75.0, 0.0, 0.2),
                                        ..default()
                                    },
                                    HealthBar,
                                ));
                            });
                    });

                // Add this to the chapter1_setup function after spawning the cards container
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            right: Val::Px(20.0),
                            top: Val::Px(20.0), // Changed from top to bottom
                            width: Val::Px(150.0),
                            height: Val::Px(50.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        background_color: Color::srgb(0.15, 0.15, 0.15).into(),
                        ..default()
                    })
                    .with_children(|parent| {
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: Style {
                                        width: Val::Percent(100.0),
                                        height: Val::Percent(100.0),
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                    background_color: Color::srgb(0.15, 0.15, 0.15).into(),
                                    ..default()
                                },
                                EndTurnButton,
                            ))
                            .with_children(|parent| {
                                parent.spawn((
                                    TextBundle::from_section(
                                        "End Turn",
                                        TextStyle {
                                            font_size: 20.0,
                                            color: Color::WHITE,
                                            ..default()
                                        },
                                    ),
                                    ButtonText,
                                ));
                            });
                    });
                // Cards container
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Px(200.0),
                            position_type: PositionType::Absolute,
                            top: Val::Px(20.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(20.0),
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|parent| {
                        // Define card types and their corresponding textures
                        let cards = vec![
                            (CardType::Ice, ice_card_texture.clone()),
                            //(CardType::Air, air_card_texture.clone()),
                            (CardType::Earth, earth_card_texture.clone()),
                            (CardType::Crystal, crystal_card_texture.clone()),
                        ];

                        // Spawn three cards
                        for (i, (card_type, card_texture)) in cards.into_iter().enumerate() {
                            // Changed to into_iter()
                            let x_position = (i as f32 - 1.0) * 220.0;

                            parent.spawn((
                                ImageBundle {
                                    style: Style {
                                        width: Val::Px(180.0),
                                        height: Val::Px(250.0),
                                        margin: UiRect::horizontal(Val::Px(10.0)),
                                        ..default()
                                    },
                                    image: UiImage::new(card_texture),
                                    background_color: Color::WHITE.into(),
                                    transform: Transform::from_xyz(x_position, 0.0, 0.0),
                                    ..default()
                                },
                                Interaction::None,
                                Card,
                                card_type, // No longer a reference
                                OriginalPosition(Vec2::new(x_position, 0.0)),
                            ));
                        }
                    });
            });
    }

    fn animate_sprite(
        time: Res<Time>,
        mut query: Query<(&mut TextureAtlas, &mut AnimationTimer, &AnimationIndices)>,
    ) {
        for (mut atlas, mut timer, indices) in &mut query {
            timer.0.tick(time.delta());
            if timer.0.just_finished() {
                atlas.index = if atlas.index == indices.last {
                    indices.first
                } else {
                    atlas.index + 1
                };
            }
        }
    }

    // Add these new components and structs in the chapter1 mod
    #[derive(Component)]
    struct VictoryScreen;

    #[derive(Component)]
    struct VictoryText;

    #[derive(Component)]
    struct ReturnToMenuVictoryTimer {
        timer: Timer,
    }

    fn spawn_victory_screen(commands: &mut Commands, asset_server: &AssetServer) {
        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        position_type: PositionType::Absolute,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.0)),
                    ..default()
                },
                VictoryScreen,
                FadeInEffect {
                    timer: Timer::from_seconds(2.0, TimerMode::Once),
                },
                ReturnToMenuVictoryTimer {
                    timer: Timer::from_seconds(5.0, TimerMode::Once),
                },
            ))
            .with_children(|parent| {
                parent.spawn((
                    TextBundle::from_section(
                        "VICTORY!",
                        TextStyle {
                            font_size: 120.0,
                            color: Color::rgba(0.0, 0.8, 0.0, 0.0), // Start transparent, but green
                            ..default()
                        },
                    ),
                    VictoryText,
                    FadeInEffect {
                        timer: Timer::from_seconds(2.0, TimerMode::Once),
                    },
                ));
            });
    }

    fn update_victory_screen(
        mut commands: Commands,
        time: Res<Time>,
        mut query: Query<
            (
                Entity,
                &mut BackgroundColor,
                &mut FadeInEffect,
                &mut ReturnToMenuVictoryTimer,
            ),
            With<VictoryScreen>,
        >,
        mut text_query: Query<&mut Text, With<VictoryText>>,
        mut game_state: ResMut<NextState<GameState>>,
        mut app_exit_events: EventWriter<AppExit>,
    ) {
        for (entity, mut bg_color, mut fade, mut return_timer) in query.iter_mut() {
            fade.timer.tick(time.delta());
            let alpha = fade.timer.fraction();
            bg_color.0 = Color::rgba(0.0, 0.0, 0.0, alpha * 0.7);

            if let Ok(mut text) = text_query.get_single_mut() {
                text.sections[0].style.color = Color::rgba(0.0, 0.8, 0.0, alpha);
            }

            return_timer.timer.tick(time.delta());
            if return_timer.timer.finished() {
                game_state.set(GameState::Game3);
                commands.entity(entity).despawn_recursive(); // Clean up victory screen
                                                             //app_exit_events.send(AppExit::Success);
            }
        }
    }

    fn check_victory_condition(
        monster_query: Query<&Health, With<Monster>>,
        victory_screen_query: Query<(), With<VictoryScreen>>,
        mut commands: Commands,
        asset_server: Res<AssetServer>,
    ) {
        if victory_screen_query.is_empty() {
            // Only check if victory screen isn't already shown
            let all_monsters_dead = monster_query.iter().all(|health| health.current <= 0.0);

            if all_monsters_dead {
                spawn_victory_screen(&mut commands, &asset_server);
            }
        }
    }
}

mod chapter3 {
    use super::GameState;
    use bevy::app::AppExit;
    use bevy::ecs::system::ParamSet;
    use bevy::prelude::*;

    #[derive(Component, Copy, Clone, Debug, PartialEq)]
    enum CardType {
        Fire,
        Ice,
        Air,
        Earth,
        Crystal,
        // Add other types as needed
    }
    // Components
    #[derive(Component)]
    struct AnimationTimer(Timer);

    #[derive(Component)]
    struct AnimationIndices {
        first: usize,
        last: usize,
    }

    #[derive(Component)]
    struct Card;

    #[derive(Component)]
    struct OriginalPosition(Vec2);

    #[derive(Component)]
    struct OnChapterOneScreen;

    #[derive(Component)]
    struct SideCharacter;

    #[derive(Component)]
    struct Monster;

    #[derive(Component)]
    struct Health {
        current: f32,
        maximum: f32,
    }

    // Add this to your existing components if not already present
    #[derive(Component)]
    struct HealthBarContainer;

    #[derive(Component)]
    struct HealthBar;

    // Add these new components in the chapter1 mod
    #[derive(Component)]
    struct EndTurnButton;

    #[derive(Component)]
    struct ButtonText;

    #[derive(Component)]
    struct Damage(f32);

    #[derive(Component)]
    struct DeathScreen;

    #[derive(Component)]
    struct DeathText;

    #[derive(Component)]
    struct FadeInEffect {
        timer: Timer,
    }

    #[derive(Component)]
    struct ReturnToMenuTimer {
        timer: Timer,
    }

    #[derive(Component)]
    struct DamageDisplay;

    #[derive(Resource, Default)]
    struct PendingAirCards(i32);

    fn spawn_death_screen(commands: &mut Commands, asset_server: &AssetServer) {
        // Main container
        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        position_type: PositionType::Absolute,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.0)),
                    ..default()
                },
                DeathScreen,
                FadeInEffect {
                    timer: Timer::from_seconds(2.0, TimerMode::Once),
                },
                // Add a timer to return to menu after 5 seconds
                ReturnToMenuTimer {
                    timer: Timer::from_seconds(5.0, TimerMode::Once),
                },
            ))
            .with_children(|parent| {
                // "YOU DIED" text
                parent.spawn((
                    TextBundle::from_section(
                        "YOU DIED",
                        TextStyle {
                            font_size: 120.0,
                            color: Color::rgba(0.8, 0.0, 0.0, 0.0), // Start transparent
                            ..default()
                        },
                    ),
                    DeathText,
                    FadeInEffect {
                        timer: Timer::from_seconds(2.0, TimerMode::Once),
                    },
                ));
            });
    }

    // Update the death screen system to handle both fade-in and menu transition
    fn update_death_screen(
        mut commands: Commands,
        time: Res<Time>,
        mut query: Query<
            (
                Entity,
                &mut BackgroundColor,
                &mut FadeInEffect,
                &mut ReturnToMenuTimer,
            ),
            With<DeathScreen>,
        >,
        mut text_query: Query<&mut Text, With<DeathText>>,
        mut game_state: ResMut<NextState<GameState>>,
        mut app_exit_events: EventWriter<AppExit>,
    ) {
        for (entity, mut bg_color, mut fade, mut return_timer) in query.iter_mut() {
            // Update fade effect
            fade.timer.tick(time.delta());
            let alpha = fade.timer.fraction();
            bg_color.0 = Color::rgba(0.0, 0.0, 0.0, alpha * 0.7);

            // Update text color
            if let Ok(mut text) = text_query.get_single_mut() {
                text.sections[0].style.color = Color::rgba(0.8, 0.0, 0.0, alpha);
            }

            // Update return timer
            return_timer.timer.tick(time.delta());
            if return_timer.timer.finished() {
                app_exit_events.send(AppExit::Success);
            }
        }
    }
    // Modify TurnState to include pending air cards
    #[derive(Resource)]
    struct TurnState {
        first_card_played: bool,
        cards_played_this_turn: Vec<CardType>,
        crystal_power: i32,
        turn_count: i32,
        pending_air_cards: i32,
    }

    impl Default for TurnState {
        fn default() -> Self {
            Self {
                first_card_played: true,
                cards_played_this_turn: Vec::new(),
                crystal_power: 0,
                turn_count: 0,
                pending_air_cards: 0, // Initialize pending_air_cards to 0
            }
        }
    }

    // Constants for base damage values
    const FIRE_BASE_DAMAGE: f32 = 8.0;
    const FIRE_FIRST_CARD_BONUS: f32 = 7.0;
    const ICE_BASE_DAMAGE: f32 = 6.0;
    const CRYSTAL_BASE_DAMAGE: f32 = 4.0;
    const AIR_BASE_DAMAGE: f32 = 2.0;
    const EARTH_BASE_DAMAGE: f32 = 5.0;

    fn update_health_bars(
        query: Query<(&Health, &Children), Without<HealthBar>>,
        mut health_bar_query: Query<&mut Sprite, With<HealthBar>>,
    ) {
        for (health, children) in query.iter() {
            for child in children.iter() {
                if let Ok(mut bar_sprite) = health_bar_query.get_mut(*child) {
                    // Update health bar width based on current health
                    let bar_width = 100.0;
                    let health_percentage = health.current / health.maximum;

                    bar_sprite.custom_size = Some(Vec2::new(
                        bar_width * health_percentage,
                        bar_sprite.custom_size.unwrap_or(Vec2::ZERO).y,
                    ));

                    // Update color based on health percentage
                    bar_sprite.color = if health_percentage > 0.5 {
                        Color::srgb(0.0, 1.0, 0.0) // Green: rgb(0, 255, 0)
                    } else if health_percentage > 0.25 {
                        Color::srgb(1.0, 0.65, 0.0) // Orange: rgb(255, 165, 0)
                    } else {
                        Color::srgb(1.0, 0.0, 0.0) // Red: rgb(255, 0, 0)
                    };
                }
            }
        }
    }
    #[derive(Resource)]
    struct FightState {
        current_turn: Turn,
        selected_card: Option<usize>,
    }

    #[derive(PartialEq)]
    enum Turn {
        Player,
        Enemy,
    }

    impl Default for FightState {
        fn default() -> Self {
            Self {
                current_turn: Turn::Player,
                selected_card: None,
            }
        }
    }

    // Update the card hover system to use FightState
    fn update_card_hover(
        mut card_query: Query<
            (
                &Interaction,
                &mut Transform,
                &OriginalPosition,
                &mut Style,
                Entity,
            ),
            (With<Card>, Changed<Interaction>),
        >,
        mut commands: Commands,
        fight_state: Res<FightState>,
    ) {
        for (interaction, mut transform, original_pos, mut style, entity) in card_query.iter_mut() {
            match *interaction {
                Interaction::Hovered => {
                    if fight_state.current_turn == Turn::Player {
                        transform.translation.y = original_pos.0.y + 30.0;
                        style.width = Val::Px(200.0);
                        style.height = Val::Px(280.0);
                    }
                }
                _ => {
                    transform.translation.y = original_pos.0.y;
                    style.width = Val::Px(180.0);
                    style.height = Val::Px(250.0);
                }
            }
        }
    }

    fn handle_card_click(
        mut commands: Commands,
        mut card_query: Query<
            (&Interaction, Entity, &CardType),
            (Changed<Interaction>, With<Card>),
        >,
        cards_in_hand: Query<Entity, With<Card>>, // Query to count cards
        mut fight_state: ResMut<FightState>,
        mut turn_state: ResMut<TurnState>,
        mut monster_query: Query<(Entity, &mut Health, &Children), With<Monster>>,
        health_container_query: Query<&Children, With<HealthBarContainer>>,
        mut health_bar_query: Query<&mut Sprite, With<HealthBar>>,
        asset_server: Res<AssetServer>,
    ) {
        if fight_state.current_turn != Turn::Player {
            return;
        }

        for (interaction, card_entity, card_type) in card_query.iter() {
            if *interaction == Interaction::Pressed {
                println!("First card played status: {}", turn_state.first_card_played);

                // Calculate damage based on whether this is the first card
                let is_first = turn_state.first_card_played;
                let cards_in_hand_count = cards_in_hand.iter().count() as f32; // Get count here

                let damage = if *card_type == CardType::Fire && is_first {
                    println!("Fire card played as first card! Enhanced damage!");
                    FIRE_BASE_DAMAGE + FIRE_FIRST_CARD_BONUS
                } else {
                    match card_type {
                        CardType::Fire => {
                            println!("Fire card played but not first");
                            FIRE_BASE_DAMAGE
                        }
                        CardType::Ice => {
                            let mut damage = ICE_BASE_DAMAGE;

                            if let Some(last_card) = turn_state.cards_played_this_turn.last() {
                                if matches!(last_card, CardType::Fire) {
                                    damage *= 2.0;
                                }
                            }

                            if turn_state
                                .cards_played_this_turn
                                .iter()
                                .any(|c| matches!(c, CardType::Earth))
                            {
                                damage = 0.0;
                            }

                            damage
                        }
                        CardType::Crystal => {
                            let effects_bonus =
                                (turn_state.cards_played_this_turn.len() as f32) * 2.0;
                            let turn_bonus = turn_state.crystal_power as f32;
                            CRYSTAL_BASE_DAMAGE + effects_bonus + turn_bonus
                        }
                        CardType::Air => AIR_BASE_DAMAGE,
                        CardType::Earth => {
                            let turn_bonus = turn_state.turn_count as f32;
                            EARTH_BASE_DAMAGE + cards_in_hand_count + turn_bonus
                            // Use the count here
                        }
                    }
                };

                // Deal damage
                for (entity, mut monster_health, children) in monster_query.iter_mut() {
                    monster_health.current = (monster_health.current - damage).max(0.0);
                    println!("Dealing {} damage. First card: {}", damage, is_first);
                    spawn_damage_text(&mut commands, damage, &asset_server);
                    // Update monster's health bar
                    for child in children.iter() {
                        if let Ok(container_children) = health_container_query.get(*child) {
                            for health_bar_entity in container_children.iter() {
                                if let Ok(mut bar_sprite) =
                                    health_bar_query.get_mut(*health_bar_entity)
                                {
                                    let bar_width = 150.0; // Match the width set in chapter1_setup
                                    let health_percentage =
                                        monster_health.current / monster_health.maximum;

                                    bar_sprite.custom_size = Some(Vec2::new(
                                        bar_width * health_percentage,
                                        bar_sprite.custom_size.unwrap_or(Vec2::ZERO).y,
                                    ));

                                    // Update color based on health percentage
                                    bar_sprite.color = if health_percentage > 0.5 {
                                        Color::srgb(0.0, 1.0, 0.0) // Green
                                    } else if health_percentage > 0.25 {
                                        Color::srgb(1.0, 0.65, 0.0) // Orange
                                    } else {
                                        Color::srgb(1.0, 0.0, 0.0) // Red
                                    };
                                }
                            }
                        }
                    }

                    // If monster dies, despawn it
                    if monster_health.current <= 0.0 {
                        commands.entity(entity).despawn_recursive();
                    }
                }

                // Handle special card effects and cleanup
                if matches!(card_type, CardType::Air) {
                    turn_state.pending_air_cards += 2;
                }

                // Update turn state BEFORE destroying the card
                turn_state.cards_played_this_turn.push(*card_type);
                turn_state.first_card_played = false;
                println!("Set first_card_played to false");

                // Destroy the played card
                commands.entity(card_entity).despawn_recursive();

                break;
            }
        }
    }

    // Add this system to help debug the turn state
    // Add this system to help debug the turn state
    fn debug_turn_state(turn_state: Res<TurnState>, fight_state: Res<FightState>) {
        println!(
            "Turn State - First card: {}, Cards played: {}",
            turn_state.first_card_played,
            turn_state.cards_played_this_turn.len(),
        );
    }

    fn process_turn(
        mut fight_state: ResMut<FightState>,
        mut query_set: ParamSet<(
            Query<(&mut Health, &Children), With<SideCharacter>>,
            Query<(&Health, &Damage), With<Monster>>,
        )>,
        health_container_query: Query<&Children, With<HealthBarContainer>>,
        mut health_bar_query: Query<&mut Sprite, With<HealthBar>>,
        asset_server: Res<AssetServer>,
        mut commands: Commands,
    ) {
        if fight_state.current_turn == Turn::Enemy {
            // First, collect all living monsters and their damage
            let monster_attacks: Vec<f32> = query_set
                .p1()
                .iter()
                .filter(|(health, _)| health.current > 0.0)
                .map(|(_, damage)| damage.0)
                .collect();

            // Then apply damage to the player
            if let Ok((mut character_health, children)) = query_set.p0().get_single_mut() {
                for damage in monster_attacks {
                    character_health.current = (character_health.current - damage).max(0.0);
                    println!(
                        "Player health: {}/{}",
                        character_health.current, character_health.maximum
                    );

                    // Health bar update logic using nested queries
                    for child in children.iter() {
                        if let Ok(container_children) = health_container_query.get(*child) {
                            for health_bar_entity in container_children.iter() {
                                if let Ok(mut bar_sprite) =
                                    health_bar_query.get_mut(*health_bar_entity)
                                {
                                    let bar_width = 150.0;
                                    let health_percentage =
                                        character_health.current / character_health.maximum;

                                    bar_sprite.custom_size = Some(Vec2::new(
                                        bar_width * health_percentage,
                                        bar_sprite.custom_size.unwrap_or(Vec2::ZERO).y,
                                    ));

                                    bar_sprite.color = if health_percentage > 0.5 {
                                        Color::srgb(0.0, 1.0, 0.0)
                                    } else if health_percentage > 0.25 {
                                        Color::srgb(1.0, 0.65, 0.0)
                                    } else {
                                        Color::srgb(1.0, 0.0, 0.0)
                                    };
                                }
                            }
                        }
                    }

                    spawn_damage_text(&mut commands, damage, &asset_server);

                    // Check for player death
                    if character_health.current <= 0.0 {
                        spawn_death_screen(&mut commands, &asset_server);
                    }
                }

                // Switch back to player turn
                fight_state.current_turn = Turn::Player;
            }
        }
    }

    // Add a component for the damage text effect
    #[derive(Component)]
    struct DamageText {
        timer: Timer,
    }

    // The spawn_damage_text and animate_damage_text functions remain the same
    fn spawn_damage_text(commands: &mut Commands, damage: f32, asset_server: &Res<AssetServer>) {
        let mut color = Color::srgb(1.0, 0.0, 0.0);

        commands.spawn((
            Text2dBundle {
                text: Text::from_section(
                    format!("-{}", damage),
                    TextStyle {
                        font_size: 30.0,
                        color,
                        ..default()
                    },
                ),
                transform: Transform::from_xyz(0.0, 0.0, 10.0),
                ..default()
            },
            DamageText {
                timer: Timer::from_seconds(1.0, TimerMode::Once),
            },
        ));
    }

    fn animate_damage_text(
        mut commands: Commands,
        time: Res<Time>,
        mut query: Query<(Entity, &mut Transform, &mut Text, &mut DamageText)>,
    ) {
        for (entity, mut transform, mut text, mut damage_text) in query.iter_mut() {
            damage_text.timer.tick(time.delta());

            // Move the text upward
            transform.translation.y += 100.0 * time.delta_seconds();

            // Fade out the text
            let alpha =
                1.0 - damage_text.timer.elapsed_secs() / damage_text.timer.duration().as_secs_f32();

            // Remove the text when the timer is finished
            if damage_text.timer.finished() {
                commands.entity(entity).despawn();
            }
        }
    }
    fn handle_end_turn_button(
        mut interaction_query: Query<
            (&Interaction, &mut BackgroundColor),
            (Changed<Interaction>, With<EndTurnButton>),
        >,
        mut fight_state: ResMut<FightState>,
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        turn_state: Res<TurnState>,
    ) {
        for (interaction, mut color) in &mut interaction_query {
            match *interaction {
                Interaction::Pressed => {
                    if fight_state.current_turn == Turn::Player {
                        // Add air cards before changing turn
                        for _ in 0..turn_state.pending_air_cards {
                            spawn_card(&mut commands, CardType::Air, &asset_server);
                        }

                        fight_state.current_turn = Turn::Enemy;
                        *color = Color::srgb(0.35, 0.35, 0.35).into();
                    }
                }
                Interaction::Hovered => {
                    if fight_state.current_turn == Turn::Player {
                        *color = Color::srgb(0.25, 0.25, 0.25).into();
                    }
                }
                Interaction::None => {
                    *color = Color::srgb(0.15, 0.15, 0.15).into();
                }
            }
        }
    }

    // Add this system to update the button's appearance based on turn state
    fn update_end_turn_button(
        fight_state: Res<FightState>,
        mut button_query: Query<&mut BackgroundColor, With<EndTurnButton>>,
        mut text_query: Query<&mut Text, With<ButtonText>>,
    ) {
        if let Ok(mut color) = button_query.get_single_mut() {
            if fight_state.current_turn == Turn::Player {
                *color = Color::srgb(0.15, 0.15, 0.15).into();
            } else {
                *color = Color::srgb(0.5, 0.5, 0.5).into();
            }
        }

        if let Ok(mut text) = text_query.get_single_mut() {
            if fight_state.current_turn == Turn::Player {
                text.sections[0].value = "End Turn".to_string();
            } else {
                text.sections[0].value = "Enemy Turn".to_string();
            }
        }
    }
    // Update the chapter1_plugin to include debug system
    pub fn chapter3_plugin(app: &mut App) {
        app.init_resource::<FightState>()
            .init_resource::<TurnState>() // This line was already correct
            .add_systems(OnEnter(GameState::Chapter3), (chapter1_setup,))
            .add_systems(
                Update,
                (
                    animate_sprite,
                    update_card_hover,
                    handle_card_click,
                    process_turn,
                    update_health_bars,
                    handle_end_turn_button,
                    update_end_turn_button,
                    animate_damage_text,
                    update_death_screen,
                    process_pending_cards,
                    update_turn_state,
                    check_victory_condition, // Add this
                    update_victory_screen,
                    //debug_turn_state,
                )
                    .chain()
                    .run_if(in_state(GameState::Chapter3)),
            )
            .add_systems(
                OnExit(GameState::Chapter3),
                super::despawn_screen::<OnChapterOneScreen>,
            );
    }

    #[derive(Component)]
    struct PendingCards {
        card_type: CardType,
        amount: i32,
    }

    fn process_pending_cards(
        mut commands: Commands,
        pending_query: Query<(Entity, &PendingCards)>,
        mut turn_state: ResMut<TurnState>,
        asset_server: Res<AssetServer>,
    ) {
        for (entity, pending) in pending_query.iter() {
            for _ in 0..pending.amount {
                spawn_card(&mut commands, pending.card_type, &asset_server);
            }
            commands.entity(entity).despawn();
        }
    }

    fn spawn_card(commands: &mut Commands, card_type: CardType, asset_server: &Res<AssetServer>) {
        let texture = match card_type {
            CardType::Fire => asset_server.load("textures/Game Icons/Fire.png"),
            CardType::Ice => asset_server.load("textures/Game Icons/Frost.png"),
            CardType::Air => asset_server.load("textures/Game Icons/air.png"),
            CardType::Earth => asset_server.load("textures/Game Icons/Earth.png"),
            CardType::Crystal => asset_server.load("textures/Game Icons/Crystal.png"),
        };

        commands.spawn((
            ImageBundle {
                style: Style {
                    width: Val::Px(180.0),
                    height: Val::Px(250.0),
                    margin: UiRect::horizontal(Val::Px(10.0)),
                    ..default()
                },
                image: UiImage::new(texture),
                background_color: Color::WHITE.into(),
                ..default()
            },
            Interaction::None,
            Card,
            card_type,
            OriginalPosition(Vec2::new(0.0, 0.0)), // Position will need to be adjusted
            OnChapterOneScreen,
        ));
    }

    fn update_turn_state(mut fight_state: ResMut<FightState>, mut turn_state: ResMut<TurnState>) {
        // if fight_state.current_turn == Turn::Player {
        //     turn_state.cards_played_this_turn.clear();
        //     turn_state.crystal_power += 1;
        //     turn_state.turn_count += 1;
        //     turn_state.pending_air_cards = 0; // Reset pending air cards
        // }
    }

    fn chapter1_setup(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
        windows: Query<&Window>,
    ) {
        commands.insert_resource(TurnState {
            first_card_played: true,
            cards_played_this_turn: Vec::new(),
            crystal_power: 0,
            turn_count: 0,
            pending_air_cards: 0,
        });
        let window = windows.single();

        // Calculate positions
        let char_x = window.width() * -0.25;
        let char_y = window.height() * -0.75;

        // Load textures
        let texture_handle: Handle<Image> = asset_server.load("textures/intro_game_sprite.png");
        let fire_card_texture: Handle<Image> = asset_server.load("textures/Game Icons/Fire.png");
        let ice_card_texture: Handle<Image> = asset_server.load("textures/Game Icons/Frost.png");
        let air_card_texture: Handle<Image> = asset_server.load("textures/Game Icons/air.png");
        let earth_card_texture: Handle<Image> = asset_server.load("textures/Game Icons/Earth.png");
        let crystal_card_texture: Handle<Image> =
            asset_server.load("textures/Game Icons/Crystal.png");
        let forest: Handle<Image> = asset_server.load("textures/waterfall.png");

        let side_character_texture = asset_server.load("textures/character.png");
        let monster_texture: Handle<Image> = asset_server.load("textures/angle.png");

        let layout = TextureAtlasLayout::from_grid(UVec2::new(576, 324), 5, 64, None, None);
        let atlas_layout = atlas_layouts.add(layout);

        // Spawn main container
        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                    ..default()
                },
                OnChapterOneScreen,
            ))
            // .with_children(|parent| {
            //     // Background animation (same as before)
            //     parent
            //         .spawn(NodeBundle {
            //             style: Style {
            //                 width: Val::Vw(100.0),
            //                 height: Val::Vh(100.0),
            //                 align_items: AlignItems::Center,
            //                 justify_content: JustifyContent::Center,
            //                 ..default()
            //             },
            //             ..default()
            //         })
            //         .with_children(|parent| {
            //             parent.spawn((
            //                 SpriteSheetBundle {
            //                     texture: texture_handle,
            //                     atlas: TextureAtlas {
            //                         layout: atlas_layout,
            //                         index: 0,
            //                     },
            //                     transform: Transform::from_xyz(
            //                         -window.width() / 2.0,
            //                         -window.height() / 2.0,
            //                         1.0,
            //                     ),
            //                     sprite: Sprite {
            //                         custom_size: Some(Vec2::new(1920.0, 1080.0)),
            //                         anchor: bevy::sprite::Anchor::Center,
            //                         ..default()
            //                     },
            //                     ..default()
            //                 },
            //                 AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
            //                 AnimationIndices {
            //                     first: 0,
            //                     last: 320,
            //                 },
            //             ));
            //         });
            .with_children(|parent| {
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            width: Val::Vw(100.0),
                            height: Val::Vh(100.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|parent| {
                        parent.spawn(SpriteBundle {
                            texture: forest,
                            transform: Transform::from_xyz(
                                -window.width() / 2.0,
                                -window.height() / 2.0,
                                1.0,
                            ),
                            sprite: Sprite {
                                custom_size: Some(Vec2::new(1920.0, 1080.0)),
                                anchor: bevy::sprite::Anchor::Center,
                                ..default()
                            },
                            ..default()
                        });
                    });

                // Side character with health
                parent
                    .spawn((
                        SpriteBundle {
                            texture: side_character_texture,
                            transform: Transform::from_xyz(
                                -window.width() / 2.0 + char_x,
                                char_y,
                                2.0,
                            ),
                            sprite: Sprite {
                                custom_size: Some(Vec2::new(400.0, 400.0)),
                                anchor: bevy::sprite::Anchor::Center,
                                ..default()
                            },
                            ..default()
                        },
                        SideCharacter,
                        Health {
                            current: 100.0,
                            maximum: 100.0,
                        },
                    ))
                    .with_children(|monster| {
                        // Health bar background
                        monster
                            .spawn((
                                SpriteBundle {
                                    sprite: Sprite {
                                        color: Color::srgb(0.2, 0.2, 0.2),
                                        custom_size: Some(Vec2::new(150.0, 10.0)),
                                        ..default()
                                    },
                                    transform: Transform::from_xyz(
                                        0.0,    // Centered horizontally relative to parent
                                        -175.0, // Below sprite with 20px padding
                                        0.1,
                                    ),
                                    ..default()
                                },
                                HealthBarContainer,
                                OnChapterOneScreen,
                            ))
                            .with_children(|container| {
                                // Actual health bar
                                container.spawn((
                                    SpriteBundle {
                                        sprite: Sprite {
                                            color: Color::srgb(0.0, 1.0, 0.0),
                                            custom_size: Some(Vec2::new(150.0, 10.0)),
                                            anchor: bevy::sprite::Anchor::CenterLeft,
                                            ..default()
                                        },
                                        transform: Transform::from_xyz(-75.0, 0.0, 0.2),
                                        ..default()
                                    },
                                    HealthBar,
                                ));
                            });
                    });
                let monster1_damage = 50.0;
                let monster2_damage = 10.0;
                // Monster 1 with health
                parent
                    .spawn((
                        SpriteBundle {
                            texture: monster_texture,
                            transform: Transform::from_xyz(
                                char_x,
                                char_y + window.height() / 16.0,
                                2.0,
                            ),
                            sprite: Sprite {
                                custom_size: Some(Vec2::new(400.0, 400.0)),
                                anchor: bevy::sprite::Anchor::Center,
                                ..default()
                            },
                            ..default()
                        },
                        Monster,
                        Health {
                            current: 44.0,
                            maximum: 44.0,
                        },
                        Damage(monster1_damage), // This monster deals 15 damage
                    ))
                    .with_children(|monster| {
                        // Spawn the black background sprite
                        monster.spawn(SpriteBundle {
                            sprite: Sprite {
                                color: Color::BLACK,
                                custom_size: Some(Vec2::new(50.0, 30.0)), // Adjust size as needed
                                ..default()
                            },
                            transform: Transform::from_xyz(0.0, 180.0, 0.0),
                            ..default()
                        });
                        // Spawn damage text above monster
                        monster.spawn((
                            Text2dBundle {
                                text: Text::from_section(
                                    format!("{}", monster1_damage),
                                    TextStyle {
                                        font_size: 24.0,
                                        color: Color::rgb(1.0, 0.0, 0.0),
                                        ..default()
                                    },
                                ),
                                transform: Transform::from_xyz(0.0, 180.0, 0.1), // Position above monster
                                ..default()
                            },
                            DamageDisplay,
                        ));
                        // Health bar background
                        monster
                            .spawn((
                                SpriteBundle {
                                    sprite: Sprite {
                                        color: Color::srgb(0.2, 0.2, 0.2),
                                        custom_size: Some(Vec2::new(150.0, 10.0)),
                                        ..default()
                                    },
                                    transform: Transform::from_xyz(
                                        0.0,    // Centered horizontally relative to parent
                                        -215.0, // Below sprite with 20px padding
                                        0.1,
                                    ),
                                    ..default()
                                },
                                HealthBarContainer,
                                OnChapterOneScreen,
                            ))
                            .with_children(|container| {
                                // Actual health bar
                                container.spawn((
                                    SpriteBundle {
                                        sprite: Sprite {
                                            color: Color::srgb(0.0, 1.0, 0.0),
                                            custom_size: Some(Vec2::new(150.0, 10.0)),
                                            anchor: bevy::sprite::Anchor::CenterLeft,
                                            ..default()
                                        },
                                        transform: Transform::from_xyz(-75.0, 0.0, 0.2),
                                        ..default()
                                    },
                                    HealthBar,
                                ));
                            });
                    });

                // Add this to the chapter1_setup function after spawning the cards container
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            right: Val::Px(20.0),
                            top: Val::Px(20.0), // Changed from top to bottom
                            width: Val::Px(150.0),
                            height: Val::Px(50.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        background_color: Color::srgb(0.15, 0.15, 0.15).into(),
                        ..default()
                    })
                    .with_children(|parent| {
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: Style {
                                        width: Val::Percent(100.0),
                                        height: Val::Percent(100.0),
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                    background_color: Color::srgb(0.15, 0.15, 0.15).into(),
                                    ..default()
                                },
                                EndTurnButton,
                            ))
                            .with_children(|parent| {
                                parent.spawn((
                                    TextBundle::from_section(
                                        "End Turn",
                                        TextStyle {
                                            font_size: 20.0,
                                            color: Color::WHITE,
                                            ..default()
                                        },
                                    ),
                                    ButtonText,
                                ));
                            });
                    });
                // Cards container
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Px(200.0),
                            position_type: PositionType::Absolute,
                            top: Val::Px(20.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(20.0),
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|parent| {
                        // Define card types and their corresponding textures
                        let cards = vec![
                            (CardType::Earth, earth_card_texture.clone()),
                            (CardType::Crystal, crystal_card_texture.clone()),
                            (CardType::Fire, fire_card_texture.clone()),
                            (CardType::Ice, ice_card_texture.clone()),
                            //(CardType::Air, air_card_texture.clone()),
                        ];

                        // Spawn three cards
                        for (i, (card_type, card_texture)) in cards.into_iter().enumerate() {
                            // Changed to into_iter()
                            let x_position = (i as f32 - 1.0) * 220.0;

                            parent.spawn((
                                ImageBundle {
                                    style: Style {
                                        width: Val::Px(180.0),
                                        height: Val::Px(250.0),
                                        margin: UiRect::horizontal(Val::Px(10.0)),
                                        ..default()
                                    },
                                    image: UiImage::new(card_texture),
                                    background_color: Color::WHITE.into(),
                                    transform: Transform::from_xyz(x_position, 0.0, 0.0),
                                    ..default()
                                },
                                Interaction::None,
                                Card,
                                card_type, // No longer a reference
                                OriginalPosition(Vec2::new(x_position, 0.0)),
                            ));
                        }
                    });
            });
    }

    fn animate_sprite(
        time: Res<Time>,
        mut query: Query<(&mut TextureAtlas, &mut AnimationTimer, &AnimationIndices)>,
    ) {
        for (mut atlas, mut timer, indices) in &mut query {
            timer.0.tick(time.delta());
            if timer.0.just_finished() {
                atlas.index = if atlas.index == indices.last {
                    indices.first
                } else {
                    atlas.index + 1
                };
            }
        }
    }

    // Add these new components and structs in the chapter1 mod
    #[derive(Component)]
    struct VictoryScreen;

    #[derive(Component)]
    struct VictoryText;

    #[derive(Component)]
    struct ReturnToMenuVictoryTimer {
        timer: Timer,
    }

    fn spawn_victory_screen(commands: &mut Commands, asset_server: &AssetServer) {
        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        position_type: PositionType::Absolute,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.0)),
                    ..default()
                },
                VictoryScreen,
                FadeInEffect {
                    timer: Timer::from_seconds(2.0, TimerMode::Once),
                },
                ReturnToMenuVictoryTimer {
                    timer: Timer::from_seconds(5.0, TimerMode::Once),
                },
            ))
            .with_children(|parent| {
                parent.spawn((
                    TextBundle::from_section(
                        "VICTORY!",
                        TextStyle {
                            font_size: 120.0,
                            color: Color::rgba(0.0, 0.8, 0.0, 0.0), // Start transparent, but green
                            ..default()
                        },
                    ),
                    VictoryText,
                    FadeInEffect {
                        timer: Timer::from_seconds(2.0, TimerMode::Once),
                    },
                ));
            });
    }

    fn update_victory_screen(
        mut commands: Commands,
        time: Res<Time>,
        mut query: Query<
            (
                Entity,
                &mut BackgroundColor,
                &mut FadeInEffect,
                &mut ReturnToMenuVictoryTimer,
            ),
            With<VictoryScreen>,
        >,
        mut text_query: Query<&mut Text, With<VictoryText>>,
        mut game_state: ResMut<NextState<GameState>>,
        mut app_exit_events: EventWriter<AppExit>,
    ) {
        for (entity, mut bg_color, mut fade, mut return_timer) in query.iter_mut() {
            fade.timer.tick(time.delta());
            let alpha = fade.timer.fraction();
            bg_color.0 = Color::rgba(0.0, 0.0, 0.0, alpha * 0.7);

            if let Ok(mut text) = text_query.get_single_mut() {
                text.sections[0].style.color = Color::rgba(0.0, 0.8, 0.0, alpha);
            }

            return_timer.timer.tick(time.delta());
            if return_timer.timer.finished() {
                commands.entity(entity).despawn_recursive(); // Clean up victory screen
                game_state.set(GameState::Game4);
                // commands.entity(entity).despawn_recursive(); // Clean up victory screen
                //app_exit_events.send(AppExit::Success);
            }
        }
    }

    fn check_victory_condition(
        monster_query: Query<&Health, With<Monster>>,
        victory_screen_query: Query<(), With<VictoryScreen>>,
        mut commands: Commands,
        asset_server: Res<AssetServer>,
    ) {
        if victory_screen_query.is_empty() {
            // Only check if victory screen isn't already shown
            let all_monsters_dead = monster_query.iter().all(|health| health.current <= 0.0);

            if all_monsters_dead {
                spawn_victory_screen(&mut commands, &asset_server);
            }
        }
    }
}

mod chapter4 {
    use super::GameState;
    use bevy::app::AppExit;
    use bevy::ecs::system::ParamSet;
    use bevy::prelude::*;

    #[derive(Component, Copy, Clone, Debug, PartialEq)]
    enum CardType {
        Fire,
        Ice,
        Air,
        Earth,
        Crystal,
        Heal,
        // Add other types as needed
    }
    // Components
    #[derive(Component)]
    struct AnimationTimer(Timer);

    #[derive(Component)]
    struct AnimationIndices {
        first: usize,
        last: usize,
    }

    #[derive(Component)]
    struct Card;

    #[derive(Component)]
    struct OriginalPosition(Vec2);

    #[derive(Component)]
    struct OnChapterOneScreen;

    #[derive(Component)]
    struct SideCharacter;

    #[derive(Component)]
    struct Monster;

    #[derive(Component)]
    struct Health {
        current: f32,
        maximum: f32,
    }

    // Add this to your existing components if not already present
    #[derive(Component)]
    struct HealthBarContainer;

    #[derive(Component)]
    struct HealthBar;

    // Add these new components in the chapter1 mod
    #[derive(Component)]
    struct EndTurnButton;

    #[derive(Component)]
    struct ButtonText;

    #[derive(Component)]
    struct Damage(f32);

    #[derive(Component)]
    struct DeathScreen;

    #[derive(Component)]
    struct DeathText;

    #[derive(Component)]
    struct FadeInEffect {
        timer: Timer,
    }

    #[derive(Component)]
    struct ReturnToMenuTimer {
        timer: Timer,
    }

    #[derive(Component)]
    struct DamageDisplay;

    #[derive(Resource, Default)]
    struct PendingAirCards(i32);

    fn spawn_death_screen(commands: &mut Commands, asset_server: &AssetServer) {
        // Main container
        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        position_type: PositionType::Absolute,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.0)),
                    ..default()
                },
                DeathScreen,
                FadeInEffect {
                    timer: Timer::from_seconds(2.0, TimerMode::Once),
                },
                // Add a timer to return to menu after 5 seconds
                ReturnToMenuTimer {
                    timer: Timer::from_seconds(5.0, TimerMode::Once),
                },
            ))
            .with_children(|parent| {
                // "YOU DIED" text
                parent.spawn((
                    TextBundle::from_section(
                        "YOU DIED",
                        TextStyle {
                            font_size: 120.0,
                            color: Color::rgba(0.8, 0.0, 0.0, 0.0), // Start transparent
                            ..default()
                        },
                    ),
                    DeathText,
                    FadeInEffect {
                        timer: Timer::from_seconds(2.0, TimerMode::Once),
                    },
                ));
            });
    }

    // Update the death screen system to handle both fade-in and menu transition
    fn update_death_screen(
        mut commands: Commands,
        time: Res<Time>,
        mut query: Query<
            (
                Entity,
                &mut BackgroundColor,
                &mut FadeInEffect,
                &mut ReturnToMenuTimer,
            ),
            With<DeathScreen>,
        >,
        mut text_query: Query<&mut Text, With<DeathText>>,
        mut game_state: ResMut<NextState<GameState>>,
        mut app_exit_events: EventWriter<AppExit>,
    ) {
        for (entity, mut bg_color, mut fade, mut return_timer) in query.iter_mut() {
            // Update fade effect
            fade.timer.tick(time.delta());
            let alpha = fade.timer.fraction();
            bg_color.0 = Color::rgba(0.0, 0.0, 0.0, alpha * 0.7);

            // Update text color
            if let Ok(mut text) = text_query.get_single_mut() {
                text.sections[0].style.color = Color::rgba(0.8, 0.0, 0.0, alpha);
            }

            // Update return timer
            return_timer.timer.tick(time.delta());
            if return_timer.timer.finished() {
                app_exit_events.send(AppExit::Success);
            }
        }
    }
    // Modify TurnState to include pending air cards
    #[derive(Resource)]
    struct TurnState {
        first_card_played: bool,
        cards_played_this_turn: Vec<CardType>,
        crystal_power: i32,
        turn_count: i32,
        pending_air_cards: i32,
    }

    impl Default for TurnState {
        fn default() -> Self {
            Self {
                first_card_played: true,
                cards_played_this_turn: Vec::new(),
                crystal_power: 0,
                turn_count: 0,
                pending_air_cards: 0, // Initialize pending_air_cards to 0
            }
        }
    }

    // Constants for base damage values
    const FIRE_BASE_DAMAGE: f32 = 8.0;
    const FIRE_FIRST_CARD_BONUS: f32 = 7.0;
    const ICE_BASE_DAMAGE: f32 = 6.0;
    const CRYSTAL_BASE_DAMAGE: f32 = 4.0;
    const AIR_BASE_DAMAGE: f32 = 2.0;
    const EARTH_BASE_DAMAGE: f32 = 5.0;
    const HEAL_BASE_DAMAGE: f32 = 8.0;

    fn update_health_bars(
        query: Query<(&Health, &Children), Without<HealthBar>>,
        mut health_bar_query: Query<&mut Sprite, With<HealthBar>>,
    ) {
        for (health, children) in query.iter() {
            for child in children.iter() {
                if let Ok(mut bar_sprite) = health_bar_query.get_mut(*child) {
                    // Update health bar width based on current health
                    let bar_width = 100.0;
                    let health_percentage = health.current / health.maximum;

                    bar_sprite.custom_size = Some(Vec2::new(
                        bar_width * health_percentage,
                        bar_sprite.custom_size.unwrap_or(Vec2::ZERO).y,
                    ));

                    // Update color based on health percentage
                    bar_sprite.color = if health_percentage > 0.5 {
                        Color::srgb(0.0, 1.0, 0.0) // Green: rgb(0, 255, 0)
                    } else if health_percentage > 0.25 {
                        Color::srgb(1.0, 0.65, 0.0) // Orange: rgb(255, 165, 0)
                    } else {
                        Color::srgb(1.0, 0.0, 0.0) // Red: rgb(255, 0, 0)
                    };
                }
            }
        }
    }
    #[derive(Resource)]
    struct FightState {
        current_turn: Turn,
        selected_card: Option<usize>,
    }

    #[derive(PartialEq)]
    enum Turn {
        Player,
        Enemy,
    }

    impl Default for FightState {
        fn default() -> Self {
            Self {
                current_turn: Turn::Player,
                selected_card: None,
            }
        }
    }

    // Update the card hover system to use FightState
    fn update_card_hover(
        mut card_query: Query<
            (
                &Interaction,
                &mut Transform,
                &OriginalPosition,
                &mut Style,
                Entity,
            ),
            (With<Card>, Changed<Interaction>),
        >,
        mut commands: Commands,
        fight_state: Res<FightState>,
    ) {
        for (interaction, mut transform, original_pos, mut style, entity) in card_query.iter_mut() {
            match *interaction {
                Interaction::Hovered => {
                    if fight_state.current_turn == Turn::Player {
                        transform.translation.y = original_pos.0.y + 30.0;
                        style.width = Val::Px(200.0);
                        style.height = Val::Px(280.0);
                    }
                }
                _ => {
                    transform.translation.y = original_pos.0.y;
                    style.width = Val::Px(180.0);
                    style.height = Val::Px(250.0);
                }
            }
        }
    }

    fn handle_card_click(
        mut commands: Commands,
        mut card_query: Query<
            (&Interaction, Entity, &CardType),
            (Changed<Interaction>, With<Card>),
        >,
        cards_in_hand: Query<Entity, With<Card>>, // Query to count cards
        mut fight_state: ResMut<FightState>,
        mut turn_state: ResMut<TurnState>,
        mut monster_query: Query<(Entity, &mut Health, &Children), With<Monster>>,
        health_container_query: Query<&Children, With<HealthBarContainer>>,
        mut health_bar_query: Query<&mut Sprite, With<HealthBar>>,
        asset_server: Res<AssetServer>,
    ) {
        if fight_state.current_turn != Turn::Player {
            return;
        }

        for (interaction, card_entity, card_type) in card_query.iter() {
            if *interaction == Interaction::Pressed {
                println!("First card played status: {}", turn_state.first_card_played);

                // Calculate damage based on whether this is the first card
                let is_first = turn_state.first_card_played;
                let cards_in_hand_count = cards_in_hand.iter().count() as f32; // Get count here

                let damage = if *card_type == CardType::Fire && is_first {
                    println!("Fire card played as first card! Enhanced damage!");
                    FIRE_BASE_DAMAGE + FIRE_FIRST_CARD_BONUS
                } else {
                    match card_type {
                        CardType::Fire => {
                            println!("Fire card played but not first");
                            FIRE_BASE_DAMAGE
                        }
                        CardType::Ice => {
                            let mut damage = ICE_BASE_DAMAGE;

                            if let Some(last_card) = turn_state.cards_played_this_turn.last() {
                                if matches!(last_card, CardType::Fire) {
                                    damage *= 2.0;
                                }
                            }

                            if turn_state
                                .cards_played_this_turn
                                .iter()
                                .any(|c| matches!(c, CardType::Earth))
                            {
                                damage = 0.0;
                            }

                            damage
                        }
                        CardType::Crystal => {
                            let effects_bonus =
                                (turn_state.cards_played_this_turn.len() as f32) * 2.0;
                            let turn_bonus = turn_state.crystal_power as f32;
                            CRYSTAL_BASE_DAMAGE + effects_bonus + turn_bonus
                        }
                        CardType::Air => AIR_BASE_DAMAGE,
                        CardType::Heal => {
                            // Check if any monster is at full health
                            let mut is_any_monster_full_hp = false;
                            for (_, monster_health, _) in monster_query.iter() {
                                if (monster_health.current - monster_health.maximum).abs()
                                    < f32::EPSILON
                                {
                                    is_any_monster_full_hp = true;
                                    break;
                                }
                            }

                            if is_any_monster_full_hp {
                                HEAL_BASE_DAMAGE // Heal will deal -8 damage (healing) if enemy is at full HP
                            } else {
                                -HEAL_BASE_DAMAGE // Otherwise deal 8 damage
                            }
                        }
                        CardType::Earth => {
                            let turn_bonus = turn_state.turn_count as f32;
                            EARTH_BASE_DAMAGE + cards_in_hand_count + turn_bonus
                            // Use the count here
                        }
                    }
                };

                // Deal damage
                for (entity, mut monster_health, children) in monster_query.iter_mut() {
                    monster_health.current = (monster_health.current - damage).max(0.0);
                    println!("Dealing {} damage. First card: {}", damage, is_first);
                    spawn_damage_text(&mut commands, damage, &asset_server);
                    // Update monster's health bar
                    for child in children.iter() {
                        if let Ok(container_children) = health_container_query.get(*child) {
                            for health_bar_entity in container_children.iter() {
                                if let Ok(mut bar_sprite) =
                                    health_bar_query.get_mut(*health_bar_entity)
                                {
                                    let bar_width = 150.0; // Match the width set in chapter1_setup
                                    let health_percentage =
                                        monster_health.current / monster_health.maximum;

                                    bar_sprite.custom_size = Some(Vec2::new(
                                        bar_width * health_percentage,
                                        bar_sprite.custom_size.unwrap_or(Vec2::ZERO).y,
                                    ));

                                    // Update color based on health percentage
                                    bar_sprite.color = if health_percentage > 0.5 {
                                        Color::srgb(0.0, 1.0, 0.0) // Green
                                    } else if health_percentage > 0.25 {
                                        Color::srgb(1.0, 0.65, 0.0) // Orange
                                    } else {
                                        Color::srgb(1.0, 0.0, 0.0) // Red
                                    };
                                }
                            }
                        }
                    }

                    // If monster dies, despawn it
                    if monster_health.current <= 0.0 {
                        commands.entity(entity).despawn_recursive();
                    }
                }

                // Handle special card effects and cleanup
                if matches!(card_type, CardType::Air) {
                    turn_state.pending_air_cards += 2;
                }

                // Update turn state BEFORE destroying the card
                turn_state.cards_played_this_turn.push(*card_type);
                turn_state.first_card_played = false;
                println!("Set first_card_played to false");

                // Destroy the played card
                commands.entity(card_entity).despawn_recursive();

                break;
            }
        }
    }

    // Add this system to help debug the turn state
    // Add this system to help debug the turn state
    fn debug_turn_state(turn_state: Res<TurnState>, fight_state: Res<FightState>) {
        println!(
            "Turn State - First card: {}, Cards played: {}",
            turn_state.first_card_played,
            turn_state.cards_played_this_turn.len(),
        );
    }

    fn process_turn(
        mut fight_state: ResMut<FightState>,
        mut query_set: ParamSet<(
            Query<(&mut Health, &Children), With<SideCharacter>>,
            Query<(&Health, &Damage), With<Monster>>,
        )>,
        health_container_query: Query<&Children, With<HealthBarContainer>>,
        mut health_bar_query: Query<&mut Sprite, With<HealthBar>>,
        asset_server: Res<AssetServer>,
        mut commands: Commands,
    ) {
        if fight_state.current_turn == Turn::Enemy {
            // First, collect all living monsters and their damage
            let monster_attacks: Vec<f32> = query_set
                .p1()
                .iter()
                .filter(|(health, _)| health.current > 0.0)
                .map(|(_, damage)| damage.0)
                .collect();

            // Then apply damage to the player
            if let Ok((mut character_health, children)) = query_set.p0().get_single_mut() {
                for damage in monster_attacks {
                    character_health.current = (character_health.current - damage).max(0.0);
                    println!(
                        "Player health: {}/{}",
                        character_health.current, character_health.maximum
                    );

                    // Health bar update logic using nested queries
                    for child in children.iter() {
                        if let Ok(container_children) = health_container_query.get(*child) {
                            for health_bar_entity in container_children.iter() {
                                if let Ok(mut bar_sprite) =
                                    health_bar_query.get_mut(*health_bar_entity)
                                {
                                    let bar_width = 150.0;
                                    let health_percentage =
                                        character_health.current / character_health.maximum;

                                    bar_sprite.custom_size = Some(Vec2::new(
                                        bar_width * health_percentage,
                                        bar_sprite.custom_size.unwrap_or(Vec2::ZERO).y,
                                    ));

                                    bar_sprite.color = if health_percentage > 0.5 {
                                        Color::srgb(0.0, 1.0, 0.0)
                                    } else if health_percentage > 0.25 {
                                        Color::srgb(1.0, 0.65, 0.0)
                                    } else {
                                        Color::srgb(1.0, 0.0, 0.0)
                                    };
                                }
                            }
                        }
                    }

                    spawn_damage_text(&mut commands, damage, &asset_server);

                    // Check for player death
                    if character_health.current <= 0.0 {
                        spawn_death_screen(&mut commands, &asset_server);
                    }
                }

                // Switch back to player turn
                fight_state.current_turn = Turn::Player;
            }
        }
    }

    // Add a component for the damage text effect
    #[derive(Component)]
    struct DamageText {
        timer: Timer,
    }

    // The spawn_damage_text and animate_damage_text functions remain the same
    fn spawn_damage_text(commands: &mut Commands, damage: f32, asset_server: &Res<AssetServer>) {
        let mut color = Color::srgb(1.0, 0.0, 0.0);

        commands.spawn((
            Text2dBundle {
                text: Text::from_section(
                    format!("-{}", damage),
                    TextStyle {
                        font_size: 30.0,
                        color,
                        ..default()
                    },
                ),
                transform: Transform::from_xyz(0.0, 0.0, 10.0),
                ..default()
            },
            DamageText {
                timer: Timer::from_seconds(1.0, TimerMode::Once),
            },
        ));
    }

    fn animate_damage_text(
        mut commands: Commands,
        time: Res<Time>,
        mut query: Query<(Entity, &mut Transform, &mut Text, &mut DamageText)>,
    ) {
        for (entity, mut transform, mut text, mut damage_text) in query.iter_mut() {
            damage_text.timer.tick(time.delta());

            // Move the text upward
            transform.translation.y += 100.0 * time.delta_seconds();

            // Fade out the text
            let alpha =
                1.0 - damage_text.timer.elapsed_secs() / damage_text.timer.duration().as_secs_f32();

            // Remove the text when the timer is finished
            if damage_text.timer.finished() {
                commands.entity(entity).despawn();
            }
        }
    }
    fn handle_end_turn_button(
        mut interaction_query: Query<
            (&Interaction, &mut BackgroundColor),
            (Changed<Interaction>, With<EndTurnButton>),
        >,
        mut fight_state: ResMut<FightState>,
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        turn_state: Res<TurnState>,
    ) {
        for (interaction, mut color) in &mut interaction_query {
            match *interaction {
                Interaction::Pressed => {
                    if fight_state.current_turn == Turn::Player {
                        // Add air cards before changing turn
                        for _ in 0..turn_state.pending_air_cards {
                            spawn_card(&mut commands, CardType::Air, &asset_server);
                        }

                        fight_state.current_turn = Turn::Enemy;
                        *color = Color::srgb(0.35, 0.35, 0.35).into();
                    }
                }
                Interaction::Hovered => {
                    if fight_state.current_turn == Turn::Player {
                        *color = Color::srgb(0.25, 0.25, 0.25).into();
                    }
                }
                Interaction::None => {
                    *color = Color::srgb(0.15, 0.15, 0.15).into();
                }
            }
        }
    }

    // Add this system to update the button's appearance based on turn state
    fn update_end_turn_button(
        fight_state: Res<FightState>,
        mut button_query: Query<&mut BackgroundColor, With<EndTurnButton>>,
        mut text_query: Query<&mut Text, With<ButtonText>>,
    ) {
        if let Ok(mut color) = button_query.get_single_mut() {
            if fight_state.current_turn == Turn::Player {
                *color = Color::srgb(0.15, 0.15, 0.15).into();
            } else {
                *color = Color::srgb(0.5, 0.5, 0.5).into();
            }
        }

        if let Ok(mut text) = text_query.get_single_mut() {
            if fight_state.current_turn == Turn::Player {
                text.sections[0].value = "End Turn".to_string();
            } else {
                text.sections[0].value = "Enemy Turn".to_string();
            }
        }
    }
    // Update the chapter1_plugin to include debug system
    pub fn chapter3_plugin(app: &mut App) {
        app.init_resource::<FightState>()
            .init_resource::<TurnState>() // This line was already correct
            .add_systems(OnEnter(GameState::Chapter4), (chapter1_setup,))
            .add_systems(
                Update,
                (
                    animate_sprite,
                    update_card_hover,
                    handle_card_click,
                    process_turn,
                    update_health_bars,
                    handle_end_turn_button,
                    update_end_turn_button,
                    animate_damage_text,
                    update_death_screen,
                    process_pending_cards,
                    update_turn_state,
                    check_victory_condition, // Add this
                    update_victory_screen,
                    //debug_turn_state,
                )
                    .chain()
                    .run_if(in_state(GameState::Chapter4)),
            )
            .add_systems(
                OnExit(GameState::Chapter3),
                super::despawn_screen::<OnChapterOneScreen>,
            );
    }

    #[derive(Component)]
    struct PendingCards {
        card_type: CardType,
        amount: i32,
    }

    fn process_pending_cards(
        mut commands: Commands,
        pending_query: Query<(Entity, &PendingCards)>,
        mut turn_state: ResMut<TurnState>,
        asset_server: Res<AssetServer>,
    ) {
        for (entity, pending) in pending_query.iter() {
            for _ in 0..pending.amount {
                spawn_card(&mut commands, pending.card_type, &asset_server);
            }
            commands.entity(entity).despawn();
        }
    }

    fn spawn_card(commands: &mut Commands, card_type: CardType, asset_server: &Res<AssetServer>) {
        let texture = match card_type {
            CardType::Fire => asset_server.load("textures/Game Icons/Fire.png"),
            CardType::Ice => asset_server.load("textures/Game Icons/Frost.png"),
            CardType::Air => asset_server.load("textures/Game Icons/air.png"),
            CardType::Earth => asset_server.load("textures/Game Icons/Earth.png"),
            CardType::Crystal => asset_server.load("textures/Game Icons/Crystal.png"),
            CardType::Heal => asset_server.load("textures/Game Icons/Heal.png"),
        };

        commands.spawn((
            ImageBundle {
                style: Style {
                    width: Val::Px(180.0),
                    height: Val::Px(250.0),
                    margin: UiRect::horizontal(Val::Px(10.0)),
                    ..default()
                },
                image: UiImage::new(texture),
                background_color: Color::WHITE.into(),
                ..default()
            },
            Interaction::None,
            Card,
            card_type,
            OriginalPosition(Vec2::new(0.0, 0.0)), // Position will need to be adjusted
            OnChapterOneScreen,
        ));
    }

    fn update_turn_state(mut fight_state: ResMut<FightState>, mut turn_state: ResMut<TurnState>) {
        // if fight_state.current_turn == Turn::Player {
        //     turn_state.cards_played_this_turn.clear();
        //     turn_state.crystal_power += 1;
        //     turn_state.turn_count += 1;
        //     turn_state.pending_air_cards = 0; // Reset pending air cards
        // }
    }

    fn chapter1_setup(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
        windows: Query<&Window>,
    ) {
        commands.insert_resource(TurnState {
            first_card_played: true,
            cards_played_this_turn: Vec::new(),
            crystal_power: 0,
            turn_count: 0,
            pending_air_cards: 0,
        });
        let window = windows.single();

        // Calculate positions
        let char_x = window.width() * -0.25;
        let char_y = window.height() * -0.75;

        // Load textures
        let texture_handle: Handle<Image> = asset_server.load("textures/intro_game_sprite.png");
        let fire_card_texture: Handle<Image> = asset_server.load("textures/Game Icons/Fire.png");
        let ice_card_texture: Handle<Image> = asset_server.load("textures/Game Icons/Frost.png");
        let air_card_texture: Handle<Image> = asset_server.load("textures/Game Icons/air.png");
        let earth_card_texture: Handle<Image> = asset_server.load("textures/Game Icons/Earth.png");
        let heal_card_texture: Handle<Image> = asset_server.load("textures/Game Icons/Heal.png");
        let crystal_card_texture: Handle<Image> =
            asset_server.load("textures/Game Icons/Crystal.png");
        let forest: Handle<Image> = asset_server.load("textures/Summon.png");

        let side_character_texture = asset_server.load("textures/character.png");
        let monster_texture: Handle<Image> = asset_server.load("textures/mage.png");

        let layout = TextureAtlasLayout::from_grid(UVec2::new(576, 324), 5, 64, None, None);
        let atlas_layout = atlas_layouts.add(layout);

        // Spawn main container
        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                    ..default()
                },
                OnChapterOneScreen,
            ))
            // .with_children(|parent| {
            //     // Background animation (same as before)
            //     parent
            //         .spawn(NodeBundle {
            //             style: Style {
            //                 width: Val::Vw(100.0),
            //                 height: Val::Vh(100.0),
            //                 align_items: AlignItems::Center,
            //                 justify_content: JustifyContent::Center,
            //                 ..default()
            //             },
            //             ..default()
            //         })
            //         .with_children(|parent| {
            //             parent.spawn((
            //                 SpriteSheetBundle {
            //                     texture: texture_handle,
            //                     atlas: TextureAtlas {
            //                         layout: atlas_layout,
            //                         index: 0,
            //                     },
            //                     transform: Transform::from_xyz(
            //                         -window.width() / 2.0,
            //                         -window.height() / 2.0,
            //                         1.0,
            //                     ),
            //                     sprite: Sprite {
            //                         custom_size: Some(Vec2::new(1920.0, 1080.0)),
            //                         anchor: bevy::sprite::Anchor::Center,
            //                         ..default()
            //                     },
            //                     ..default()
            //                 },
            //                 AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
            //                 AnimationIndices {
            //                     first: 0,
            //                     last: 320,
            //                 },
            //             ));
            //         });
            .with_children(|parent| {
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            width: Val::Vw(100.0),
                            height: Val::Vh(100.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|parent| {
                        parent.spawn(SpriteBundle {
                            texture: forest,
                            transform: Transform::from_xyz(
                                -window.width() / 2.0,
                                -window.height() / 2.0,
                                1.0,
                            ),
                            sprite: Sprite {
                                custom_size: Some(Vec2::new(1920.0, 1080.0)),
                                anchor: bevy::sprite::Anchor::Center,
                                ..default()
                            },
                            ..default()
                        });
                    });

                // Side character with health
                parent
                    .spawn((
                        SpriteBundle {
                            texture: side_character_texture,
                            transform: Transform::from_xyz(
                                -window.width() / 2.0 + char_x,
                                char_y,
                                2.0,
                            ),
                            sprite: Sprite {
                                custom_size: Some(Vec2::new(400.0, 400.0)),
                                anchor: bevy::sprite::Anchor::Center,
                                ..default()
                            },
                            ..default()
                        },
                        SideCharacter,
                        Health {
                            current: 100.0,
                            maximum: 100.0,
                        },
                    ))
                    .with_children(|monster| {
                        // Health bar background
                        monster
                            .spawn((
                                SpriteBundle {
                                    sprite: Sprite {
                                        color: Color::srgb(0.2, 0.2, 0.2),
                                        custom_size: Some(Vec2::new(150.0, 10.0)),
                                        ..default()
                                    },
                                    transform: Transform::from_xyz(
                                        0.0,    // Centered horizontally relative to parent
                                        -175.0, // Below sprite with 20px padding
                                        0.1,
                                    ),
                                    ..default()
                                },
                                HealthBarContainer,
                                OnChapterOneScreen,
                            ))
                            .with_children(|container| {
                                // Actual health bar
                                container.spawn((
                                    SpriteBundle {
                                        sprite: Sprite {
                                            color: Color::srgb(0.0, 1.0, 0.0),
                                            custom_size: Some(Vec2::new(150.0, 10.0)),
                                            anchor: bevy::sprite::Anchor::CenterLeft,
                                            ..default()
                                        },
                                        transform: Transform::from_xyz(-75.0, 0.0, 0.2),
                                        ..default()
                                    },
                                    HealthBar,
                                ));
                            });
                    });
                let monster1_damage = 100.0;
                let monster2_damage = 10.0;
                // Monster 1 with health
                parent
                    .spawn((
                        SpriteBundle {
                            texture: monster_texture,
                            transform: Transform::from_xyz(
                                char_x,
                                char_y + window.height() / 16.0,
                                2.0,
                            ),
                            sprite: Sprite {
                                custom_size: Some(Vec2::new(400.0, 400.0)),
                                anchor: bevy::sprite::Anchor::Center,
                                ..default()
                            },
                            ..default()
                        },
                        Monster,
                        Health {
                            current: 44.0,
                            maximum: 44.0,
                        },
                        Damage(monster1_damage), // This monster deals 15 damage
                    ))
                    .with_children(|monster| {
                        // Spawn the black background sprite
                        monster.spawn(SpriteBundle {
                            sprite: Sprite {
                                color: Color::BLACK,
                                custom_size: Some(Vec2::new(50.0, 30.0)), // Adjust size as needed
                                ..default()
                            },
                            transform: Transform::from_xyz(0.0, 210.0, 0.0),
                            ..default()
                        });
                        // Spawn damage text above monster
                        monster.spawn((
                            Text2dBundle {
                                text: Text::from_section(
                                    format!("{}", monster1_damage),
                                    TextStyle {
                                        font_size: 24.0,
                                        color: Color::rgb(1.0, 0.0, 0.0),
                                        ..default()
                                    },
                                ),
                                transform: Transform::from_xyz(0.0, 210.0, 0.1), // Position above monster
                                ..default()
                            },
                            DamageDisplay,
                        ));
                        // Health bar background
                        monster
                            .spawn((
                                SpriteBundle {
                                    sprite: Sprite {
                                        color: Color::srgb(0.2, 0.2, 0.2),
                                        custom_size: Some(Vec2::new(150.0, 10.0)),
                                        ..default()
                                    },
                                    transform: Transform::from_xyz(
                                        0.0,    // Centered horizontally relative to parent
                                        -215.0, // Below sprite with 20px padding
                                        0.1,
                                    ),
                                    ..default()
                                },
                                HealthBarContainer,
                                OnChapterOneScreen,
                            ))
                            .with_children(|container| {
                                // Actual health bar
                                container.spawn((
                                    SpriteBundle {
                                        sprite: Sprite {
                                            color: Color::srgb(0.0, 1.0, 0.0),
                                            custom_size: Some(Vec2::new(150.0, 10.0)),
                                            anchor: bevy::sprite::Anchor::CenterLeft,
                                            ..default()
                                        },
                                        transform: Transform::from_xyz(-75.0, 0.0, 0.2),
                                        ..default()
                                    },
                                    HealthBar,
                                ));
                            });
                    });

                // Add this to the chapter1_setup function after spawning the cards container
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            right: Val::Px(20.0),
                            top: Val::Px(20.0), // Changed from top to bottom
                            width: Val::Px(150.0),
                            height: Val::Px(50.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        background_color: Color::srgb(0.15, 0.15, 0.15).into(),
                        ..default()
                    })
                    .with_children(|parent| {
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: Style {
                                        width: Val::Percent(100.0),
                                        height: Val::Percent(100.0),
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                    background_color: Color::srgb(0.15, 0.15, 0.15).into(),
                                    ..default()
                                },
                                EndTurnButton,
                            ))
                            .with_children(|parent| {
                                parent.spawn((
                                    TextBundle::from_section(
                                        "End Turn",
                                        TextStyle {
                                            font_size: 20.0,
                                            color: Color::WHITE,
                                            ..default()
                                        },
                                    ),
                                    ButtonText,
                                ));
                            });
                    });
                // Cards container
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Px(200.0),
                            position_type: PositionType::Absolute,
                            top: Val::Px(20.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(20.0),
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|parent| {
                        // Define card types and their corresponding textures
                        let cards = vec![
                            (CardType::Earth, earth_card_texture.clone()),
                            (CardType::Crystal, crystal_card_texture.clone()),
                            (CardType::Fire, fire_card_texture.clone()),
                            (CardType::Ice, ice_card_texture.clone()),
                            (CardType::Heal, heal_card_texture.clone()),
                        ];

                        // Spawn three cards
                        for (i, (card_type, card_texture)) in cards.into_iter().enumerate() {
                            // Changed to into_iter()
                            let x_position = (i as f32 - 1.0) * 220.0;

                            parent.spawn((
                                ImageBundle {
                                    style: Style {
                                        width: Val::Px(180.0),
                                        height: Val::Px(250.0),
                                        margin: UiRect::horizontal(Val::Px(10.0)),
                                        ..default()
                                    },
                                    image: UiImage::new(card_texture),
                                    background_color: Color::WHITE.into(),
                                    transform: Transform::from_xyz(x_position, 0.0, 0.0),
                                    ..default()
                                },
                                Interaction::None,
                                Card,
                                card_type, // No longer a reference
                                OriginalPosition(Vec2::new(x_position, 0.0)),
                            ));
                        }
                    });
            });
    }

    fn animate_sprite(
        time: Res<Time>,
        mut query: Query<(&mut TextureAtlas, &mut AnimationTimer, &AnimationIndices)>,
    ) {
        for (mut atlas, mut timer, indices) in &mut query {
            timer.0.tick(time.delta());
            if timer.0.just_finished() {
                atlas.index = if atlas.index == indices.last {
                    indices.first
                } else {
                    atlas.index + 1
                };
            }
        }
    }

    // Add these new components and structs in the chapter1 mod
    #[derive(Component)]
    struct VictoryScreen;

    #[derive(Component)]
    struct VictoryText;

    #[derive(Component)]
    struct ReturnToMenuVictoryTimer {
        timer: Timer,
    }

    fn spawn_victory_screen(commands: &mut Commands, asset_server: &AssetServer) {
        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        position_type: PositionType::Absolute,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.0)),
                    ..default()
                },
                VictoryScreen,
                FadeInEffect {
                    timer: Timer::from_seconds(2.0, TimerMode::Once),
                },
                ReturnToMenuVictoryTimer {
                    timer: Timer::from_seconds(5.0, TimerMode::Once),
                },
            ))
            .with_children(|parent| {
                parent.spawn((
                    TextBundle::from_section(
                        "VICTORY!",
                        TextStyle {
                            font_size: 120.0,
                            color: Color::rgba(0.0, 0.8, 0.0, 0.0), // Start transparent, but green
                            ..default()
                        },
                    ),
                    VictoryText,
                    FadeInEffect {
                        timer: Timer::from_seconds(2.0, TimerMode::Once),
                    },
                ));
            });
    }

    fn update_victory_screen(
        mut commands: Commands,
        time: Res<Time>,
        mut query: Query<
            (
                Entity,
                &mut BackgroundColor,
                &mut FadeInEffect,
                &mut ReturnToMenuVictoryTimer,
            ),
            With<VictoryScreen>,
        >,
        mut text_query: Query<&mut Text, With<VictoryText>>,
        mut game_state: ResMut<NextState<GameState>>,
        mut app_exit_events: EventWriter<AppExit>,
    ) {
        for (entity, mut bg_color, mut fade, mut return_timer) in query.iter_mut() {
            fade.timer.tick(time.delta());
            let alpha = fade.timer.fraction();
            bg_color.0 = Color::rgba(0.0, 0.0, 0.0, alpha * 0.7);

            if let Ok(mut text) = text_query.get_single_mut() {
                text.sections[0].style.color = Color::rgba(0.0, 0.8, 0.0, alpha);
            }

            return_timer.timer.tick(time.delta());
            if return_timer.timer.finished() {
                // game_state.set(GameState::Game3);
                // commands.entity(entity).despawn_recursive(); // Clean up victory screen
                app_exit_events.send(AppExit::Success);
            }
        }
    }

    fn check_victory_condition(
        monster_query: Query<&Health, With<Monster>>,
        victory_screen_query: Query<(), With<VictoryScreen>>,
        mut commands: Commands,
        asset_server: Res<AssetServer>,
    ) {
        if victory_screen_query.is_empty() {
            // Only check if victory screen isn't already shown
            let all_monsters_dead = monster_query.iter().all(|health| health.current <= 0.0);

            if all_monsters_dead {
                spawn_victory_screen(&mut commands, &asset_server);
            }
        }
    }
}
