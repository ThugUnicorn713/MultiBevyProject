use bevy::prelude::*;
use crate::network::server::GameOutcome;

#[derive(Resource)]
pub struct GameState {
   pub remaining: f32,
   pub total_visibility: f32,
   pub game_over: Option<GameOutcome>,
   pub started: bool,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            remaining: 180.0,
            total_visibility: 0.0,
            game_over: None,
            started: false,
        }
    }
}

#[derive(Component)]
pub struct TimerText;

#[derive(Component)]
pub struct GhostSeenText;

#[derive(Component)]
pub struct GameOverText;

#[derive(Component)]
pub struct WaitingText;


pub fn setup_ui (mut commands: Commands){

    commands
        .spawn(Node{
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexStart,
            ..default()
        })
        .with_children(|parent| {

             parent.spawn((
                Text::new("Waiting for host to start..."),
                TextFont {
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 0.0)),
                Node {
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                },
                WaitingText,
            ));
        
            parent.spawn((
                Text::new("3:00"),
                TextFont {
                    font_size: 50.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::top(Val::Px(1.0)),
                    ..default()
                },
                TimerText,
                Visibility::Hidden,
            ));
            
            parent.spawn((
                Text::new("Ghost has been revealed for: 0.0s / 15s"),
                TextFont {
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.5, 0.0)),
                Node {
                    margin: UiRect::top(Val::Px(5.0)),
                    ..default()
                },
                GhostSeenText,
                Visibility::Hidden,
            ));
            
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 80.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.0, 0.0)),
                Node {
                    margin: UiRect::top(Val::Px(100.0)),
                    ..default()
                },
                GameOverText,
                Visibility::Hidden,
            ));
        });

}

pub fn update_ui(
    game_state: Res<GameState>,
    mut timer_query: Query<(&mut Text, &mut Visibility), (With<TimerText>, Without<GhostSeenText>, Without<GameOverText>, Without<WaitingText>)>,
    mut seen_query: Query<(&mut Text, &mut Visibility), (With<GhostSeenText>, Without<TimerText>, Without<GameOverText>, Without<WaitingText>)>,
    mut game_over_query: Query<(&mut Text, &mut Visibility), (With<GameOverText>, Without<TimerText>, Without<GhostSeenText>, Without<WaitingText>)>,
    mut waiting_query: Query<&mut Visibility, (With<WaitingText>, Without<TimerText>, Without<GhostSeenText>, Without<GameOverText>)>,
){

      if let Ok(mut waiting_ui) = waiting_query.single_mut() {
        *waiting_ui = if game_state.started {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }

        //timer    
     if let Ok((mut text, mut visibility)) = timer_query.single_mut() {
        if game_state.started && game_state.game_over.is_none() {
            *visibility = Visibility::Visible;
            
            let mins = (game_state.remaining as u32) / 60;
            let secs = (game_state.remaining as u32) % 60;
            **text = format!("Time: {}:{:02}", mins, secs);
        }
    }

        
    if let Ok((mut text, mut visibility)) = seen_query.single_mut() {
        if game_state.started && game_state.game_over.is_none() {
            *visibility = Visibility::Visible;
            **text = format!("Ghost has be revealed for: {:.1}s / 15s", game_state.total_visibility);
        }
    }


        //game over text
    if let Ok((mut text, mut visibility)) = game_over_query.single_mut() {
        if let Some(outcome) = game_state.game_over {
            *visibility = Visibility::Visible;
            **text = match outcome {
                GameOutcome::GhostWins => "GHOST WINS!".to_string(),
                GameOutcome::BustersWin => "GHOSTBUSTERS WIN!".to_string(),
            };
        }
    }
}