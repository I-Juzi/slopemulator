//! Ranking board (spec README.md section 6, `SkipListExManager`).
//!
//! Native keeps one probabilistic skip list per list index. The levels only
//! change speed, not the visible order, so the port keeps each list as a
//! vector in rank order. List 0 holds every node; list `index` (1..=max)
//! holds the nodes of one job. A node lives in list 0 and in its own list.
//!
//! Order: a larger key ranks higher. Keys compare as (key1, key2, key3)
//! unsigned (`SkipListEx::InsertNode` @0x14002E7E0).

/// `skiplistex_key_s`.
pub type Key = (u64, u64, u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardError {
    /// SKIPLISTEXERRORCODE_INVALID_PARAMETER (3).
    InvalidParameter,
    /// SKIPLISTEXERRORCODE_OVERLAPPED_KEY: the key is already in the list.
    OverlappedKey,
}

/// `SkipListExInitValues`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardInit {
    pub max_level: i32,
    pub prob: i32,
    pub max_node_count: i32,
    pub max_index: u8,
}

#[derive(Debug, Clone)]
struct Node<V> {
    key: Key,
    value: V,
    index: u8,
}

/// `SkipListExManager`.
#[derive(Debug, Clone)]
pub struct Board<V> {
    max_index: u8,
    max_nodes: usize,
    lists: Vec<Vec<Node<V>>>,
}

/// Ranks in list 0 and in the job list (`SkipListExOutput::ranks`).
/// 0 means "not in that list".
pub type Ranks = [i32; 2];

impl<V: Clone + PartialEq> Board<V> {
    /// SkipListExManager::Init @0x14002FDB0.
    pub fn new(init: BoardInit) -> Result<Self, BoardError> {
        if !(0..16).contains(&init.max_level)
            || !(0..=100).contains(&init.prob)
            || init.max_node_count <= 0
            || !(1..=14).contains(&init.max_index)
        {
            return Err(BoardError::InvalidParameter);
        }
        Ok(Self {
            max_index: init.max_index,
            max_nodes: init.max_node_count as usize,
            lists: (0..=init.max_index).map(|_| Vec::new()).collect(),
        })
    }

    pub fn max_index(&self) -> u8 {
        self.max_index
    }

    pub fn len(&self, index: u8) -> usize {
        self.lists.get(index as usize).map_or(0, Vec::len)
    }

    pub fn is_empty(&self) -> bool {
        self.lists[0].is_empty()
    }

    fn check_index(&self, index: u8) -> Result<(), BoardError> {
        if index == 0 || index > self.max_index {
            Err(BoardError::InvalidParameter)
        } else {
            Ok(())
        }
    }

    /// SkipListEx::searchKey @0x14002F7E0: 1-based rank of an exact key.
    fn position(&self, list: usize, key: &Key) -> Result<usize, usize> {
        // Descending order: compare reversed.
        self.lists[list].binary_search_by(|n| key.cmp(&n.key))
    }

    /// SkipListExManager::insertNode @0x140030E70 for one list.
    fn insert_into(&mut self, list: usize, node: Node<V>) -> Result<i32, BoardError> {
        let full = self.lists[list].len() >= self.max_nodes;
        if full {
            let tail = &self.lists[list].last().expect("full list").key;
            if *tail > node.key {
                // Below the last ranker: not stored, rank 0.
                return Ok(0);
            }
            if *tail < node.key {
                self.lists[list].pop();
            }
            // Equal to the tail: falls through to the duplicate error.
        }
        match self.position(list, &node.key) {
            Ok(_) => Err(BoardError::OverlappedKey),
            Err(at) => {
                self.lists[list].insert(at, node);
                Ok(at as i32 + 1)
            }
        }
    }

    /// completeRemoveNode @0x140031300: out of list 0 and out of its own list.
    fn remove_everywhere(&mut self, key: &Key, index: u8) {
        for list in [0, index as usize] {
            if let Ok(at) = self.position(list, key) {
                self.lists[list].remove(at);
            }
        }
    }

    fn place(&mut self, key: Key, value: V, index: u8) -> Result<Ranks, BoardError> {
        let node = Node { key, value, index };
        let all = self.insert_into(0, node.clone())?;
        match self.insert_into(index as usize, node) {
            Ok(own) => Ok([all, own]),
            Err(e) => {
                // Insert failure rolls back the list 0 entry.
                if all > 0 {
                    self.remove_everywhere(&key, 0);
                }
                Err(e)
            }
        }
    }

    /// SkipListExManager::Insert @0x140030050.
    pub fn insert(&mut self, key: Key, value: V, index: u8) -> Result<Ranks, BoardError> {
        self.check_index(index)?;
        self.place(key, value, index)
    }

    /// SkipListExManager::completeSearchNode @0x140031170: the job list
    /// first, then list 0.
    fn find(&self, key: &Key, index: u8) -> Option<&Node<V>> {
        [index as usize, 0]
            .into_iter()
            .find_map(|l| self.position(l, key).ok().map(|at| &self.lists[l][at]))
    }

    /// SkipListExManager::Remove @0x1400302D0. A missing key is not an error.
    pub fn remove(&mut self, key: Key, index: u8) -> Result<(), BoardError> {
        self.check_index(index)?;
        if let Some(n) = self.find(&key, index) {
            let own = n.index;
            self.remove_everywhere(&key, own);
        }
        Ok(())
    }

    /// SkipListExManager::Update @0x140030410: take the old node out (or
    /// make a new one), give it the new key and insert it again. The node
    /// keeps its value. A failed insert leaves the node out of both lists.
    pub fn update(&mut self, old: Key, new: Key, value: V, index: u8) -> Result<Ranks, BoardError> {
        self.check_index(index)?;
        let value = match self.find(&old, index) {
            Some(n) => {
                let (v, own) = (n.value.clone(), n.index);
                self.remove_everywhere(&old, own);
                v
            }
            None => value,
        };
        let all = self.insert_into(
            0,
            Node {
                key: new,
                value: value.clone(),
                index,
            },
        )?;
        let own = self.insert_into(
            index as usize,
            Node {
                key: new,
                value,
                index,
            },
        )?;
        Ok([all, own])
    }

    /// SkipListExManager::Search @0x140030790: ranks of a key (0 = absent).
    pub fn search(&self, key: Key, index: u8) -> Result<Ranks, BoardError> {
        self.check_index(index)?;
        let rank = |l: usize| self.position(l, &key).map_or(0, |at| at as i32 + 1);
        Ok([rank(0), rank(index as usize)])
    }

    /// SkipListExManager::LookUp @0x140030A10: up to `count` entries from
    /// `start_rank` in list `index` (0 allowed). A start past the end is
    /// OK with no entries.
    pub fn look_up(
        &self,
        index: u8,
        start_rank: i32,
        count: i32,
    ) -> Result<Vec<(Key, &V)>, BoardError> {
        if index > self.max_index || start_rank <= 0 || count <= 0 {
            return Err(BoardError::InvalidParameter);
        }
        Ok(self.lists[index as usize]
            .iter()
            .skip(start_rank as usize - 1)
            .take(count as usize)
            .map(|n| (n.key, &n.value))
            .collect())
    }
}
