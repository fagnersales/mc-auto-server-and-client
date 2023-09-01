use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub struct Walk {
    pub from: [f64; 3],
    pub to: [f64; 3],
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub struct Instruction {
    pub walk: Walk,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    instruction: Vec<Instruction>,
}

pub fn read_instructions() -> Vec<Instruction> {
    let content = include_str!("instructions.toml");

    let config = toml::from_str::<Config>(&content).unwrap();

    config.instruction
}
