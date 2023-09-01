mod instructions;
mod vectors;

use actix_web::web::Bytes;
use awc::ws;
use enigo::*;
use futures_util::{lock::Mutex, SinkExt, StreamExt as _};
use instructions::read_instructions;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::{
    select,
    time::{sleep, timeout},
};
use vectors::Vector3D;

#[derive(Debug, Deserialize, Serialize)]
struct Coords {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Debug, Deserialize, Serialize)]
struct Head {
    y: f64,
    yaw: f64,
}

#[derive(Debug, Deserialize, Serialize)]
struct CoordinatorData {
    coords: Coords,
    head: Head,
    movement_speed: f64,
}

impl CoordinatorData {
    fn new() -> CoordinatorData {
        CoordinatorData {
            coords: Coords {
                x: 0.,
                y: 0.,
                z: 0.,
            },
            head: Head { y: 0., yaw: 0. },
            movement_speed: 0.,
        }
    }
}

struct Walking {
    run: bool,
}

impl From<f64> for Walking {
    fn from(value: f64) -> Self {
        Walking { run: value > 5. }
    }
}

struct TurningData {
    force: i32,
}

impl From<i32> for TurningData {
    fn from(value: i32) -> Self {
        TurningData { force: value }
    }
}

enum Turning {
    VLEFT(TurningData),
    VRIGHT(TurningData),
    VIDLE,
}

impl From<i32> for Turning {
    fn from(value: i32) -> Self {
        if value < 0 {
            Turning::VLEFT(TurningData::from(value))
        } else if value > 0 {
            Turning::VRIGHT(TurningData::from(value))
        } else {
            Turning::VIDLE
        }
    }
}

struct State {
    running: bool,
    walking: Option<Walking>,
    turning: Option<Turning>,
}

impl State {
    fn new() -> State {
        Self {
            running: false,
            walking: None,
            turning: None,
        }
    }
}

#[actix_web::main]
async fn main() {
    let (_, mut ws) = awc::Client::new()
        .ws("ws://127.0.0.1:8080/ws")
        .connect()
        .await
        .unwrap();

    let enigo = Arc::new(Mutex::new(Enigo::new()));
    let data = Arc::new(Mutex::new(CoordinatorData::new()));
    let state = Arc::new(Mutex::new(State::new()));

    let mut instructions = read_instructions();
    instructions.reverse();
    let instructions = Arc::new(Mutex::new(instructions));

    let data_clone = Arc::clone(&data);
    let instructions_clone = Arc::clone(&instructions);

    let enigo1 = Arc::clone(&enigo);
    let enigo2 = Arc::clone(&enigo);

    let state1 = Arc::clone(&state);
    let state2 = Arc::clone(&state);
    let state3 = Arc::clone(&state);
    let state4 = Arc::clone(&state);

    let main_task = async move {
        // Thread for logging
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_millis(50)).await;
                let state = state1.lock().await;

                if let Some(walking) = &state.walking {
                    if walking.run {
                        println!("[LOG]: Running");
                    } else {
                        println!("[LOG]: Walking");
                    }
                }

                if let Some(turning) = &state.turning {
                    match turning {
                        Turning::VLEFT(_force) => println!("[LOG]: Turning Left"),
                        Turning::VRIGHT(_force) => println!("[LOG]: Turning Right"),
                        Turning::VIDLE => (),
                    }
                }
            }
        });

        // Thread for deciding to turn
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_millis(50)).await;
                let mut enigo = enigo1.lock().await;
                let state = state2.lock().await;

                if let Some(turning) = &state.turning {
                    match turning {
                        Turning::VLEFT(data) => enigo.mouse_move_relative(data.force * -1, 0),
                        Turning::VRIGHT(data) => enigo.mouse_move_relative(data.force * -1, 0),
                        Turning::VIDLE => (),
                    }
                }
            }
        });

        // Thread for deciding to walk
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_millis(50)).await;
                let mut enigo = enigo2.lock().await;
                let mut state = state3.lock().await;

                if let Some(walking) = &mut state.walking {
                    if walking.run {
                        enigo.key_down(Key::Control);
                        state.running = true;
                    } else if state.running {
                        state.running = false;
                        enigo.key_up(Key::W);
                        enigo.key_up(Key::Control);
                        sleep(Duration::from_millis(50)).await;
                    }

                    enigo.key_down(Key::W);
                } else {
                    enigo.key_up(Key::W);
                }
            }
        });

        // Thread for managing the struc 'State'
        tokio::spawn(async move {
            let mut instructions = instructions_clone.lock().await;
            while let Some(instruction) = instructions.last().copied() {
                sleep(Duration::from_millis(50)).await;
                let mut state = state4.lock().await;
                let data = data_clone.lock().await;
                let goal = Vector3D::from(instruction.walk.to);

                let my_position = Vector3D::from(&data.coords);
                let my_angle = data.head.yaw;

                let direction = goal - my_position;

                let direction_angle = direction.get_normalized().to_angle();
                let mut raw_angle_diff = my_angle - direction_angle;

                while raw_angle_diff > 180.0 {
                    raw_angle_diff -= 360.0;
                }

                while raw_angle_diff < -180.0 {
                    raw_angle_diff += 360.0;
                }

                let force = (raw_angle_diff as i32).max(-32).min(32);

                if force == 0 {
                    state.turning = None
                } else {
                    state.turning = Some(Turning::from(force));
                }

                let distance = (goal - my_position).get_magnitude();

                if distance < 1. {
                    state.walking = None;
                    instructions.pop();
                } else {
                    state.walking = Some(Walking::from(distance));
                }
            }
        });

        loop {
            select! {
              Some(msg) = ws.next() => {
                match msg {
                  Ok(ws::Frame::Text(msg)) => {
                    let blabla = serde_json::from_slice::<CoordinatorData>(&msg);

                    match blabla {
                        Ok(parse) => {
                          let mut data = data.lock().await;

                          data.coords.x = parse.coords.x;
                          data.coords.y = parse.coords.y;
                          data.coords.z = parse.coords.z;
                          data.head.yaw = parse.head.yaw;
                          data.head.y = parse.head.y;
                          data.movement_speed = parse.movement_speed;
                        },
                        Err(err) => {
                          println!("Failed to parse the coordinator data");
                          println!("{err:?}");
                        },
                    }


                  },

                  Ok(ws::Frame::Ping(_)) => {
                    ws.send(ws::Message::Pong(Bytes::new())).await.unwrap();
                  },

                  _ => ()
                }
              }

              _ = sleep(Duration::from_secs(30)) => {
                println!("Terminating after 30 seconds");
                break;
              }
            }
        }
    };

    if let Err(_) = timeout(Duration::from_secs(30), main_task).await {
        println!("Terminating after 30 seconds.");
    }
}
