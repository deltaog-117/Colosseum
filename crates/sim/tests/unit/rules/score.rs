use sim::rules::Score;

#[test]
fn a_new_score_is_zero() {
    let score = Score::default();
    assert_eq!((score.kills(), score.deaths(), score.frags()), (0, 0, 0));
}

#[test]
fn frags_are_kills_minus_deaths() {
    let mut score = Score::default();
    score.record_kill();
    score.record_kill();
    score.record_death();
    assert_eq!(score.frags(), 1);
    score.record_death();
    score.record_death();
    assert_eq!(score.frags(), -1);
}
