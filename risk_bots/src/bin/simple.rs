use rand::prelude::SeedableRng;

fn main() {
    let rng = rand_xoshiro::Xoshiro256StarStar::seed_from_u64(0x100);
    let simple = risk_bots::simple::SimpleExample::new(rng);
    let simple = risk_helper::ManagedPlayerBot::new(simple);
    let connection = json_connection::JsonGame::new(simple);
    connection.run();
}
