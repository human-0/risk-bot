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
            territory_occupied: Eval(0.003315954179343992, 0.01733528843180152),
            weak_territory: Eval(-0.0714311161391381, -0.0037244297863779164),
            isolated_territory: Eval(-0.00040022064362791996, -0.06569914467196432),
            player_eliminated: Eval(0.9919550758267683, 1.5722449290385963),
            territory_conquered: Eval(0.4724614848987875, 0.35789063202899885),
            troop_count: Eval(0.926231230440439, 1.338092013414867),
            bias: Eval(-0.5433299644421017, -0.2201531424695424),
            resolve_k: 0.14864234033422372,
            continent_by_player: EnumMap::from_array([
                EnumMap::from_array([
                    Eval(0.9558890103221532, 0.15098427931299677),
                    Eval(-1.6776350202078967, -0.15164548820984267),
                    Eval(-1.3765896881734276, -0.232673138969994),
                    Eval(-0.3384658254959926, -0.14901338140369416),
                    Eval(-0.3717108258748576, -0.061237332536988306),
                ]),
                EnumMap::from_array([
                    Eval(0.7277694569181395, 0.0666996757872502),
                    Eval(-2.025908953786366, -0.3872655343701831),
                    Eval(-1.499885023005234, -0.0005524694821560541),
                    Eval(-0.38553733186005634, -0.49667975093608724),
                    Eval(-0.17645178084387356, -0.32060523252239903),
                ]),
                EnumMap::from_array([
                    Eval(0.9326679208533363, 0.33625543658410884),
                    Eval(-1.9888719820782672, -0.6387179048622024),
                    Eval(-1.1231484829501457, -0.6520227847420222),
                    Eval(-0.10940900128741532, -0.15745123622142884),
                    Eval(-0.0369167933802483, -0.3589365843367668),
                ]),
                EnumMap::from_array([
                    Eval(0.6307224614377632, 0.6701743236482217),
                    Eval(-1.3983444053811835, -0.6430636781471771),
                    Eval(-0.8138698577841516, -0.598918910129025),
                    Eval(-0.5431220819920496, -0.14213397636367478),
                    Eval(-0.1189081449764817, -0.13227513709024483),
                ]),
                EnumMap::from_array([
                    Eval(0.32523546520730007, 0.09992925264161054),
                    Eval(-0.6910158095290883, -0.1398995847272377),
                    Eval(-0.2301028220990661, -0.12131142983732943),
                    Eval(-0.10723667309111082, -0.2880672439340498),
                    Eval(-0.016275346914563716, -0.07026083121554327),
                ]),
                EnumMap::from_array([
                    Eval(0.8203591444204165, 0.08574972678381543),
                    Eval(-1.1552784134901546, -0.1598721191956807),
                    Eval(-0.3385357443536904, -0.18604620180146417),
                    Eval(-0.10472990623261633, -0.12217723691458308),
                    Eval(-0.17740899398813584, -0.17179401833242883),
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
