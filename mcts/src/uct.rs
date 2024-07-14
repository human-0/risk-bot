pub fn uct(score: f64, visits: u32, parent_visits: u32) -> f64 {
    use std::f64::consts::SQRT_2;

    let visits = f64::from(visits);
    let win_rate = score / visits;

    win_rate * 20.0_f64.recip() + SQRT_2 * (f64::from(parent_visits).ln() / visits).sqrt()
}

pub fn puct(c_puct: f64, score: f64, visits: u32, parent_visits: u32, prediction: f64) -> f64 {
    assert_ne!(visits, 0);
    assert_ne!(parent_visits, 0);
    let visits = f64::from(visits);
    let win_rate = score / visits;

    let parent_visits = f64::from(parent_visits);

    win_rate + (1.5 * parent_visits.ln() / visits).sqrt()
        - c_puct * (2.0 / prediction) * (parent_visits.ln() / parent_visits).sqrt()
}

pub fn a0puct(c_puct: f64, score: f64, visits: u32, parent_visits: u32, prediction: f64) -> f64 {
    let visits = f64::from(visits);
    let parent_visits = f64::from(parent_visits);
    let win_rate = score / visits;

    win_rate + c_puct * prediction * parent_visits.sqrt() / (1.0 + visits)
}

pub fn softmax(values: &mut [f64]) {
    let mut total = 0.0;
    for value in &mut *values {
        *value = value.exp();
        total += *value;
    }

    for value in values {
        *value *= total.recip();
    }
}
