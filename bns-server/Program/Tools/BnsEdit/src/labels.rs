//! Friendly names for the simple view. Fields not listed here show only for experts.

/// Code name, friendly name. The order is the order on screen.
const FRIENDLY: &[(&str, &str)] = &[
    ("name2", "Name"),
    ("name", "Name"),
    ("text", "Text"),
    ("title", "Title"),
    ("level", "Level"),
    ("item-grade", "Rarity"),
    ("equip-level", "Level needed to use"),
    ("price", "Sell price"),
    ("buy-price", "Buy price"),
    ("item", "Item"),
    ("stack-count", "How many fit in one stack"),
    ("gear-score", "Gear score"),
    ("attack-power-equip-min", "Attack (lowest)"),
    ("attack-power-equip-max", "Attack (highest)"),
    ("defend-power-equip-value", "Defense"),
    ("max-hp-equip", "Extra health"),
    ("max-durability", "Durability"),
    ("cannot-sell", "Cannot sell"),
    ("cannot-trade", "Cannot trade"),
    ("cannot-dispose", "Cannot throw away"),
    ("cannot-depot", "Cannot put in storage"),
    ("auctionable", "Can sell at the market"),
    ("equip-sex", "Gender"),
    ("equip-race", "Race"),
    ("equip-type", "Where it is worn"),
    ("weapon-type", "Weapon type"),
    ("usable-duration", "Can be used for (time)"),
    ("description2", "Description"),
    ("icon", "Picture"),
    ("store2", "Shop"),
    ("store", "Shop"),
    ("boss-npc", "Is a boss"),
    ("offensive", "Attacks first"),
    ("scale", "Size"),
    ("walk-speed", "Walk speed"),
    ("run-speed", "Run speed"),
    ("skill-attack-power-min", "Damage (lowest)"),
    ("skill-attack-power-max", "Damage (highest)"),
    ("cast-duration", "Cast time"),
    ("recycle-group-duration", "Wait before next use"),
    ("consume-sp-value", "Focus cost"),
    ("consume-hp-value", "Health cost"),
    ("gather-range", "Range"),
];

pub fn friendly(name: &str) -> Option<&'static str> {
    FRIENDLY.iter().find(|(n, _)| *n == name).map(|(_, f)| *f)
}

pub fn order(name: &str) -> usize {
    FRIENDLY
        .iter()
        .position(|(n, _)| *n == name)
        .unwrap_or(usize::MAX)
}

/// Search filters: label, kind word, table names to look for (first found wins).
pub const KINDS: &[(&str, &str, &[&str])] = &[
    ("Items", "item", &["item"]),
    ("NPCs", "npc", &["npc"]),
    ("Skills", "skill", &["skill3", "skill"]),
    ("Text", "text", &["text"]),
];

/// "Item", "NPC", ... or the table name.
pub fn kind_of(table: &str) -> String {
    match table {
        "item" => "Item".into(),
        "npc" => "NPC".into(),
        "skill3" | "skill" => "Skill".into(),
        "text" => "Text".into(),
        "store2" => "Shop".into(),
        t => t.into(),
    }
}
