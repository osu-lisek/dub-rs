use rand::{distributions::Alphanumeric, Rng};
use tracing::info;

pub fn to_fixed(value: f64, decimals: i32) -> f64 {
    if decimals < 0 {
        info!("Value < 0");
        return value;
    }

    let y = 10i32.pow(decimals as u32) as f64;
    (value * y).round() / y
}

pub fn random_string(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}
