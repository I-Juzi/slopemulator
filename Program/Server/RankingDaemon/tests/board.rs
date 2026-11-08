//! SkipListExManager port (README section 6).
use rankingdaemon::board::*;

fn board(max_nodes: i32) -> Board<u32> {
    Board::new(BoardInit {
        max_level: 15,
        prob: 36,
        max_node_count: max_nodes,
        max_index: 3,
    })
    .unwrap()
}
fn k(a: u64) -> Key {
    (a, 0, 0)
}

#[test]
fn init_rejects_bad_values() {
    let ok = BoardInit {
        max_level: 15,
        prob: 36,
        max_node_count: 10,
        max_index: 14,
    };
    assert!(Board::<u32>::new(ok).is_ok());
    for bad in [
        BoardInit {
            max_level: 16,
            ..ok
        },
        BoardInit { prob: 101, ..ok },
        BoardInit {
            max_node_count: 0,
            ..ok
        },
        BoardInit { max_index: 0, ..ok },
        BoardInit {
            max_index: 15,
            ..ok
        },
    ] {
        assert_eq!(
            Board::<u32>::new(bad).unwrap_err(),
            BoardError::InvalidParameter
        );
    }
}

#[test]
fn larger_key_ranks_higher_in_both_lists() {
    let mut b = board(100);
    assert_eq!(b.insert(k(10), 1, 1), Ok([1, 1]));
    assert_eq!(b.insert(k(30), 2, 2), Ok([1, 1]));
    assert_eq!(b.insert(k(20), 3, 1), Ok([2, 1]));
    assert_eq!(b.insert((20, 0, 1), 4, 1), Ok([2, 1]));
    assert_eq!(b.search(k(10), 1), Ok([4, 3]));
    assert_eq!(b.search(k(30), 1), Ok([1, 0]));
    assert_eq!(b.search(k(99), 1), Ok([0, 0]));
    assert_eq!(b.len(0), 4);
    assert_eq!(b.len(1), 3);
}

#[test]
fn index_must_be_a_job_list() {
    let mut b = board(100);
    assert_eq!(b.insert(k(1), 1, 0), Err(BoardError::InvalidParameter));
    assert_eq!(b.insert(k(1), 1, 4), Err(BoardError::InvalidParameter));
    assert_eq!(b.remove(k(1), 0), Err(BoardError::InvalidParameter));
    assert!(b.is_empty());
}

#[test]
fn duplicate_key_fails_and_rolls_back() {
    let mut b = board(100);
    b.insert(k(5), 1, 1).unwrap();
    assert_eq!(b.insert(k(5), 2, 1), Err(BoardError::OverlappedKey));
    assert_eq!(b.len(0), 1);
    // Same key in another job list: list 0 rejects it first.
    assert_eq!(b.insert(k(5), 2, 2), Err(BoardError::OverlappedKey));
    assert_eq!(b.len(2), 0);
}

#[test]
fn full_list_skips_below_tail_and_evicts_above() {
    let mut b = board(2);
    b.insert(k(10), 1, 1).unwrap();
    b.insert(k(20), 2, 1).unwrap();
    assert_eq!(b.insert(k(5), 3, 1), Ok([0, 0]));
    assert_eq!(b.len(0), 2);
    assert_eq!(b.insert(k(15), 4, 1), Ok([2, 2]));
    assert_eq!(b.search(k(10), 1), Ok([0, 0]));
    assert_eq!(b.insert(k(15), 5, 1), Err(BoardError::OverlappedKey));
    // Another job list still has room although list 0 is full.
    assert_eq!(b.insert(k(1), 6, 2), Ok([0, 1]));
}

#[test]
fn remove_takes_node_out_of_both_lists() {
    let mut b = board(100);
    b.insert(k(10), 1, 2).unwrap();
    b.insert(k(20), 2, 1).unwrap();
    // Wrong job index still finds it through list 0.
    assert_eq!(b.remove(k(10), 1), Ok(()));
    assert_eq!(b.len(0), 1);
    assert_eq!(b.len(2), 0);
    assert_eq!(b.remove(k(99), 1), Ok(()));
}

#[test]
fn update_moves_node_and_keeps_value() {
    let mut b = board(100);
    b.insert(k(10), 7, 1).unwrap();
    b.insert(k(20), 8, 1).unwrap();
    assert_eq!(b.update(k(10), k(30), 0, 1), Ok([1, 1]));
    let top = b.look_up(1, 1, 1).unwrap();
    assert_eq!(top, vec![(k(30), &7)]);
    // Missing old key: a new node with the given value.
    assert_eq!(b.update(k(1), k(5), 9, 2), Ok([3, 1]));
    assert_eq!(b.look_up(2, 1, 5).unwrap(), vec![(k(5), &9)]);
}

#[test]
fn look_up_pages_and_validates() {
    let mut b = board(100);
    for i in 1..=5 {
        b.insert(k(i * 10), i as u32, 1).unwrap();
    }
    let page: Vec<u64> = b.look_up(0, 2, 2).unwrap().iter().map(|e| e.0 .0).collect();
    assert_eq!(page, vec![40, 30]);
    assert_eq!(b.look_up(1, 6, 3).unwrap(), vec![]);
    assert_eq!(b.look_up(1, 0, 3), Err(BoardError::InvalidParameter));
    assert_eq!(b.look_up(1, 1, 0), Err(BoardError::InvalidParameter));
    assert_eq!(b.look_up(4, 1, 1), Err(BoardError::InvalidParameter));
}
