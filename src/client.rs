mod vectors;

use actix_web::web::Bytes;
use awc::ws;
use enigo::*;
use futures_util::{lock::Mutex, SinkExt, StreamExt as _};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::{borrow::BorrowMut, sync::Arc};
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
struct CoordinatorData {
    coords: Coords,
    head_angle: f64,
}

#[actix_web::main]
async fn main() {
    let (_, mut ws) = awc::Client::new()
        .ws("ws://127.0.0.1:8080/ws")
        .connect()
        .await
        .unwrap();

    let enigo = Arc::new(Mutex::new(Enigo::new()));
    let data = Arc::new(Mutex::new(CoordinatorData {
        coords: Coords {
            x: 0.,
            y: 0.,
            z: 0.,
        },
        head_angle: 0.,
    }));

    let enigo_clone = Arc::clone(&enigo);
    let data_clone = Arc::clone(&data);

    let goal_position = Arc::new(Mutex::new(Vector3D {
        x: -120.5,
        y: 0.,
        z: 154.5,
    }));

    let goal_clone = Arc::clone(&goal_position);

    let main_task = async move {
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_millis(50)).await;
                let data = data_clone.lock().await;
                let mut enigo = enigo_clone.lock().await;

                let my_position = Vector3D {
                    x: data.coords.x,
                    y: 0.,
                    z: data.coords.z,
                };
                let my_angle = data.head_angle;
                let goal = goal_clone.lock().await.to_owned();

                let direction: Vector3D = goal - my_position;

                let mut turn_to_direction = || -> bool {
                    let direction_angle = direction.get_normalized().to_angle();
                    let angle_diff = my_angle - direction_angle;
                    let mut clamped_angle_diff = angle_diff;

                    while clamped_angle_diff > 180.0 {
                        clamped_angle_diff -= 360.0;
                    }

                    while clamped_angle_diff < -180.0 {
                        clamped_angle_diff += 360.0;
                    }

                    // Limit the amount to move based on the clamped angle difference
                    let to_move = (clamped_angle_diff as i32).max(-32).min(32);

                    if to_move == 0 {
                        return true;
                    }

                    println!(
                        "!!!TURNING | Angle_diff: {} | Clamped: {} | Moving: {}",
                        angle_diff as i32, clamped_angle_diff as i32, to_move
                    );

                    enigo.mouse_move_relative(to_move * -1, 0);

                    return false;
                };

                if turn_to_direction() == true {
                    let distance = (goal - my_position).get_magnitude();

                    if distance > 0.5 {
                        println!("!!!WALKING | Magnitude: {}", distance);
                        enigo.key_click(Key::W);
                    }
                } else {
                    println!("!!!IDLE");
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
                          data.head_angle = parse.head_angle;
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
