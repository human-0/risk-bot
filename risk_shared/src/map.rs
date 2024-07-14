use enum_map::{Enum, EnumMap};

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, enum_map::Enum, enumn::N)]
#[cfg_attr(
    feature = "serde",
    derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr)
)]
pub enum TerritoryId {
    Alaska,
    Alberta,
    CentralAmerica,
    EasternUs,
    Greenland,
    NorthwestTerritory,
    Ontario,
    Quebec,
    WesternUs,
    GreatBritain,
    Iceland,
    NorthernEurope,
    Scandinavia,
    SouthernEurope,
    Ukraine,
    WesternEurope,
    Afghanistan,
    China,
    India,
    Irkutsk,
    Japan,
    Kamchatka,
    MiddleEast,
    Mongolia,
    Siam,
    Siberia,
    Ural,
    Yakutsk,
    Argentina,
    Brazil,
    Venezuela,
    Peru,
    Congo,
    EastAfrica,
    Egypt,
    Madagascar,
    NorthAfrica,
    SouthAfrica,
    EasternAustralia,
    NewGuinea,
    Indonesia,
    WesternAustralia,
}

impl TerritoryId {
    pub const ALL: [Self; <Self as enum_map::Enum>::LENGTH] = const {
        let mut values = unsafe { [std::mem::transmute::<u8, Self>(0); Self::LENGTH] };

        let mut i = 0;
        while i < Self::LENGTH {
            values[i] = unsafe { std::mem::transmute::<u8, Self>(i as u8) };

            i += 1;
        }

        values
    };

    pub const fn continent(self) -> Continent {
        CONTINENTS.as_array()[self as usize]
    }
}

#[cfg_attr(feature = "serde", derive(serde_repr::Deserialize_repr))]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, enumn::N, enum_map::Enum)]
#[repr(u8)]
pub enum Continent {
    NorthAmerica,
    Europe,
    Asia,
    SouthAmerica,
    Africa,
    Australia,
}

impl Continent {
    pub const ALL: [Self; 6] = [
        Self::NorthAmerica,
        Self::Europe,
        Self::Asia,
        Self::SouthAmerica,
        Self::Africa,
        Self::Australia,
    ];

    pub const fn territory_count(self) -> u32 {
        [9, 7, 12, 4, 6, 4][self as usize]
    }

    pub const fn bonus(self) -> u32 {
        [5, 5, 7, 2, 3, 2][self as usize]
    }

    pub fn iter_territories(self) -> impl Iterator<Item = TerritoryId> {
        TerritoryId::ALL
            .into_iter()
            .filter(move |x| x.continent() == self)
    }
}

macro_rules! edge_arrays {
    ($($origin:expr => {
        $($dest:expr,)*
    },)*) => {
        const {
            let mut array: [&[$crate::map::TerritoryId]; <$crate::map::TerritoryId as ::enum_map::Enum>::LENGTH] = [&[]; <$crate::map::TerritoryId as ::enum_map::Enum>::LENGTH];
            $(
                array[$origin as ::core::primitive::usize] = &[
                    $($dest,)*
                ];
            )*

            array
        }
    };
}

pub const EDGES: EnumMap<TerritoryId, &[TerritoryId]> = EnumMap::from_array(edge_arrays! {
    TerritoryId::Alaska => {
        TerritoryId::Alberta,
        TerritoryId::NorthwestTerritory,
        TerritoryId::Kamchatka,
    },
    TerritoryId::Alberta => {
        TerritoryId::Ontario,
        TerritoryId::NorthwestTerritory,
        TerritoryId::Alaska,
        TerritoryId::WesternUs,
    },
    TerritoryId::CentralAmerica => {
        TerritoryId::EasternUs,
        TerritoryId::WesternUs,
        TerritoryId::Venezuela,
    },
    TerritoryId::EasternUs => {
        TerritoryId::Quebec,
        TerritoryId::Ontario,
        TerritoryId::WesternUs,
        TerritoryId::CentralAmerica,
    },
    TerritoryId::Greenland => {
        TerritoryId::NorthwestTerritory,
        TerritoryId::Ontario,
        TerritoryId::Quebec,
        TerritoryId::Iceland,
    },
    TerritoryId::NorthwestTerritory => {
        TerritoryId::Greenland,
        TerritoryId::Alaska,
        TerritoryId::Alberta,
        TerritoryId::Ontario,
    },
    TerritoryId::Ontario => {
        TerritoryId::Quebec,
        TerritoryId::Greenland,
        TerritoryId::NorthwestTerritory,
        TerritoryId::Alberta,
        TerritoryId::WesternUs,
        TerritoryId::EasternUs,
    },
    TerritoryId::Quebec => {
        TerritoryId::Greenland,
        TerritoryId::Ontario,
        TerritoryId::EasternUs,
    },
    TerritoryId::WesternUs => {
        TerritoryId::EasternUs,
        TerritoryId::Ontario,
        TerritoryId::Alberta,
        TerritoryId::CentralAmerica,
    },
    TerritoryId::GreatBritain => {
        TerritoryId::NorthernEurope,
        TerritoryId::Scandinavia,
        TerritoryId::Iceland,
        TerritoryId::WesternEurope,
    },
    TerritoryId::Iceland => {
        TerritoryId::Scandinavia,
        TerritoryId::Greenland,
        TerritoryId::GreatBritain,
    },
    TerritoryId::NorthernEurope => {
        TerritoryId::Ukraine,
        TerritoryId::Scandinavia,
        TerritoryId::GreatBritain,
        TerritoryId::WesternEurope,
        TerritoryId::SouthernEurope,
    },
    TerritoryId::Scandinavia => {
        TerritoryId::Ukraine,
        TerritoryId::Iceland,
        TerritoryId::GreatBritain,
        TerritoryId::NorthernEurope,
    },
    TerritoryId::SouthernEurope => {
        TerritoryId::MiddleEast,
        TerritoryId::Ukraine,
        TerritoryId::NorthernEurope,
        TerritoryId::WesternEurope,
        TerritoryId::NorthAfrica,
        TerritoryId::Egypt,
    },
    TerritoryId::Ukraine => {
        TerritoryId::Afghanistan,
        TerritoryId::Ural,
        TerritoryId::Scandinavia,
        TerritoryId::NorthernEurope,
        TerritoryId::SouthernEurope,
        TerritoryId::MiddleEast,
    },
    TerritoryId::WesternEurope => {
        TerritoryId::SouthernEurope,
        TerritoryId::NorthernEurope,
        TerritoryId::GreatBritain,
        TerritoryId::NorthAfrica,
    },
    TerritoryId::Afghanistan => {
        TerritoryId::China,
        TerritoryId::Ural,
        TerritoryId::Ukraine,
        TerritoryId::MiddleEast,
        TerritoryId::India,
    },
    TerritoryId::China => {
        TerritoryId::Mongolia,
        TerritoryId::Siberia,
        TerritoryId::Ural,
        TerritoryId::Afghanistan,
        TerritoryId::India,
        TerritoryId::Siam,
    },
    TerritoryId::India => {
        TerritoryId::Siam,
        TerritoryId::China,
        TerritoryId::Afghanistan,
        TerritoryId::MiddleEast,
    },
    TerritoryId::Irkutsk => {
        TerritoryId::Kamchatka,
        TerritoryId::Yakutsk,
        TerritoryId::Siberia,
        TerritoryId::Mongolia,
    },
    TerritoryId::Japan => {
        TerritoryId::Kamchatka,
        TerritoryId::Mongolia,
    },
    TerritoryId::Kamchatka => {
        TerritoryId::Alaska,
        TerritoryId::Yakutsk,
        TerritoryId::Irkutsk,
        TerritoryId::Mongolia,
        TerritoryId::Japan,
    },
    TerritoryId::MiddleEast => {
        TerritoryId::India,
        TerritoryId::Afghanistan,
        TerritoryId::Ukraine,
        TerritoryId::SouthernEurope,
        TerritoryId::Egypt,
        TerritoryId::EastAfrica,
    },
    TerritoryId::Mongolia => {
        TerritoryId::Japan,
        TerritoryId::Kamchatka,
        TerritoryId::Irkutsk,
        TerritoryId::Siberia,
        TerritoryId::China,
    },
    TerritoryId::Siam => {
        TerritoryId::China,
        TerritoryId::India,
        TerritoryId::Indonesia,
    },
    TerritoryId::Siberia => {
        TerritoryId::Yakutsk,
        TerritoryId::Ural,
        TerritoryId::China,
        TerritoryId::Mongolia,
        TerritoryId::Irkutsk,
    },
    TerritoryId::Ural => {
        TerritoryId::Siberia,
        TerritoryId::Ukraine,
        TerritoryId::Afghanistan,
        TerritoryId::China,
    },
    TerritoryId::Yakutsk => {
        TerritoryId::Kamchatka,
        TerritoryId::Siberia,
        TerritoryId::Irkutsk,
    },
    TerritoryId::Argentina => {
        TerritoryId::Brazil,
        TerritoryId::Peru,
    },
    TerritoryId::Brazil => {
        TerritoryId::NorthAfrica,
        TerritoryId::Venezuela,
        TerritoryId::Peru,
        TerritoryId::Argentina,
    },
    TerritoryId::Venezuela => {
        TerritoryId::CentralAmerica,
        TerritoryId::Peru,
        TerritoryId::Brazil,
    },
    TerritoryId::Peru => {
        TerritoryId::Brazil,
        TerritoryId::Venezuela,
        TerritoryId::Argentina,
    },
    TerritoryId::Congo => {
        TerritoryId::EastAfrica,
        TerritoryId::NorthAfrica,
        TerritoryId::SouthAfrica,
    },
    TerritoryId::EastAfrica => {
        TerritoryId::MiddleEast,
        TerritoryId::Egypt,
        TerritoryId::NorthAfrica,
        TerritoryId::Congo,
        TerritoryId::SouthAfrica,
        TerritoryId::Madagascar,
    },
    TerritoryId::Egypt => {
        TerritoryId::MiddleEast,
        TerritoryId::SouthernEurope,
        TerritoryId::NorthAfrica,
        TerritoryId::EastAfrica,
    },
    TerritoryId::Madagascar => {
        TerritoryId::EastAfrica,
        TerritoryId::SouthAfrica,
    },
    TerritoryId::NorthAfrica => {
        TerritoryId::EastAfrica,
        TerritoryId::Egypt,
        TerritoryId::SouthernEurope,
        TerritoryId::WesternEurope,
        TerritoryId::Brazil,
        TerritoryId::Congo,
    },
    TerritoryId::SouthAfrica => {
        TerritoryId::Madagascar,
        TerritoryId::EastAfrica,
        TerritoryId::Congo,
    },
    TerritoryId::EasternAustralia => {
        TerritoryId::NewGuinea,
        TerritoryId::WesternAustralia,
    },
    TerritoryId::NewGuinea => {
        TerritoryId::Indonesia,
        TerritoryId::WesternAustralia,
        TerritoryId::EasternAustralia,
    },
    TerritoryId::Indonesia => {
        TerritoryId::NewGuinea,
        TerritoryId::Siam,
        TerritoryId::WesternAustralia,
    },
    TerritoryId::WesternAustralia => {
        TerritoryId::EasternAustralia,
        TerritoryId::NewGuinea,
        TerritoryId::Indonesia,
    },
});

macro_rules! const_enum_map {
    ($($territory:expr => $continent:expr,)*) => {
        const {
            let mut i = 0;
            $(
                if i != $territory as ::core::primitive::usize {
                    ::core::panic!("Enum entries must be in order");
                }
                i += 1;
            )*

                let _ = i; // Silence unused warning
            ::enum_map::EnumMap::from_array([
                $($continent,)*
            ])
        }
    };
}

const CONTINENTS: EnumMap<TerritoryId, Continent> = const_enum_map! {
    TerritoryId::Alaska => Continent::NorthAmerica,
    TerritoryId::Alberta => Continent::NorthAmerica,
    TerritoryId::CentralAmerica => Continent::NorthAmerica,
    TerritoryId::EasternUs => Continent::NorthAmerica,
    TerritoryId::Greenland => Continent::NorthAmerica,
    TerritoryId::NorthwestTerritory => Continent::NorthAmerica,
    TerritoryId::Ontario => Continent::NorthAmerica,
    TerritoryId::Quebec => Continent::NorthAmerica,
    TerritoryId::WesternUs => Continent::NorthAmerica,
    TerritoryId::GreatBritain => Continent::Europe,
    TerritoryId::Iceland => Continent::Europe,
    TerritoryId::NorthernEurope => Continent::Europe,
    TerritoryId::Scandinavia => Continent::Europe,
    TerritoryId::SouthernEurope => Continent::Europe,
    TerritoryId::Ukraine => Continent::Europe,
    TerritoryId::WesternEurope => Continent::Europe,
    TerritoryId::Afghanistan => Continent::Asia,
    TerritoryId::China => Continent::Asia,
    TerritoryId::India => Continent::Asia,
    TerritoryId::Irkutsk => Continent::Asia,
    TerritoryId::Japan => Continent::Asia,
    TerritoryId::Kamchatka => Continent::Asia,
    TerritoryId::MiddleEast => Continent::Asia,
    TerritoryId::Mongolia => Continent::Asia,
    TerritoryId::Siam => Continent::Asia,
    TerritoryId::Siberia => Continent::Asia,
    TerritoryId::Ural => Continent::Asia,
    TerritoryId::Yakutsk => Continent::Asia,
    TerritoryId::Argentina => Continent::SouthAmerica,
    TerritoryId::Brazil => Continent::SouthAmerica,
    TerritoryId::Venezuela => Continent::SouthAmerica,
    TerritoryId::Peru => Continent::SouthAmerica,
    TerritoryId::Congo => Continent::Africa,
    TerritoryId::EastAfrica => Continent::Africa,
    TerritoryId::Egypt => Continent::Africa,
    TerritoryId::Madagascar => Continent::Africa,
    TerritoryId::NorthAfrica => Continent::Africa,
    TerritoryId::SouthAfrica => Continent::Africa,
    TerritoryId::EasternAustralia => Continent::Australia,
    TerritoryId::NewGuinea => Continent::Australia,
    TerritoryId::Indonesia => Continent::Australia,
    TerritoryId::WesternAustralia => Continent::Australia,
};
