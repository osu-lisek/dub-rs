use rand::{distributions::Alphanumeric, Rng};
use tracing::info;

pub fn to_fixed(value: f64, decimals: i32) -> f64 {
    if decimals < 0 {
        info!("Value < 0");
        return value;
    }
    let mut rounded = value * 10f64.powi(decimals.min(6)); // Scale by at most 10^6
    rounded = rounded.round();
    rounded / 10f64.powi(decimals.min(6))
}

pub fn random_string(len: usize) -> String {
    let result: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect();

    return result;
}
