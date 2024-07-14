use risk_shared::player::PlayerId;

use crate::game::{AttackGame, Move, PlayerMove};

pub mod puct;
pub mod state;

pub trait StrategyExt: mcts::Strategy<Action = Move> {
    type Params;
    type Rng: rand::RngCore;

    fn from_rng(root_game: AttackGame, rng: Self::Rng) -> Self;
    fn from_params_rng(root_game: AttackGame, params: Self::Params, rng: Self::Rng) -> Self;
    fn create_from(&self, root_game: AttackGame) -> Self;

    fn root_game(&self) -> &AttackGame;
    fn evaluate(&self, game: &AttackGame) -> f64;

    fn simulation_rounds(&self) -> u64;
    fn reset_simulation_rounds(&self);
}

const PROBABILITIES: [[WeightedAlias; 2]; 3] = [
    [
        WeightedAlias::new([0, 15, 21, 0, 0, 0, 0, 0]),
        WeightedAlias::new([0, 55, 0, 161, 0, 0, 0, 0]),
    ],
    [
        WeightedAlias::new([0, 125, 91, 0, 0, 0, 0, 0]),
        WeightedAlias::new([0, 0, 295, 0, 420, 0, 581, 0]),
    ],
    [
        WeightedAlias::new([0, 441, 855, 0, 0, 0, 0, 0]),
        WeightedAlias::new([0, 0, 2890, 0, 2611, 0, 2275, 0]),
    ],
];

const CHANCE_MOVES: [[[Move; 8]; 2]; 3] = const {
    let mut moves = [[[Move::Chance(0, 0); 8]; 2]; 3];

    let mut num_attackers = 1;
    while num_attackers <= 3 {
        let mut num_defenders = 1;
        while num_defenders <= 2 {
            let mut i = 0;
            while i < 8 {
                let attackers_lost = i / (num_defenders + 1);
                let defenders_lost = i % (num_defenders + 1);

                moves[num_attackers - 1][num_defenders - 1][i] =
                    Move::Chance(attackers_lost as u8, defenders_lost as u8);
                i += 1;
            }

            num_defenders += 1;
        }

        num_attackers += 1;
    }

    moves
};

fn resolve_chance(game: &AttackGame, mov: PlayerMove, rng: &mut impl rand::Rng) -> Move {
    assert_eq!(game.occupier(mov.origin), PlayerId::P0);
    assert_ne!(game.occupier(mov.dest), PlayerId::P0);
    let origin_troops: usize = game.troops(mov.origin).try_into().unwrap();
    let dest_troops = game.troops(mov.dest).try_into().unwrap();

    let num_attackers = std::cmp::min(origin_troops - 1, 3);
    let num_defenders = std::cmp::min(dest_troops, 2);
    let result = PROBABILITIES[num_attackers - 1][num_defenders - 1].sample(rng);
    CHANCE_MOVES[num_attackers - 1][num_defenders - 1][result]
}

struct WeightedAlias {
    u_table: [u32; 8],
    k_table: [u32; 8],
}

impl WeightedAlias {
    const fn new(weights: [u32; 8]) -> Self {
        let sum = weights[0]
            + weights[1]
            + weights[2]
            + weights[3]
            + weights[4]
            + weights[5]
            + weights[6]
            + weights[7];

        let mut u_table = [
            8 * weights[0],
            8 * weights[1],
            8 * weights[2],
            8 * weights[3],
            8 * weights[4],
            8 * weights[5],
            8 * weights[6],
            8 * weights[7],
        ];

        let mut k_table = [0, 1, 2, 3, 4, 5, 6, 7];

        enum Fullness {
            Over,
            Under,
            Exact,
        }

        const fn fullness(value: u32, sum: u32) -> Fullness {
            if value > sum {
                Fullness::Over
            } else if value < sum {
                Fullness::Under
            } else {
                Fullness::Exact
            }
        }

        let mut fullness_table = [
            fullness(u_table[0], sum),
            fullness(u_table[1], sum),
            fullness(u_table[2], sum),
            fullness(u_table[3], sum),
            fullness(u_table[4], sum),
            fullness(u_table[5], sum),
            fullness(u_table[6], sum),
            fullness(u_table[7], sum),
        ];

        'a: loop {
            let (overfull, underfull) = {
                let mut i = 0;
                let mut overfull = None;
                let mut underfull = None;
                while i < 8 {
                    match fullness_table[i] {
                        Fullness::Over => overfull = Some(i),
                        Fullness::Under => underfull = Some(i),
                        Fullness::Exact => (),
                    }

                    i += 1;
                }

                match (overfull, underfull) {
                    (Some(overfull), Some(underfull)) => (overfull, underfull),
                    (None, None) => break 'a,
                    _ => unreachable!(),
                }
            };

            k_table[underfull] = overfull as u32;
            u_table[overfull] = u_table[overfull] + u_table[underfull] - sum;
            fullness_table[underfull] = Fullness::Exact;
            fullness_table[overfull] = fullness(u_table[overfull], sum);
        }

        // Rescale
        let mut i = 0;
        while i < 8 {
            u_table[i] = ((u_table[i] as u64 * (1_u64 << 29)) / sum as u64) as u32;
            i += 1;
        }

        Self { u_table, k_table }
    }

    fn sample(&self, rng: &mut impl rand::Rng) -> usize {
        let value = rng.next_u32();
        let i = (value & 0x7) as usize;
        let y = value >> 3;
        if y < self.u_table[i] {
            i
        } else {
            self.k_table[i] as usize
        }
    }
}
