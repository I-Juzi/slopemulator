//! RankingBoardManager score paths (README section 6c).
use rankingdaemon::boards::*;

fn boards() -> RankingBoards {
    RankingBoards::new(15, 36, 100)
}
fn ins(season: i16, player_type: i16, id: u64, job: i16, primary: u64, t: i64) -> ScoreInsert {
    ScoreInsert {
        season_contents_type: season,
        player_type,
        player_id: id,
        job,
        primary,
        secondary: 0,
        request_time: t,
        log_id: 9,
    }
}

#[test]
fn rating_types_keep_only_the_high_dword() {
    assert_eq!(score_key(1, 0x1_2345_6789, 5, 6), (0x1_0000_0000, 5, 6));
    assert_eq!(score_key(7, 0xFFFF_FFFF, 5, 6), (0, 5, 6));
    assert_eq!(score_key(6, 0x1_2345_6789, 5, 6), (0x1_2345_6789, 5, 6));
    for t in [1, 2, 3, 4, 7, 10, 11, 12] {
        assert!(masks_low_dword(t));
    }
    for t in [5, 6, 8, 9, 13, 14, 15] {
        assert!(!masks_low_dword(t));
    }
}

#[test]
fn board_index_by_player_type() {
    assert_eq!(board_index(1, 4), 10);
    assert_eq!(board_index(2, 1), 1);
    assert_eq!(board_index(2, 2), 2);
    assert_eq!(board_index(2, 3), 0);
    assert_eq!(board_index(3, 7), 1);
    assert_eq!(board_index(4, 1), 0);
}

#[test]
fn types_without_a_list_index_have_no_board() {
    let b = boards();
    assert!(b.board(2).is_none());
    assert_eq!(b.board(8).unwrap().max_index(), 2);
    assert_eq!(b.board(13).unwrap().max_index(), 1);
    assert!(b.board(0).is_none());
    assert!(b.board(16).is_none());
}

#[test]
fn insert_validates_then_ranks() {
    let mut b = boards();
    assert_eq!(b.insert_score(&ins(0, 1, 1, 1, 10, 1)), None);
    assert_eq!(b.insert_score(&ins(6, 0, 1, 1, 10, 1)), None);
    assert_eq!(b.insert_score(&ins(6, 1, 0, 1, 10, 1)), None);
    assert_eq!(b.insert_score(&ins(6, 1, 1, 1, 0, 1)), None);
    assert_eq!(b.insert_score(&ins(6, 1, 1, 1, 10, 0)), None);
    // No board for type 2.
    assert_eq!(b.insert_score(&ins(2, 1, 1, 1, 10, 1)), None);
    // Bad job → index 0 → Insert fails.
    assert_eq!(b.insert_score(&ins(6, 1, 1, 99, 10, 1)), None);

    let r = b.insert_score(&ins(6, 1, 1, 1, 10, 1)).unwrap();
    assert_eq!(r.key, (10, 1, 9));
    assert_eq!(r.ranks, [1, 1]);
    let r = b.insert_score(&ins(6, 1, 2, 2, 20, 1)).unwrap();
    assert_eq!(r.ranks, [1, 1]);
    let r = b.insert_score(&ins(6, 1, 3, 1, 15, 1)).unwrap();
    assert_eq!(r.ranks, [2, 1]);
    // Same key twice fails.
    assert_eq!(b.insert_score(&ins(6, 1, 4, 1, 15, 1)), None);
}

#[test]
fn update_moves_the_player() {
    let mut b = boards();
    let old = b.insert_score(&ins(6, 1, 1, 1, 10, 1)).unwrap().key;
    b.insert_score(&ins(6, 1, 2, 1, 20, 1)).unwrap();
    assert_eq!(b.update_score((1, 1), 1, 6, EMPTY_KEY, 30, 2, 9), None);
    assert_eq!(b.update_score((1, 1), 1, 6, old, 30, 2, 0), None);
    assert_eq!(b.update_score((4, 1), 1, 6, old, 30, 2, 9), None);
    let r = b.update_score((1, 1), 1, 6, old, 30, 2, 9).unwrap();
    assert_eq!(r.key, (30, 2, 9));
    assert_eq!(r.ranks, [1, 1]);
    assert_eq!(b.board(6).unwrap().len(0), 2);
    let top = b.board(6).unwrap().look_up(0, 1, 1).unwrap();
    assert_eq!(top[0].1, &(1, 1));
}

#[test]
fn delete_removes_and_reports() {
    let mut b = boards();
    let k = b.insert_score(&ins(6, 1, 1, 1, 10, 1)).unwrap().key;
    assert!(!b.delete_score((1, 1), 1, 6, EMPTY_KEY));
    assert!(!b.delete_score((2, 1), 3, 6, k));
    assert!(b.delete_score((1, 1), 1, 6, k));
    assert_eq!(b.board(6).unwrap().len(0), 0);
    // Missing key: Remove still succeeds.
    assert!(b.delete_score((1, 1), 1, 6, k));
    assert!(!b.delete_score((1, 1), 1, 2, k));
}
