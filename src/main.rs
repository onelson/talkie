use bevy::prelude::*;
use iyes_loopless::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GameState {
    /// Setup assets, etc.
    Loading,
    /// Wait for user to confirm (before transitioning to the next state?)
    Prompt,
    /// Present a list of choices, use the selection to drive the next state.
    Choice,
    /// Draw a passage of the dialogue text, character by character.
    Playback,
}

// This is the list of "things in the game I want to be able to do based on input"
#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum Action {
    Confirm,
    Up,
    Down,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(InputManagerPlugin::<Action>::default())
        .add_loopless_state(GameState::Loading)
        .add_startup_system(spawn_player)
        // Read the ActionState in your systems using queries!
        .add_system(input_handling)
        .add_system(debug_current_state)
        .run();
}

#[derive(Component)]
struct Player;

fn spawn_player(mut commands: Commands) {
    commands
        .spawn()
        .insert(Player)
        .insert_bundle(InputManagerBundle::<Action> {
            action_state: ActionState::default(),
            input_map: InputMap::new([
                (KeyCode::Space, Action::Confirm),
                (KeyCode::Return, Action::Confirm),
                (KeyCode::Up, Action::Up),
                (KeyCode::W, Action::Up),
                (KeyCode::Down, Action::Down),
                (KeyCode::S, Action::Down),
            ]),
        });
}

// Query for the `ActionState` component in your game logic systems!
fn input_handling(query: Query<&ActionState<Action>, With<Player>>) {
    let action_state = query.single();
    // Each action has a button-like state of its own that you can check
    if action_state.just_pressed(Action::Confirm) {
        debug!("confirm!");
    } else if action_state.pressed(Action::Confirm) {
        debug!("still confirming");
    }
    if action_state.pressed(Action::Up) {
        debug!("up");
    } else if action_state.pressed(Action::Down) {
        debug!("down");
    }
}

/// We can just access the `CurrentState`, and even use change detection!
fn debug_current_state(state: Res<CurrentState<GameState>>) {
    if state.is_changed() {
        debug!("Detected state change to {:?}!", state);
    }
}
