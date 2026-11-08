//! RankingBoardManager score paths (spec README.md section 6c).
//!
//! One `Board` per season contents type 1..=15. The node value is the
//! player key (playerType, playerId). Native stores a `Player*`.

use crate::board::{Board, BoardInit, Key, Ranks};
use crate::service::{max_ranking_list_index, pc_job_to_board_index};

/// Player key: (playerType, playerId).
pub type PlayerKey = (i8, u64);

/// The empty score: `s_initializedCount` key at 0x1400E3D50..0x1400E3D68,
/// uninitialized .data with no write xrefs, so all zero.
pub const EMPTY_KEY: Key = (0, 0, 0);

/// Season types whose primary score keeps only the high dword in key1
/// (InsertScoreForRatingScoreType @0x140019850: 1..=4, 7, 10..=12).
pub fn masks_low_dword(season_contents_type: i16) -> bool {
    matches!(season_contents_type, 1..=4 | 7 | 10..=12)
}

/// key1 = primary (high dword only for rating types), key2 = requestTime,
/// key3 = logId.
pub fn score_key(season_contents_type: i16, primary: u64, request_time: i64, log_id: u64) -> Key {
    let key1 = if masks_low_dword(season_contents_type) {
        primary & 0xFFFF_FFFF_0000_0000
    } else {
        primary
    };
    (key1, request_time as u64, log_id)
}

/// Board index of a player: PC → job table; type 2 → job 1/2 (else 0,
/// which Insert rejects); type 3 (party) → 1.
pub fn board_index(player_type: i16, job: i16) -> u8 {
    match player_type {
        1 => pc_job_to_board_index(job),
        2 => match job {
            1 | 2 => job as u8,
            _ => 0,
        },
        3 => 1,
        _ => 0,
    }
}

/// Insert input (native arguments).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoreInsert {
    pub season_contents_type: i16,
    pub player_type: i16,
    pub player_id: u64,
    pub job: i16,
    pub primary: u64,
    pub secondary: u64,
    pub request_time: i64,
    pub log_id: u64,
}

/// Success output: the stored key (`rankingScore`) and both ranks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoreResult {
    pub key: Key,
    pub ranks: Ranks,
}

#[derive(Debug, Clone, Default)]
pub struct RankingBoards {
    boards: Vec<Option<Board<PlayerKey>>>,
}

impl RankingBoards {
    /// InitializeRankingBoard @0x1400196F0 for each type 1..=15. A type
    /// whose Init fails keeps no board (null manager).
    pub fn new(max_level: i32, prob: i32, max_node_count: i32) -> Self {
        let boards = (1..=15u8)
            .map(|t| {
                Board::new(BoardInit {
                    max_level,
                    prob,
                    max_node_count,
                    max_index: max_ranking_list_index(t),
                })
                .ok()
            })
            .collect();
        Self { boards }
    }

    fn slot(&mut self, season_contents_type: i16) -> Option<&mut Board<PlayerKey>> {
        if !(1..=15).contains(&season_contents_type) {
            return None;
        }
        self.boards
            .get_mut(season_contents_type as usize - 1)?
            .as_mut()
    }

    pub fn board(&self, season_contents_type: i16) -> Option<&Board<PlayerKey>> {
        if !(1..=15).contains(&season_contents_type) {
            return None;
        }
        self.boards.get(season_contents_type as usize - 1)?.as_ref()
    }

    /// InsertScoreForRatingScoreType @0x140019850. `None` = native false.
    pub fn insert_score(&mut self, s: &ScoreInsert) -> Option<ScoreResult> {
        if !(1..=15).contains(&s.season_contents_type)
            || !(1..=3).contains(&s.player_type)
            || s.player_id == 0
            || s.primary == 0
            || s.request_time == 0
        {
            return None;
        }
        let key = score_key(s.season_contents_type, s.primary, s.request_time, s.log_id);
        let index = board_index(s.player_type, s.job);
        let value = (s.player_type as i8, s.player_id);
        let ranks = self
            .slot(s.season_contents_type)?
            .insert(key, value, index)
            .ok()?;
        Some(ScoreResult { key, ranks })
    }

    /// UpdateScoreForRatingScoreType @0x140019E20. The player type and job
    /// come from the stored player. The empty old key and an unchanged
    /// empty new key fail.
    #[allow(clippy::too_many_arguments)]
    pub fn update_score(
        &mut self,
        player: PlayerKey,
        job: i16,
        season_contents_type: i16,
        old: Key,
        primary: u64,
        request_time: i64,
        log_id: u64,
    ) -> Option<ScoreResult> {
        if !(1..=15).contains(&season_contents_type)
            || old == EMPTY_KEY
            || primary == 0
            || request_time == 0
            || log_id == 0
        {
            return None;
        }
        let key = score_key(season_contents_type, primary, request_time, log_id);
        if key == EMPTY_KEY || !(1..=3).contains(&player.0) {
            return None;
        }
        let index = board_index(player.0 as i16, job);
        let ranks = self
            .slot(season_contents_type)?
            .update(old, key, player, index)
            .ok()?;
        Some(ScoreResult { key, ranks })
    }

    /// DeleteScore @0x14001BC20. True when Remove succeeds; the caller
    /// then resets the player score to `EMPTY_KEY` with time 0.
    pub fn delete_score(
        &mut self,
        player: PlayerKey,
        job: i16,
        season_contents_type: i16,
        score: Key,
    ) -> bool {
        if score == EMPTY_KEY || !(1..=3).contains(&player.0) {
            return false;
        }
        let index = board_index(player.0 as i16, job);
        self.slot(season_contents_type)
            .is_some_and(|b| b.remove(score, index).is_ok())
    }
}

/// One reward tier of the season reward group record
/// (`_loaderDefs[2]` row: min ranks at +16, max ranks at +96, 20 slots,
/// tier count byte at +336).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RewardTier {
    pub min_rank: i32,
    pub max_rank: i32,
}

/// `RankingWithTopRanker` inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TopRankerQuery {
    pub season_contents_type: i8,
    pub schedule_number: i32,
    pub player_type: i8,
    pub player_id: u64,
    pub job: i8,
    pub score: Key,
}

/// The ranker that marks the next better reward tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TopRanker {
    pub ranking: i32,
    pub player: PlayerKey,
    pub score: Key,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RankingWithTopRanker {
    pub ranks: Ranks,
    pub top: Option<TopRanker>,
}

/// Types 8 and 13..=15 rank over list 0; the others use the job rank.
pub fn ranks_over_all(season_contents_type: i8) -> bool {
    matches!(season_contents_type, 8 | 13..=15)
}

impl RankingBoards {
    /// GetRankingWithTopRanker @0x14001A750. `tiers` is the reward group
    /// of the current season (None when the reward id is 0 or the row is
    /// missing: native logs and returns true with ranks only). `None` =
    /// native false.
    pub fn ranking_with_top_ranker(
        &self,
        q: &TopRankerQuery,
        tiers: Option<&[RewardTier]>,
    ) -> Option<RankingWithTopRanker> {
        if !(1..=15).contains(&q.season_contents_type)
            || q.schedule_number <= 0
            || !(1..=3).contains(&q.player_type)
            || q.player_id == 0
            || !(0..=14).contains(&q.job)
            || q.score == EMPTY_KEY
        {
            return None;
        }
        let index = board_index(q.player_type as i16, q.job as i16);
        let board = self.board(q.season_contents_type as i16)?;
        let ranks = board.search(q.score, index).ok()?;
        let mut out = RankingWithTopRanker { ranks, top: None };
        // No job rank: done, even for the list-0 types.
        if ranks[1] == 0 {
            return Some(out);
        }
        let Some(tiers) = tiers else {
            return Some(out);
        };
        let all = ranks_over_all(q.season_contents_type);
        let rank = if all { ranks[0] } else { ranks[1] };
        let Some(tier) = tiers
            .iter()
            .find(|t| t.min_rank <= rank && t.max_rank >= rank)
        else {
            return Some(out);
        };
        let start = if tier.min_rank == 1 {
            1
        } else {
            tier.min_rank - 1
        };
        let list = if all { 0 } else { index };
        // LookUp failure or an empty result (null value) is native false.
        let (score, player) = board.look_up(list, start, 1).ok()?.into_iter().next()?;
        out.top = Some(TopRanker {
            ranking: start,
            player: *player,
            score,
        });
        Some(out)
    }
}
