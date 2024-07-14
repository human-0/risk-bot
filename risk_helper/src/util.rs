use enum_map::EnumMap;
use risk_shared::{
    map::{TerritoryId, EDGES},
    Card, CardSymbol,
};

pub fn get_card_set(cards: &[Card]) -> Option<[Card; 3]> {
    let mut cards_by_symbol = EnumMap::from_fn(|_| Vec::new());

    for card in cards {
        cards_by_symbol[card.symbol()].push(*card);
    }

    // Try to make a different symbols set
    if cards_by_symbol.values().filter(|x| !x.is_empty()).count() >= 3 {
        let mut found = 0;
        let mut card_set: [Card; 3] = [Card::new(0).unwrap(); 3];
        for &card in cards {
            if !card_set[0..found]
                .iter()
                .any(|x| x.symbol() == card.symbol())
            {
                card_set[found] = card;
                found += 1;

                if found == 3 {
                    return Some(card_set);
                }
            }
        }

        unreachable!();
    }

    for (_, cards) in cards_by_symbol
        .iter()
        .filter(|(s, _)| *s != CardSymbol::Wildcard)
    {
        if cards.len() + cards_by_symbol[CardSymbol::Wildcard].len() >= 3 {
            let mut card_set: [Card; 3] = [Card::new(0).unwrap(); 3];
            let cards_needed = std::cmp::min(3, cards.len());
            let wildcards_need = 3_usize.saturating_sub(cards.len());

            card_set[0..cards_needed].copy_from_slice(&cards[0..cards_needed]);
            card_set[cards_needed..]
                .copy_from_slice(&cards_by_symbol[CardSymbol::Wildcard][0..wildcards_need]);

            return Some(card_set);
        }
    }

    None
}

pub fn border_territories(territories: &[TerritoryId]) -> Vec<TerritoryId> {
    let mut included = EnumMap::from_array([false; 42]);
    for &territory in territories {
        included[territory] = true;
    }

    territories
        .iter()
        .filter(|&&x| EDGES[x].iter().any(|&x| !included[x]))
        .copied()
        .collect()
}

pub fn nonborder_territories(territories: &[TerritoryId]) -> Vec<TerritoryId> {
    let mut included = EnumMap::from_array([false; 42]);
    for &territory in territories {
        included[territory] = true;
    }

    territories
        .iter()
        .filter(|&&x| EDGES[x].iter().all(|&x| included[x]))
        .copied()
        .collect()
}

pub fn adjacent_territories(territories: &[TerritoryId]) -> Vec<TerritoryId> {
    let mut included = EnumMap::from_array([false; 42]);
    for &territory in territories {
        included[territory] = true;
    }

    let mut adjacent = Vec::new();
    for &territory in territories {
        for &territory in EDGES[territory] {
            if !included[territory] {
                adjacent.push(territory);
                included[territory] = true;
            }
        }
    }

    adjacent
}
