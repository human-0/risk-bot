#![no_main]
use rand_xoshiro::rand_core::SeedableRng;

#[no_mangle]
pub extern "C" fn run(seed: i32) {
    let rng = rand_xoshiro::Xoshiro256StarStar::seed_from_u64(seed as u64);
    let simple = risk_bots::strategy::PuctBot::new(rng);
    let simple = risk_helper::ManagedPlayerBot::new(simple);
    let connection = json_connection::JsonGame::new(simple);
    connection.run();
}
