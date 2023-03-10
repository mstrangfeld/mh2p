use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};
use serde::{Deserialize, Serialize};

// X, Y, Z with X being left/right, Y being forward/backward, Z being up/down
#[derive(Resource)]
struct Target {
    position: Vec3,
    max: Vec3,
    speed: f32,
}

#[derive(Component, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct MovingHead {
    position: Vec3,
    pan: MovingHeadChannel,
    tilt: MovingHeadChannel,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct Config {
    moving_heads: Vec<MovingHead>,
    room: Vec3,
    home: Vec3,
    speed: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct MovingHeadChannel {
    channel: u8,
    value: f32,
    max_value: f32,
}

impl MovingHead {
    fn point_to(&mut self, target: Vec3) {
        let direction = target - self.position;
        let distance = direction.length();
        let direction = direction / distance;

        let pan = direction.x.atan2(direction.z).to_degrees();
        let tilt = direction.y.atan2(distance).to_degrees();

        self.pan.value = pan;
        self.tilt.value = tilt;
    }

    fn send_midi(&self) {
        // TODO: Send MIDI messages
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Moving Heads 2 Point".to_string(),
                width: 800.0,
                height: 600.0,
                ..Default::default()
            },
            ..default()
        }))
        .add_plugin(EguiPlugin)
        .add_startup_system(setup)
        .add_system(ui_example)
        .add_system(gamepad_connections)
        .add_system(gamepad_input)
        .add_system(update_movingheads)
        .run();
}

fn ui_example(
    mut egui_context: ResMut<EguiContext>,
    target: Res<Target>,
    moving_heads: Query<&MovingHead>,
) {
    egui::Window::new("Target").show(egui_context.ctx_mut(), |ui| {
        ui.label(target.position.to_string());
    });
    egui::Window::new("Moving Heads").show(egui_context.ctx_mut(), |ui| {
        for moving_head in moving_heads.iter() {
            ui.label(moving_head.position.to_string());
            ui.label(moving_head.pan.value.to_string());
            ui.label(moving_head.tilt.value.to_string());
        }
    });
}

fn setup(mut commands: Commands) {
    // Read yaml file to get the list of moving heads

    let config_file = std::fs::read_to_string("config.yaml").unwrap();
    let config = serde_yaml::from_str::<Config>(&config_file);
    match config {
        Ok(config) => {
            for movinghead in config.moving_heads {
                commands.spawn(movinghead);
            }
            commands.insert_resource(Target {
                position: config.home,
                max: config.room,
                speed: config.speed,
            });
        }
        Err(error) => {
            println!("Error: {}", error);
        }
    }
}

fn update_movingheads(mut movingheads: Query<&mut MovingHead>, target: Res<Target>) {
    for mut movinghead in movingheads.iter_mut() {
        movinghead.point_to(target.position);
        movinghead.send_midi();
    }
}

#[derive(Resource)]
struct MyGamepad(Gamepad);

fn gamepad_connections(
    mut commands: Commands,
    my_gamepad: Option<Res<MyGamepad>>,
    mut gamepad_evr: EventReader<GamepadEvent>,
) {
    for ev in gamepad_evr.iter() {
        // the ID of the gamepad
        let id = ev.gamepad;
        match &ev.event_type {
            GamepadEventType::Connected(info) => {
                println!(
                    "New gamepad connected with ID: {:?}, name: {}",
                    id, info.name
                );

                // if we don't have any gamepad yet, use this one
                if my_gamepad.is_none() {
                    commands.insert_resource(MyGamepad(id));
                }
            }
            GamepadEventType::Disconnected => {
                println!("Lost gamepad connection with ID: {:?}", id);

                // if it's the one we previously associated with the player,
                // disassociate it:
                if let Some(MyGamepad(old_id)) = my_gamepad.as_deref() {
                    if *old_id == id {
                        commands.remove_resource::<MyGamepad>();
                    }
                }
            }
            // other events are irrelevant
            _ => {}
        }
    }
}

fn gamepad_input(
    axes: Res<Axis<GamepadAxis>>,
    buttons: Res<Input<GamepadButton>>,
    my_gamepad: Option<Res<MyGamepad>>,
    mut target: ResMut<Target>,
) {
    let gamepad = if let Some(gp) = my_gamepad {
        // a gamepad is connected, we have the id
        gp.0
    } else {
        // no gamepad is connected
        return;
    };

    // The joysticks are represented using a separate axis for X and Y
    let axis_ly = GamepadAxis {
        gamepad,
        axis_type: GamepadAxisType::LeftStickY,
    };
    let axis_ry = GamepadAxis {
        gamepad,
        axis_type: GamepadAxisType::RightStickY,
    };
    let axis_rx = GamepadAxis {
        gamepad,
        axis_type: GamepadAxisType::RightStickX,
    };

    if let (Some(x), Some(y), Some(z)) = (axes.get(axis_rx), axes.get(axis_ry), axes.get(axis_ly)) {
        let delta = Vec3::new(x, y, z) / 100.0 * target.speed;

        // Update the target position but keep it within the room
        target.position += delta;
        target.position.x = target.position.x.clamp(0.0, target.max.x);
        target.position.y = target.position.y.clamp(0.0, target.max.y);
        target.position.z = target.position.z.clamp(0.0, target.max.z);
    }
}
