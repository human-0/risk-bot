use enum_map::EnumMap;
use risk_shared::{
    map::{Continent, TerritoryId, EDGES},
    player::PlayerId,
};

use crate::game::AttackGame;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Eval(pub f64, pub f64);

impl Eval {
    pub fn accum(&mut self, scale: f64, other: Self) -> &mut Self {
        self.0 += scale * other.0;
        self.1 += scale * other.1;
        self
    }

    pub fn resolve(self, k: f64, card_sets_redeemed: u32) -> f64 {
        let phase = (-k * card_sets_redeemed as f64).exp();
        self.0 * phase + self.1 * (1.0 - phase)
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Params {
    pub territory_occupied: Eval,
    pub weak_territory: Eval,
    pub isolated_territory: Eval,
    pub player_eliminated: Eval,
    pub territory_conquered: Eval,
    pub troop_count: Eval,
    pub bias: Eval,
    pub resolve_k: f64,
    pub continent_by_player: EnumMap<Continent, EnumMap<PlayerId, Eval>>,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            territory_occupied: Eval(0.004602183368215947, 0.013104483261206624),
            weak_territory: Eval(-0.06986899701902574, -0.006039317807500187),
            isolated_territory: Eval(-0.0023445806366162577, -0.08354679434136691),
            player_eliminated: Eval(1.2571525464830933, 1.8348492592151964),
            territory_conquered: Eval(0.251873746041146, 0.33770224917234126),
            troop_count: Eval(0.9379646647806544, 1.267830540861828),
            resolve_k: 0.14662364924017113,
            bias: Eval(-0.6038499097696227, -0.36524733244238594),
            continent_by_player: EnumMap::from_array([
                EnumMap::from_array([
                    Eval(1.0694851061151829, 0.06441334684074566),
                    Eval(-1.5597025025673659, -0.15745570400215952),
                    Eval(-1.6228947105965463, -0.11907015330387281),
                    Eval(-0.33870410392482064, -0.1364071630400263),
                    Eval(-0.5356322184801511, -0.10317302189366849),
                ]),
                EnumMap::from_array([
                    Eval(0.5491656406773419, 0.37041034537987116),
                    Eval(-2.1910144018747886, -0.368342577692111),
                    Eval(-1.4249260276555964, -0.005275070987203928),
                    Eval(-0.18566724389644645, -0.6451241430352684),
                    Eval(-0.08033828887263685, -0.42379664960382146),
                ]),
                EnumMap::from_array([
                    Eval(0.8047103607875169, 0.18521893217969115),
                    Eval(-1.9735927542878395, -0.6161537374725065),
                    Eval(-1.1728623532918598, -0.4978172218341361),
                    Eval(-0.053214523047240135, -0.15567833749883137),
                    Eval(-0.09133174485266425, -0.3255621448024393),
                ]),
                EnumMap::from_array([
                    Eval(0.5046775660789545, 0.505476133893031),
                    Eval(-1.4440393129525158, -0.48812430672181506),
                    Eval(-0.8993329617218632, -0.5935821029043132),
                    Eval(-0.5023216419717192, -0.28579123517483485),
                    Eval(-0.31250103320655964, -0.06674038325837807),
                ]),
                EnumMap::from_array([
                    Eval(0.4735398110186245, 0.24255722776766536),
                    Eval(-0.909810061036103, -0.10508908393099618),
                    Eval(-0.3640056552462979, -0.20906185192844956),
                    Eval(-0.4112714749276394, -0.15229075557518612),
                    Eval(-0.0700342694809049, -0.2047943403306252),
                ]),
                EnumMap::from_array([
                    Eval(0.7927570999552106, 0.01368859451191441),
                    Eval(-1.1850267624497925, -0.017359392018905993),
                    Eval(-0.26344912884447114, -0.26016772288891943),
                    Eval(-0.2153862156931699, -0.11649649242733816),
                    Eval(-0.32683830722755913, -0.06966700121303006),
                ]),
            ]),
        }
    }
}

pub fn evaluate(game: &AttackGame, params: &Params) -> f64 {
    let mut territories_occupied = 0;
    let mut continent_territories_occupied = EnumMap::from_fn(|_| EnumMap::from_fn(|_| 0));
    let mut weak_territories = 0;
    let mut isolated_territories = 0;
    let mut my_troops = 0;
    let mut total_troops = 0;

    for territory in TerritoryId::ALL {
        let troops = game.troops(territory);
        if game.occupier(territory).is_p0() {
            territories_occupied += 1;
            my_troops += troops;
            total_troops += troops;

            let mut weak = false;
            let mut isolated = true;
            for &t in EDGES[territory] {
                if game.occupier(t).is_p0() {
                    isolated = false;
                } else if troops == 1 {
                    weak = true;
                }
            }

            if weak {
                weak_territories += 1;
            }

            if isolated {
                isolated_territories += 1
            }
        } else {
            total_troops += troops;
        }

        continent_territories_occupied[territory.continent()][game.occupier(territory)] += 1;
    }

    let expected_proportion = 1.0 / game.initial_players() as f64;
    let expected_troops = total_troops as f64 * expected_proportion;

    #[rustfmt::skip]
    let mut score = params.bias;
    score
        .accum(territories_occupied as f64, params.territory_occupied)
        .accum(weak_territories as f64, params.weak_territory)
        .accum(isolated_territories as f64, params.isolated_territory)
        .accum(
            1.0 - (expected_troops / my_troops as f64).sqrt(),
            params.troop_count,
        )
        .accum(game.players_eliminated() as f64, params.player_eliminated);

    for (continent, occupiers) in continent_territories_occupied {
        if let Some((occupier, _)) = occupiers
            .into_iter()
            .find(|(_, count)| *count == continent.territory_count())
        {
            score.accum(1.0, params.continent_by_player[continent][occupier]);
        }
    }

    if game.territory_conquered() {
        score.accum(1.0, params.territory_conquered);
    }

    1.0 / (1.0 + (-score.resolve(params.resolve_k, game.card_sets_redeemed())).exp())
}
