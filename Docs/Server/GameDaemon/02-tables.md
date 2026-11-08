# GameDaemon table survey: 2020 data

Source: decoded LMXBOSLB bytes from the local 2020 GameDaemon data folder. No executable or external code was used. Counts describe the 2026-11-08 snapshot. Records are direct element children of `table`. Attribute paths include nested elements.

Types are lexical inferences, not a server schema. `bool` means literal true/false. Numeric 0/1 stays int. Enum-like sets contain at most 20 lowercase/hyphen names; they can still be aliases. ❓ Reference targets and optional/default semantics require further checks.

XML paths: 21065. Binary table types: 252. Nonbinary files skipped: 1. Other binary root names/counts: {"combat-sequence-root": 2}.

## Table counts and root attribute names

| Table type | Files | Records | Root attribute names |
| --- | ---: | ---: | --- |
| ArenaMatchingRule | 1 | 360 | release-module, release-side, type, version |
| ArenaPortal | 1 | 8 | release-module, release-side, type, version |
| BadgeSynthesisRecipe | 1 | 1536 | release-module, release-side, type, version |
| BattleRoyalFieldEffectPouchMesh | 1 | 0 | release-module, release-side, type, version |
| BossGroundAttackTargetPattern | 1 | 182 | release-module, release-side, type, version |
| BossNpc | 1 | 2822 | release-module, release-side, type, version |
| BotScoringCondition | 1 | 8 | release-module, release-side, type, version |
| BotScoringDestination | 1 | 993 | release-module, release-side, type, version |
| BotScoringLevel | 1 | 5 | release-module, release-side, type, version |
| ChallengeList | 1 | 17 | release-module, release-side, type, version |
| ChallengeListReward | 1 | 61 | release-module, release-side, type, version |
| ContributionReward | 1 | 428 | release-module, release-side, type, version |
| CustomizingDetailIcon | 1 | 2819 | release-module, release-side, type, version |
| CustomizingDetailSlider | 1 | 530 | release-module, release-side, type, version |
| DuelMatchingRule | 1 | 100 | release-module, release-side, type, version |
| DuelMemberDisplaySkill | 1 | 63 | release-module, release-side, type, version |
| FielditemDrop | 3 | 1000 | release-module, release-side, type, version |
| Fish | 1 | 103 | release-module, release-side, type, version |
| Fishing-Paste | 1 | 7 | release-module, release-side, type, version |
| GradeBenefits | 1 | 130 | release-module, release-side, type, version |
| GuildUniformComponent | 3 | 416 | release-module, release-side, type, version |
| GuildUniformParamTypeCategory | 1 | 6 | release-module, release-side, type, version |
| GuildUniformRecipe | 1 | 1071 | release-module, release-side, type, version |
| GuildUniformRegistrationQuota | 1 | 3 | release-module, release-side, type, version |
| GuildUniformSlider | 1 | 7 | release-module, release-side, type, version |
| InterDungeonList | 1 | 58 | release-module, release-side, type, version |
| ItemExchange | 1 | 1296 | release-module, release-side, type, version |
| ItemGrowth | 1 | 4120 | release-module, release-side, type, version |
| ItemSkill | 24 | 10782 | release-module, release-side, type, version |
| ItemSpirit | 1 | 58 | release-module, release-side, type, version |
| ItemTransformRecipe | 4 | 35027 | release-module, release-side, type, version |
| ItemUsableRelation | 1 | 74 | release-module, release-side, type, version |
| JumpingCharacter | 1 | 13 | release-module, release-side, type, version |
| JumpingCharacter2 | 1 | 1 | release-module, release-side, type, version |
| JumpingQuest | 1 | 2520 | release-module, release-side, type, version |
| JumpingQuest2 | 1 | 1008 | release-module, release-side, type, version |
| JumpingTeleport | 1 | 1254 | release-module, release-side, type, version |
| JumpingTeleport2 | 1 | 132 | release-module, release-side, type, version |
| Mentoring | 1 | 1 | release-module, release-side, type, version |
| NpcConvoy | 1 | 51 | release-module, release-side, type, version |
| NpcSquadCombatPattern | 1 | 14 | release-module, release-side, type, version |
| PcCafeBenefits | 1 | 12 | release-module, release-side, type, version |
| PcInitialSetting | 1 | 93 | release-module, release-side, type, version |
| PcSkill3 | 1 | 1258 | release-module, release-side, type, version |
| QuestReward | 1 | 2339 | release-module, release-side, type, version |
| QuestRewardSkill3 | 1 | 235 | release-module, release-side, type, version |
| RandomStore | 1 | 2 | release-module, release-side, type, version |
| RandomStoreDrawReward | 1 | 5 | release-module, release-side, type, version |
| RankingPvEStage | 1 | 2 | release-module, release-side, type, version |
| RankingReward | 1 | 1212 | release-module, release-side, type, version |
| RankingRewardGroup | 1 | 72 | release-module, release-side, type, version |
| SkillCastCondition3 | 81 | 12830 | release-module, release-side, type, version |
| SkillDashAttribute3 | 1 | 37 | release-module, release-side, type, version |
| SkillGatherRange3 | 4 | 3783 | release-module, release-side, type, version |
| SkillModifyLimit | 1 | 14 | release-module, release-side, type, version |
| SkillResultControll3 | 1 | 57 | release-module, release-side, type, version |
| SkillSkin | 1 | 3 | release-module, release-side, type, version |
| SkillTargetFilter3 | 1 | 138 | release-module, release-side, type, version |
| SlateScroll | 1 | 18 | release-module, release-side, type, version |
| SlateScrollStone | 1 | 1395 | release-module, release-side, type, version |
| SlateStone | 1 | 405 | release-module, release-side, type, version |
| SoulMask | 1 | 25 | release-module, release-side, type, version |
| SoulMaskSet | 1 | 32 | release-module, release-side, type, version |
| SummonedBeautyShop | 1 | 5 | release-module, release-side, type, version |
| SummonedDesignerPreset | 1 | 5 | release-module, release-side, type, version |
| SummonedMasteryLevel | 2 | 560 | release-module, release-side, type, version |
| SummonedPreset | 1 | 78 | release-module, release-side, type, version |
| Terrain | 164 | 164 | release-module, release-side, type, version |
| Title | 1 | 0 | release-module, release-side, type, version |
| TrainSkillVersion | 1 | 14 | release-module, release-side, type, version |
| WeaponGemSlot | 1 | 5 | release-module, release-side, type, version |
| WeaponGemTransformResult | 1 | 325 | release-module, release-side, type, version |
| WeeklyTimeTable | 1 | 55 | release-module, release-side, type, version |
| ZoneArea | 1533 | 76936 | release-module, release-side, type, version |
| ZoneAreaGroup | 1 | 160 | release-module, release-side, type, version |
| ZoneBaseCamp | 1533 | 68 | release-module, release-side, type, version |
| ZoneCampfire | 1533 | 466 | release-module, release-side, type, version |
| ZoneCellAttribute | 1 | 4 | release-module, release-side, type, version |
| ZoneConvoy | 1 | 39 | release-module, release-side, type, version |
| ZoneEnv | 4 | 98 | release-module, release-side, type, version |
| ZoneEnv2 | 1 | 11626 | release-module, release-side, type, version |
| ZoneEnv2Place | 1533 | 11682 | release-module, release-side, type, version |
| ZoneEnv2Spawn | 1 | 11832 | release-module, release-side, type, version |
| ZoneGatherSource | 1533 | 0 | release-module, release-side, type, version |
| ZoneNpcRandomSpawn | 1 | 159 | release-module, release-side, type, version |
| ZoneNpcSpawn | 77 | 68455 | release-module, release-side, type, version |
| ZoneNpcSpawnEvent | 1 | 1 | release-module, release-side, type, version |
| ZoneNpcSquad | 75 | 10795 | release-module, release-side, type, version |
| ZonePathway | 1533 | 77786 | release-module, release-side, type, version |
| ZonePcSpawn | 1533 | 2951 | release-module, release-side, type, version |
| ZoneRespawn | 1533 | 1652 | release-module, release-side, type, version |
| ZoneRule | 1 | 17 | release-module, release-side, type, version |
| ZoneTeleportPosition | 1533 | 156 | release-module, release-side, type, version |
| ZoneTeleportSwitch | 1533 | 190 | release-module, release-side, type, version |
| ZoneTriggerEventCond | 1 | 29 | release-module, release-side, type, version |
| ZoneTriggerEventStage | 1 | 16 | release-module, release-side, type, version |
| account-post-charge | 1 | 2 | release-module, release-side, type, version |
| achievement | 1 | 1558 | release-module, release-side, type, version |
| achievement-register | 1 | 1067 | release-module, release-side, type, version |
| act-sequence | 115 | 9091 | release-module, release-side, type, version |
| attraction-group | 1 | 74 | release-module, release-side, type, version |
| attraction-reward | 1 | 80 | release-module, release-side, type, version |
| badge-set-item | 1 | 384 | release-module, release-side, type, version |
| badge-synthesis-cost | 1 | 512 | release-module, release-side, type, version |
| battle-field-grade | 1 | 15 | release-module, release-side, type, version |
| battle-royal-field | 1 | 1 | release-module, release-side, type, version |
| battle-royal-field-effect-pouch | 1 | 25 | release-module, release-side, type, version |
| battle-royal-field-event | 1 | 1 | release-module, release-side, type, version |
| battle-royal-field-item-spawn-position | 1 | 1400 | release-module, release-side, type, version |
| battle-royal-field-score-conversion | 1 | 6 | release-module, release-side, type, version |
| board-gacha | 1 | 1 | release-module, release-side, type, version |
| board-gacha-reward | 1 | 18 | release-module, release-side, type, version |
| boast-event | 1 | 57 | release-module, release-side, type, version |
| boss-challenge | 1 | 1 | release-module, release-side, type, version |
| boss-challenge-round | 1 | 6 | release-module, release-side, type, version |
| campfire | 1 | 103 | release-module, release-side, type, version |
| cave | 1 | 21 | release-module, release-side, type, version |
| cave2 | 1 | 710 | release-module, release-side, type, version |
| challenge-party | 1 | 1 | release-module, release-side, type, version |
| cinematic | 1 | 321 | release-module, release-side, type, version |
| classic-field-zone | 1 | 12 | release-module, release-side, type, version |
| closet-collecting-grade | 1 | 7 | release-module, release-side, type, version |
| closet-group | 1 | 1856 | release-module, release-side, type, version |
| collecting | 1 | 177 | release-module, release-side, type, version |
| combat-sequence | 122 | 11533 | release-module, release-side, type, version |
| content-quota | 1 | 51 | release-module, release-side, type, version |
| contents-escape-penalty | 1 | 4 | release-module, release-side, type, version |
| contents-reset | 1 | 21 | release-module, release-side, type, version |
| craft-case-recipe | 1 | 711 | release-module, release-side, type, version |
| craft-group-recipe | 1 | 168 | release-module, release-side, type, version |
| craft-recipe | 1 | 1299 | release-module, release-side, type, version |
| craft-recipe-step | 1 | 1486 | release-module, release-side, type, version |
| custom-dress-design-cost | 1 | 866 | release-module, release-side, type, version |
| custom-dress-design-type-info | 1 | 13 | release-module, release-side, type, version |
| default-rating-score-group | 1 | 17 | release-module, release-side, type, version |
| difficulty-type | 1 | 3 | release-module, release-side, type, version |
| difficulty-type-modify | 1 | 102 | release-module, release-side, type, version |
| duel | 1 | 36 | release-module, release-side, type, version |
| duel-bot | 1 | 315 | release-module, release-side, type, version |
| duel-bot-ability | 1 | 285 | release-module, release-side, type, version |
| duel-bot-challenge | 1 | 1 | release-module, release-side, type, version |
| duel-bot-challenge-floor | 1 | 100 | release-module, release-side, type, version |
| duel-bot-challenge-strategic-tool | 1 | 20 | release-module, release-side, type, version |
| duel-bot-equipped-item | 1 | 280 | release-module, release-side, type, version |
| duel-bot-group | 1 | 29 | release-module, release-side, type, version |
| duel-bot-trained-skill | 1 | 9 | release-module, release-side, type, version |
| duel-bot-training-room | 1 | 1 | release-module, release-side, type, version |
| duel-bot-training-room-reward | 1 | 270 | release-module, release-side, type, version |
| duel-bot-training-room-version | 1 | 1 | release-module, release-side, type, version |
| duel-grade | 1 | 5 | release-module, release-side, type, version |
| duel-zone | 1 | 1 | release-module, release-side, type, version |
| dungeon | 1 | 51 | release-module, release-side, type, version |
| effect | 119 | 86147 | release-module, release-side, type, version |
| effect-group | 116 | 39442 | release-module, release-side, type, version |
| energy-point | 1 | 1 | release-module, release-side, type, version |
| energy-point-reward | 1 | 5 | release-module, release-side, type, version |
| equip-gem-piece | 1 | 8013 | release-module, release-side, type, version |
| event-skill | 1 | 15 | release-module, release-side, type, version |
| expand-inventory | 1 | 45 | release-module, release-side, type, version |
| expand-random-store | 1 | 6 | release-module, release-side, type, version |
| expand-skill-page | 1 | 5 | release-module, release-side, type, version |
| faction | 1 | 14 | release-module, release-side, type, version |
| faction-battle-field-zone | 1 | 1 | release-module, release-side, type, version |
| faction-level | 1 | 16 | release-module, release-side, type, version |
| field-zone | 1 | 5 | release-module, release-side, type, version |
| fielditem | 1 | 832 | release-module, release-side, type, version |
| filter | 1 | 920 | release-module, release-side, type, version |
| fishing-field | 1 | 13 | release-module, release-side, type, version |
| friend-noti-location | 1 | 331 | release-module, release-side, type, version |
| guild-action | 1 | 19 | release-module, release-side, type, version |
| guild-bank-item | 1 | 5 | release-module, release-side, type, version |
| guild-combat | 1 | 1 | release-module, release-side, type, version |
| guild-craft-recipe | 1 | 105 | release-module, release-side, type, version |
| guild-discount | 1 | 6 | release-module, release-side, type, version |
| guild-member-grade-authority | 1 | 36 | release-module, release-side, type, version |
| guildlevel | 1 | 16 | release-module, release-side, type, version |
| item | 14 | 144241 | release-module, release-side, type, version |
| item-brand | 1 | 3188 | release-module, release-side, type, version |
| item-brand-tooltip | 1 | 0 | release-module, release-side, type, version |
| item-buy-price | 1 | 3550 | release-module, release-side, type, version |
| item-event | 1 | 221 | release-module, release-side, type, version |
| item-random-ability-section | 1 | 1877 | release-module, release-side, type, version |
| item-random-ability-slot | 1 | 5649 | release-module, release-side, type, version |
| item-transform-retry-cost | 1 | 1 | release-module, release-side, type, version |
| item-usable-group | 1 | 15 | release-module, release-side, type, version |
| jackpot-boss-zone | 1 | 1 | release-module, release-side, type, version |
| job | 1 | 14 | release-module, release-side, type, version |
| job-quest | 1 | 0 | release-module, release-side, type, version |
| job-style | 1 | 64 | release-module, release-side, type, version |
| level | 1 | 60 | release-module, release-side, type, version |
| market-register-amount-tax-rate | 1 | 10 | release-module, release-side, type, version |
| mastery-ability | 1 | 1200 | release-module, release-side, type, version |
| mastery-level | 1 | 50 | release-module, release-side, type, version |
| mastery-stat-point | 1 | 500 | release-module, release-side, type, version |
| mastery-stat-point-pick | 1 | 390 | release-module, release-side, type, version |
| mysterious-store | 1 | 2 | release-module, release-side, type, version |
| mysterious-store-item-group | 1 | 28 | release-module, release-side, type, version |
| newbie-care | 1 | 6 | release-module, release-side, type, version |
| npc | 18 | 34434 | release-module, release-side, type, version |
| npc-brain-parameters | 18 | 34446 | release-module, release-side, type, version |
| npc-reset-stage | 1 | 227 | release-module, release-side, type, version |
| party-battle-field-zone | 1 | 3 | release-module, release-side, type, version |
| partymatch | 1 | 93 | release-module, release-side, type, version |
| pc | 5 | 10200 | release-module, release-side, type, version |
| pc-race-sex-job | 1 | 46 | release-module, release-side, type, version |
| pet-food-recovery | 1 | 3 | release-module, release-side, type, version |
| public-raid | 1 | 7 | release-module, release-side, type, version |
| public-raid-event | 1 | 2 | release-module, release-side, type, version |
| quest | 2610 | 2610 | release-module, release-side, type, version |
| raid-dungeon | 1 | 15 | release-module, release-side, type, version |
| random-distribution | 1 | 8 | release-module, release-side, type, version |
| random-store-item | 1 | 976 | release-module, release-side, type, version |
| random-store-item-group | 1 | 122 | release-module, release-side, type, version |
| random-store-prob-group | 1 | 1 | release-module, release-side, type, version |
| reward | 10 | 123409 | release-module, release-side, type, version |
| script | 162 | 17266 | release-module, release-side, type, version |
| season-contents-schedule | 1 | 3043 | release-module, release-side, type, version |
| set-item | 1 | 372 | release-module, release-side, type, version |
| simple-field | 1 | 7 | release-module, release-side, type, version |
| skill-build-up | 12 | 272 | release-module, release-side, type, version |
| skill-by-equipment | 1 | 205 | release-module, release-side, type, version |
| skill-caster-function-control | 1 | 3 | release-module, release-side, type, version |
| skill-combo | 1 | 133 | release-module, release-side, type, version |
| skill-inheritance | 1 | 2674 | release-module, release-side, type, version |
| skill-level-modify | 2 | 603 | release-module, release-side, type, version |
| skill-mastery-level-modify | 2 | 603 | release-module, release-side, type, version |
| skill-modify-info | 14 | 13552 | release-module, release-side, type, version |
| skill-modify-info-group | 2 | 6980 | release-module, release-side, type, version |
| skill-train-category | 12 | 1226 | release-module, release-side, type, version |
| skill-training-room | 1 | 1 | release-module, release-side, type, version |
| skill-training-room-group | 1 | 96 | release-module, release-side, type, version |
| skill-training-room-subject | 1 | 552 | release-module, release-side, type, version |
| skill-trait | 12 | 504 | release-module, release-side, type, version |
| skill3 | 91 | 20791 | release-module, release-side, type, version |
| skillAttributeRule | 1 | 51 | release-module, release-side, type, version |
| skillSystematization | 1 | 155 | release-module, release-side, type, version |
| social | 88 | 35266 | release-module, release-side, type, version |
| soul-npc-skill | 1 | 16 | release-module, release-side, type, version |
| special-skill-gauge | 1 | 12 | release-module, release-side, type, version |
| stance | 1 | 45 | release-module, release-side, type, version |
| store | 1 | 151 | release-module, release-side, type, version |
| store-by-item | 1 | 46 | release-module, release-side, type, version |
| store2 | 1 | 408 | release-module, release-side, type, version |
| summoned | 1 | 8 | release-module, release-side, type, version |
| summonedLevel | 5 | 1928 | release-module, release-side, type, version |
| survey | 1 | 0 | release-module, release-side, type, version |
| teleport | 1 | 151 | release-module, release-side, type, version |
| tendency-field | 1 | 2 | release-module, release-side, type, version |
| unlocated-store | 1 | 7 | release-module, release-side, type, version |
| weapon-appearance-change-cost | 1 | 1287 | release-module, release-side, type, version |
| zone | 1533 | 1533 | release-module, release-side, type, version |
| zone-pc-random-spawn | 1 | 1 | release-module, release-side, type, version |

## Root metadata sets

These are enum-like names or numeric metadata counts. Version entries show the number of distinct versions, not their values.

| Table | release-module names | release-side names | Distinct versions |
| --- | --- | --- | ---: |
| ArenaMatchingRule | ItemData | server | 1 |
| ArenaPortal | ItemData | server | 1 |
| BadgeSynthesisRecipe | ItemData | server | 1 |
| BattleRoyalFieldEffectPouchMesh | ZoneData0 | server | 1 |
| BossGroundAttackTargetPattern | CombatData | server | 1 |
| BossNpc | AIData | server | 1 |
| BotScoringCondition | LocalizationData | server | 1 |
| BotScoringDestination | LocalizationData | server | 1 |
| BotScoringLevel | LocalizationData | server | 1 |
| ChallengeList | ItemData | server | 1 |
| ChallengeListReward | ItemData | server | 1 |
| ContributionReward | ItemData | server | 1 |
| CustomizingDetailIcon | ItemData | server | 1 |
| CustomizingDetailSlider | ItemData | server | 1 |
| DuelMatchingRule | ItemData | server | 1 |
| DuelMemberDisplaySkill | CombatData | server | 1 |
| FielditemDrop | AIData | server | 1 |
| Fish | ItemData | server | 1 |
| Fishing-Paste | ItemData | server | 1 |
| GradeBenefits | LocalizationData | server | 1 |
| GuildUniformComponent | ItemData | server | 1 |
| GuildUniformParamTypeCategory | ItemData | server | 1 |
| GuildUniformRecipe | ItemData | server | 1 |
| GuildUniformRegistrationQuota | ItemData | server | 1 |
| GuildUniformSlider | ItemData | server | 1 |
| InterDungeonList | ZoneData0 | server | 1 |
| ItemExchange | ItemData | server | 1 |
| ItemGrowth | ItemData | server | 1 |
| ItemSkill | CombatData | server | 1 |
| ItemSpirit | ItemData | server | 1 |
| ItemTransformRecipe | ItemData | server | 1 |
| ItemUsableRelation | ItemData | server | 1 |
| JumpingCharacter | CombatData | server | 1 |
| JumpingCharacter2 | ItemData | server | 1 |
| JumpingQuest | QuestData | server | 1 |
| JumpingQuest2 | QuestData | server | 1 |
| JumpingTeleport | QuestData | server | 1 |
| JumpingTeleport2 | QuestData | server | 1 |
| Mentoring | ItemData | server | 1 |
| NpcConvoy | AIData | server | 1 |
| NpcSquadCombatPattern | AIData | server | 1 |
| PcCafeBenefits | LocalizationData | server | 1 |
| PcInitialSetting | CombatData | server | 1 |
| PcSkill3 | LocalizationData | server | 1 |
| QuestReward | ItemData | server | 1 |
| QuestRewardSkill3 | ItemData | server | 1 |
| RandomStore | ItemData | server | 1 |
| RandomStoreDrawReward | ItemData | server | 1 |
| RankingPvEStage | LocalizationData | server | 1 |
| RankingReward | ItemData | server | 1 |
| RankingRewardGroup | ItemData | server | 1 |
| SkillCastCondition3 | CombatData, ItemData | server | 1 |
| SkillDashAttribute3 | CombatData | server | 1 |
| SkillGatherRange3 | CombatData, ItemData | server | 1 |
| SkillModifyLimit | CombatData | server | 1 |
| SkillResultControll3 | CombatData | server | 1 |
| SkillSkin | ItemData | server | 1 |
| SkillTargetFilter3 | CombatData | server | 1 |
| SlateScroll | ItemData | server | 1 |
| SlateScrollStone | ItemData | server | 1 |
| SlateStone | ItemData | server | 1 |
| SoulMask | CombatData | server | 1 |
| SoulMaskSet | CombatData | server | 1 |
| SummonedBeautyShop | ItemData | server | 1 |
| SummonedDesignerPreset | CombatData | server | 1 |
| SummonedMasteryLevel | LocalizationData | server | 1 |
| SummonedPreset | CombatData | server | 1 |
| Terrain | ZoneData1, ZoneData2 | server | 1 |
| Title | ItemData | server | 1 |
| TrainSkillVersion | CombatData | server | 1 |
| WeaponGemSlot | ItemData | server | 1 |
| WeaponGemTransformResult | ItemData | server | 1 |
| WeeklyTimeTable | LocalizationData | server | 1 |
| ZoneArea | ZoneData1, ZoneData2 | server | 1 |
| ZoneAreaGroup | ZoneData0 | server | 1 |
| ZoneBaseCamp | ZoneData1, ZoneData2 | server | 1 |
| ZoneCampfire | ZoneData1, ZoneData2 | server | 1 |
| ZoneCellAttribute | ZoneData0 | server | 1 |
| ZoneConvoy | AIData | server | 1 |
| ZoneEnv | ZoneData1, ZoneData2 | server | 1 |
| ZoneEnv2 | ZoneData0 | server | 1 |
| ZoneEnv2Place | ZoneData1, ZoneData2 | server | 1 |
| ZoneEnv2Spawn | ZoneData0 | server | 1 |
| ZoneGatherSource | ZoneData1, ZoneData2 | server | 1 |
| ZoneNpcRandomSpawn | ZoneData0 | server | 1 |
| ZoneNpcSpawn | ZoneData0 | server | 1 |
| ZoneNpcSpawnEvent | AIData | server | 1 |
| ZoneNpcSquad | ZoneData0 | server | 1 |
| ZonePathway | ZoneData1, ZoneData2 | server | 1 |
| ZonePcSpawn | ZoneData1, ZoneData2 | server | 1 |
| ZoneRespawn | ZoneData1, ZoneData2 | server | 1 |
| ZoneRule | ZoneData0 | server | 1 |
| ZoneTeleportPosition | ZoneData1, ZoneData2 | server | 1 |
| ZoneTeleportSwitch | ZoneData1, ZoneData2 | server | 1 |
| ZoneTriggerEventCond | AIData | server | 1 |
| ZoneTriggerEventStage | AIData | server | 1 |
| account-post-charge | ItemData | server | 1 |
| achievement | ItemData | server | 1 |
| achievement-register | ItemData | server | 1 |
| act-sequence | AIData | server | 1 |
| attraction-group | ZoneData0 | server | 1 |
| attraction-reward | ItemData | server | 1 |
| badge-set-item | ItemData | server | 1 |
| badge-synthesis-cost | ItemData | server | 1 |
| battle-field-grade | ItemData | server | 1 |
| battle-royal-field | ZoneData0 | server | 1 |
| battle-royal-field-effect-pouch | ZoneData0 | server | 1 |
| battle-royal-field-event | ZoneData0 | server | 1 |
| battle-royal-field-item-spawn-position | ZoneData0 | server | 1 |
| battle-royal-field-score-conversion | ItemData | server | 1 |
| board-gacha | ItemData | server | 1 |
| board-gacha-reward | ItemData | server | 1 |
| boast-event | ItemData | server | 1 |
| boss-challenge | AIData | server | 1 |
| boss-challenge-round | AIData | server | 1 |
| campfire | ZoneData0 | server | 1 |
| cave | ZoneData0 | server | 1 |
| cave2 | ZoneData0 | server | 1 |
| challenge-party | ZoneData0 | server | 1 |
| cinematic | QuestData | server | 1 |
| classic-field-zone | ZoneData0 | server | 1 |
| closet-collecting-grade | ItemData | server | 1 |
| closet-group | ItemData | server | 1 |
| collecting | ItemData | server | 1 |
| combat-sequence | AIData | server | 1 |
| content-quota | ItemData | server | 1 |
| contents-escape-penalty | ItemData | server | 1 |
| contents-reset | ItemData | server | 1 |
| craft-case-recipe | ItemData | server | 1 |
| craft-group-recipe | ItemData | server | 1 |
| craft-recipe | ItemData | server | 1 |
| craft-recipe-step | ItemData | server | 1 |
| custom-dress-design-cost | ItemData | server | 1 |
| custom-dress-design-type-info | ItemData | server | 1 |
| default-rating-score-group | ItemData | server | 1 |
| difficulty-type | AIData | server | 1 |
| difficulty-type-modify | AIData | server | 1 |
| duel | ItemData | server | 1 |
| duel-bot | CombatData | server | 1 |
| duel-bot-ability | CombatData | server | 1 |
| duel-bot-challenge | ZoneData0 | server | 1 |
| duel-bot-challenge-floor | ItemData | server | 1 |
| duel-bot-challenge-strategic-tool | CombatData | server | 1 |
| duel-bot-equipped-item | ItemData | server | 1 |
| duel-bot-group | ZoneData0 | server | 1 |
| duel-bot-trained-skill | CombatData | server | 1 |
| duel-bot-training-room | ZoneData0 | server | 1 |
| duel-bot-training-room-reward | ItemData | server | 1 |
| duel-bot-training-room-version | ItemData | server | 1 |
| duel-grade | ItemData | server | 1 |
| duel-zone | ItemData | server | 1 |
| dungeon | ZoneData0 | server | 1 |
| effect | CombatData, ItemData | server | 1 |
| effect-group | CombatData, ItemData | server | 1 |
| energy-point | ItemData | server | 1 |
| energy-point-reward | ItemData | server | 1 |
| equip-gem-piece | ItemData | server | 1 |
| event-skill | CombatData | server | 1 |
| expand-inventory | ItemData | server | 1 |
| expand-random-store | ItemData | server | 1 |
| expand-skill-page | ItemData | server | 1 |
| faction | ItemData | server | 1 |
| faction-battle-field-zone | ZoneData0 | server | 1 |
| faction-level | LocalizationData | server | 1 |
| field-zone | ZoneData0 | server | 1 |
| fielditem | AIData | server | 1 |
| filter | CombatData | server | 1 |
| fishing-field | ItemData | server | 1 |
| friend-noti-location | ZoneData0 | server | 1 |
| guild-action | ItemData | server | 1 |
| guild-bank-item | ItemData | server | 1 |
| guild-combat | ZoneData0 | server | 1 |
| guild-craft-recipe | ItemData | server | 1 |
| guild-discount | ItemData | server | 1 |
| guild-member-grade-authority | ItemData | server | 1 |
| guildlevel | ItemData | server | 1 |
| item | ItemData | server | 1 |
| item-brand | ItemData | server | 1 |
| item-brand-tooltip | ItemData | server | 1 |
| item-buy-price | ItemData | server | 1 |
| item-event | ItemData | server | 1 |
| item-random-ability-section | ItemData | server | 1 |
| item-random-ability-slot | ItemData | server | 1 |
| item-transform-retry-cost | ItemData | server | 1 |
| item-usable-group | ItemData | server | 1 |
| jackpot-boss-zone | AIData | server | 1 |
| job | CombatData | server | 1 |
| job-quest | QuestData | server | 1 |
| job-style | CombatData | server | 1 |
| level | LocalizationData | server | 1 |
| market-register-amount-tax-rate | ItemData | server | 1 |
| mastery-ability | LocalizationData | server | 1 |
| mastery-level | LocalizationData | server | 1 |
| mastery-stat-point | LocalizationData | server | 1 |
| mastery-stat-point-pick | LocalizationData | server | 1 |
| mysterious-store | ItemData | server | 1 |
| mysterious-store-item-group | ItemData | server | 1 |
| newbie-care | ItemData | server | 1 |
| npc | AIData | server | 1 |
| npc-brain-parameters | AIData | server | 1 |
| npc-reset-stage | AIData | server | 1 |
| party-battle-field-zone | ZoneData0 | server | 1 |
| partymatch | ZoneData0 | server | 1 |
| pc | LocalizationData | server | 1 |
| pc-race-sex-job | CombatData | server | 1 |
| pet-food-recovery | ItemData | server | 1 |
| public-raid | ZoneData0 | server | 1 |
| public-raid-event | ZoneData0 | server | 1 |
| quest | QuestData | server | 1 |
| raid-dungeon | ZoneData0 | server | 1 |
| random-distribution | ItemData | server | 1 |
| random-store-item | ItemData | server | 1 |
| random-store-item-group | ItemData | server | 1 |
| random-store-prob-group | ItemData | server | 1 |
| reward | ItemData | server | 1 |
| script | AIData | server | 1 |
| season-contents-schedule | ItemData | server | 1 |
| set-item | ItemData | server | 1 |
| simple-field | ZoneData0 | server | 1 |
| skill-build-up | CombatData | server | 1 |
| skill-by-equipment | CombatData | server | 1 |
| skill-caster-function-control | CombatData | server | 1 |
| skill-combo | CombatData | server | 1 |
| skill-inheritance | CombatData | server | 1 |
| skill-level-modify | CombatData | server | 1 |
| skill-mastery-level-modify | CombatData | server | 1 |
| skill-modify-info | CombatData | server | 1 |
| skill-modify-info-group | CombatData | server | 1 |
| skill-train-category | CombatData | server | 1 |
| skill-training-room | ZoneData0 | server | 1 |
| skill-training-room-group | ZoneData0 | server | 1 |
| skill-training-room-subject | ZoneData0 | server | 1 |
| skill-trait | CombatData | server | 1 |
| skill3 | CombatData, ItemData | server | 1 |
| skillAttributeRule | CombatData | server | 1 |
| skillSystematization | CombatData | server | 1 |
| social | QuestData | server | 1 |
| soul-npc-skill | AIData | server | 1 |
| special-skill-gauge | CombatData | server | 1 |
| stance | CombatData | server | 1 |
| store | ItemData | server | 1 |
| store-by-item | ItemData | server | 1 |
| store2 | ItemData | server | 1 |
| summoned | LocalizationData | server | 1 |
| summonedLevel | LocalizationData | server | 1 |
| survey | LocalizationData | server | 1 |
| teleport | ZoneData0 | server | 1 |
| tendency-field | ZoneData0 | server | 1 |
| unlocated-store | ItemData | server | 1 |
| weapon-appearance-change-cost | ItemData | server | 1 |
| zone | ZoneData1, ZoneData2 | server | 1 |
| zone-pc-random-spawn | ZoneData0 | server | 1 |

## Attribute names and inferred types

### ArenaMatchingRule

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | game-mode | 360 | string |  |
| record | level | 360 | int |  |
| record | rating-revise | 360 | int |  |

### ArenaPortal

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 8 | string |  |
| record | death-match-1vs1-1 | 1 | string |  |
| record | death-match-1vs1-2 | 1 | string |  |
| record | death-match-1vs1-3 | 1 | string |  |
| record | death-match-1vs1-4 | 1 | string |  |
| record | death-match-unrated-1 | 1 | string |  |
| record | death-match-unrated-2 | 1 | string |  |
| record | death-match-unrated-3 | 1 | string |  |
| record | death-match-unrated-4 | 1 | string |  |
| record | public-raid-mode | 6 | string |  |
| record | required-min-level | 8 | int |  |
| record | tag-match-3vs3-1 | 1 | string |  |
| record | tag-match-3vs3-2 | 1 | string |  |
| record | tag-match-3vs3-3 | 1 | string |  |
| record | tag-match-3vs3-4 | 1 | string |  |
| record | tag-match-unrated-1 | 1 | string |  |
| record | tag-match-unrated-2 | 1 | string |  |
| record | tag-match-unrated-3 | 1 | string |  |
| record | tag-match-unrated-4 | 1 | string |  |
| record | type | 8 | enum-like string ❓ | duel, dungeon, public-raid |

### BadgeSynthesisRecipe

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 1536 | string |  |
| record | category | 1536 | string |  |
| record | result-item-1 | 1536 | string |  |
| record | result-item-10 | 1536 | string |  |
| record | result-item-11 | 1536 | string |  |
| record | result-item-12 | 1536 | string |  |
| record | result-item-13 | 1536 | string |  |
| record | result-item-14 | 1536 | string |  |
| record | result-item-15 | 1536 | string |  |
| record | result-item-16 | 1536 | string |  |
| record | result-item-17 | 1536 | string |  |
| record | result-item-18 | 1536 | string |  |
| record | result-item-19 | 23 | string |  |
| record | result-item-2 | 1536 | string |  |
| record | result-item-20 | 23 | string |  |
| record | result-item-21 | 23 | string |  |
| record | result-item-22 | 23 | string |  |
| record | result-item-23 | 23 | string |  |
| record | result-item-24 | 23 | string |  |
| record | result-item-25 | 23 | string |  |
| record | result-item-26 | 23 | string |  |
| record | result-item-27 | 23 | string |  |
| record | result-item-3 | 1536 | string |  |
| record | result-item-4 | 1536 | string |  |
| record | result-item-5 | 1536 | string |  |
| record | result-item-6 | 1536 | string |  |
| record | result-item-7 | 1536 | string |  |
| record | result-item-8 | 1536 | string |  |
| record | result-item-9 | 1536 | string |  |
| record | result-item-rate-1 | 1536 | int |  |
| record | result-item-rate-10 | 1536 | int |  |
| record | result-item-rate-11 | 1536 | int |  |
| record | result-item-rate-12 | 1536 | int |  |
| record | result-item-rate-13 | 1536 | int |  |
| record | result-item-rate-14 | 1536 | int |  |
| record | result-item-rate-15 | 1536 | int |  |
| record | result-item-rate-16 | 1536 | int |  |
| record | result-item-rate-17 | 1536 | int |  |
| record | result-item-rate-18 | 1536 | int |  |
| record | result-item-rate-19 | 21 | int |  |
| record | result-item-rate-2 | 1536 | int |  |
| record | result-item-rate-20 | 21 | int |  |
| record | result-item-rate-21 | 21 | int |  |
| record | result-item-rate-22 | 21 | int |  |
| record | result-item-rate-23 | 21 | int |  |
| record | result-item-rate-24 | 21 | int |  |
| record | result-item-rate-25 | 21 | int |  |
| record | result-item-rate-26 | 21 | int |  |
| record | result-item-rate-27 | 21 | int |  |
| record | result-item-rate-3 | 1536 | int |  |
| record | result-item-rate-4 | 1536 | int |  |
| record | result-item-rate-5 | 1536 | int |  |
| record | result-item-rate-6 | 1536 | int |  |
| record | result-item-rate-7 | 1536 | int |  |
| record | result-item-rate-8 | 1536 | int |  |
| record | result-item-rate-9 | 1536 | int |  |
| record | score | 1536 | int |  |

### BattleRoyalFieldEffectPouchMesh

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |

### BossGroundAttackTargetPattern

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 182 | string |  |
| record | id | 182 | int |  |
| record | position-1 | 182 | string |  |
| record | position-2 | 182 | string |  |
| record | position-3 | 181 | string |  |
| record | position-4 | 179 | string |  |
| record | position-5 | 177 | string |  |
| record | position-6 | 160 | string |  |

### BossNpc

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 2822 | string |  |
| record | berserk-sequence | 241 | string |  |
| record | berserk-sequence-invoke-time | 207 | int |  |
| record | boss-brain-type | 2822 | enum-like string ❓ | boss, raid-boss |
| record | default-gp | 38 | int |  |
| record | difficulty-type-modify | 105 | string |  |
| record | disable-contribution-reward-range-check | 31 | enum-like string ❓ | y |
| record | gp-section-1 | 38 | int |  |
| record | gp-section-10 | 22 | int |  |
| record | gp-section-11 | 22 | int |  |
| record | gp-section-2 | 38 | int |  |
| record | gp-section-3 | 38 | int |  |
| record | gp-section-4 | 37 | int |  |
| record | gp-section-5 | 37 | int |  |
| record | gp-section-6 | 36 | int |  |
| record | gp-section-7 | 36 | int |  |
| record | gp-section-8 | 30 | int |  |
| record | gp-section-9 | 30 | int |  |
| record | gp-section-cnt | 38 | int |  |
| record | rising-attack-invoke-time | 123 | int |  |
| record | rising-attack-skill3 | 110 | string |  |
| record | skill-spawn-attack-target | 51 | enum-like string ❓ | my-boss-target, n, nearest-target, none |
| record | skill-spawn-group-1 | 65 | string |  |
| record | skill-spawn-group-2 | 2 | string |  |
| record | use-bleeding | 2822 | enum-like string ❓ | n, y |
| record | use-hp-zone-broadcast | 10 | enum-like string ❓ | y |
| record | use-second-gauge | 38 | enum-like string ❓ | y |
| record | wakeup-attack-invoke-time | 17 | int |  |
| record | wakeup-attack-skill3 | 17 | string |  |

### BotScoringCondition

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | condition-type | 8 | string |  |
| record | condition-value-1 | 5 | int |  |
| record | condition-value-2 | 4 | int |  |
| record | condition-value-3 | 4 | int |  |
| record | duration-sec | 8 | int |  |
| record | extra-score-1 | 4 | int |  |
| record | extra-score-2 | 4 | int |  |
| record | extra-score-3 | 4 | int |  |
| record | score-1 | 8 | int |  |
| record | score-2 | 4 | int |  |
| record | score-3 | 4 | int |  |
| record | use-check | 8 | enum-like string ❓ | y |

### BotScoringDestination

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | dest-alias | 993 | string |  |
| record | dest-id | 993 | int |  |
| record | dest-type | 993 | enum-like string ❓ | item-for-env-manipulate, npc, quest, zone |

### BotScoringLevel

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 5 | string |  |
| record | fix-env-reward-min-count | 5 | enum-like string ❓ | n |
| record | isolate-persistent-zone | 5 | enum-like string ❓ | y |
| record | level | 5 | int |  |
| record | reduce-acquire-duel-point | 5 | int |  |
| record | reduce-acquire-exp | 5 | int |  |
| record | reduce-acquire-money | 5 | int |  |
| record | reduce-acquire-partybattle-point | 5 | int |  |
| record | reduce-attack-power | 5 | int |  |
| record | reduce-defensive-power | 5 | int |  |
| record | require-bot-score | 5 | int |  |

### ChallengeList

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | challenge-count-for-reward-1 | 17 | int |  |
| record | challenge-count-for-reward-2 | 17 | int |  |
| record | challenge-count-for-reward-3 | 17 | int |  |
| record | challenge-count-for-reward-4 | 10 | int |  |
| record | challenge-quest-basic-1 | 17 | string |  |
| record | challenge-quest-basic-10 | 7 | string |  |
| record | challenge-quest-basic-11 | 7 | string |  |
| record | challenge-quest-basic-12 | 7 | string |  |
| record | challenge-quest-basic-13 | 7 | string |  |
| record | challenge-quest-basic-14 | 5 | string |  |
| record | challenge-quest-basic-15 | 3 | string |  |
| record | challenge-quest-basic-2 | 17 | string |  |
| record | challenge-quest-basic-3 | 17 | string |  |
| record | challenge-quest-basic-4 | 17 | string |  |
| record | challenge-quest-basic-5 | 17 | string |  |
| record | challenge-quest-basic-6 | 17 | string |  |
| record | challenge-quest-basic-7 | 17 | string |  |
| record | challenge-quest-basic-8 | 7 | string |  |
| record | challenge-quest-basic-9 | 7 | string |  |
| record | challenge-type | 17 | string |  |
| record | required-level | 17 | int |  |
| record | required-mastery-level | 17 | int |  |
| record | reward-1 | 17 | string |  |
| record | reward-2 | 17 | string |  |
| record | reward-3 | 17 | string |  |
| record | reward-4 | 10 | string |  |
| record | type | 17 | enum-like string ❓ | dayofweek, week |
| record | week-start-date-time | 1 | string |  |

### ChallengeListReward

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 61 | string |  |
| record | reward-account-exp | 61 | int |  |
| record | reward-item-1 | 61 | string |  |
| record | reward-item-2 | 61 | string |  |
| record | reward-item-3 | 37 | string |  |
| record | reward-item-4 | 17 | string |  |
| record | reward-item-5 | 13 | string |  |
| record | reward-item-count-1 | 61 | int |  |
| record | reward-item-count-2 | 61 | int |  |
| record | reward-item-count-3 | 37 | int |  |
| record | reward-item-count-4 | 17 | int |  |
| record | reward-item-count-5 | 13 | int |  |
| record | reward-money | 20 | int |  |

### ContributionReward

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 428 | string |  |
| record | contribution-type | 428 | enum-like string ❓ | faction, guild, normal |
| record | group-damage-hp-per-ten-thousand-1 | 418 | int |  |
| record | group-damage-hp-per-ten-thousand-10 | 122 | int |  |
| record | group-damage-hp-per-ten-thousand-2 | 279 | int |  |
| record | group-damage-hp-per-ten-thousand-3 | 144 | int |  |
| record | group-damage-hp-per-ten-thousand-4 | 130 | int |  |
| record | group-damage-hp-per-ten-thousand-5 | 126 | int |  |
| record | group-damage-hp-per-ten-thousand-6 | 126 | int |  |
| record | group-damage-hp-per-ten-thousand-7 | 126 | int |  |
| record | group-damage-hp-per-ten-thousand-8 | 126 | int |  |
| record | group-damage-hp-per-ten-thousand-9 | 126 | int |  |
| record | group-item-1 | 408 | string |  |
| record | group-item-10 | 112 | string |  |
| record | group-item-2 | 269 | string |  |
| record | group-item-3 | 134 | string |  |
| record | group-item-4 | 120 | string |  |
| record | group-item-5 | 116 | string |  |
| record | group-item-6 | 116 | string |  |
| record | group-item-7 | 116 | string |  |
| record | group-item-8 | 116 | string |  |
| record | group-item-9 | 116 | string |  |
| record | group-item-count-1 | 408 | int |  |
| record | group-item-count-10 | 112 | int |  |
| record | group-item-count-2 | 269 | int |  |
| record | group-item-count-3 | 134 | int |  |
| record | group-item-count-4 | 120 | int |  |
| record | group-item-count-5 | 116 | int |  |
| record | group-item-count-6 | 116 | int |  |
| record | group-item-count-7 | 116 | int |  |
| record | group-item-count-8 | 116 | int |  |
| record | group-item-count-9 | 116 | int |  |
| record | guild-reward | 5 | string |  |
| record | killer-item | 344 | string |  |
| record | killer-item-count | 344 | int |  |
| record | minimum-group-item | 25 | string |  |
| record | minimum-group-item-count | 25 | int |  |

### CustomizingDetailIcon

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | order | 2796 | int |  |
| record | param-index | 2819 | int |  |
| record | param-value | 2819 | int |  |
| record | race | 2819 | string |  |
| record | sex | 2819 | string |  |

### CustomizingDetailSlider

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | max-value | 519 | int |  |
| record | min-value | 519 | int |  |
| record | param-index | 530 | int |  |
| record | race | 530 | string |  |
| record | sex | 530 | string |  |

### DuelMatchingRule

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | game-mode | 100 | string |  |
| record | level | 100 | int |  |
| record | rating-revise | 100 | int |  |

### DuelMemberDisplaySkill

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | job-style | 63 | string |  |
| record | pc-job | 63 | string |  |
| record | skill-id-1 | 63 | int |  |
| record | skill-id-2 | 63 | int |  |
| record | skill-id-3 | 63 | int |  |
| record | skill-id-4 | 63 | int |  |

### FielditemDrop

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 1000 | string |  |
| record | area | 69 | string |  |
| record | drop-case | 697 | enum-like string ❓ | bleeding, effect, effect-and-bleeding, killed, manipulated |
| record | drop-place | 995 | enum-like string ❓ | area, front-npc, hand, none |
| record | effect-attribute-1 | 15 | string |  |
| record | fielditem-1 | 1000 | string |  |
| record | fielditem-2 | 49 | string |  |
| record | fielditem-3 | 33 | string |  |
| record | hp-percent | 3 | int |  |
| record | prob-1 | 1000 | int |  |
| record | prob-2 | 50 | int |  |
| record | prob-3 | 33 | int |  |
| record | quest-id | 31 | int |  |
| record | quest-mission-step | 20 | int |  |
| record | type | 1000 | string |  |

### Fish

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | acquire-life-contents-point | 103 | int |  |
| record | alias | 103 | string |  |
| record | dispose-fishing-paste-probability | 103 | int |  |
| record | distribution-type | 103 | string |  |
| record | hookset-available-time-max | 103 | int |  |
| record | hookset-available-time-min | 103 | int |  |
| record | id | 103 | int |  |
| record | reduce-life-contents-cq-amount | 103 | int |  |
| record | size-max | 103 | int |  |
| record | size-min | 103 | int |  |

### Fishing-Paste

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | additional-fish-grade-selection-value | 7 | int |  |
| record | alias | 7 | string |  |
| record | id | 7 | int |  |
| record | item | 7 | string |  |

### GradeBenefits

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | benefit-type | 130 | string |  |
| record | effect | 20 | string |  |
| record | enable-in-arena-duel | 130 | enum-like string ❓ | n, y |
| record | game-grade-key | 130 | int |  |
| record | level | 130 | int |  |
| record | value | 110 | int |  |

### GuildUniformComponent

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | param-type | 416 | int |  |
| record | param-value | 416 | int |  |
| record | recipe-id | 416 | int |  |
| record | required-guild-level | 416 | int |  |
| record | usable | 416 | string |  |

### GuildUniformParamTypeCategory

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | master-param-type | 6 | int |  |
| record | param-type-1 | 6 | int |  |
| record | param-type-2 | 5 | int |  |
| record | param-type-3 | 5 | int |  |
| record | param-type-4 | 1 | int |  |
| record | param-type-5 | 1 | int |  |
| record | param-type-category | 6 | string |  |
| record | payment-type | 6 | enum-like string ❓ | selective |

### GuildUniformRecipe

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | bank-money | 1071 | int |  |
| record | bank-required-item-1 | 744 | string |  |
| record | bank-required-item-2 | 658 | string |  |
| record | bank-required-item-3 | 642 | string |  |
| record | bank-required-item-count-1 | 744 | int |  |
| record | bank-required-item-count-2 | 658 | int |  |
| record | bank-required-item-count-3 | 642 | int |  |
| record | recipe-id | 1071 | int |  |
| record | required-item-1 | 1071 | string |  |
| record | required-item-2 | 726 | string |  |
| record | required-item-count-1 | 1071 | int |  |
| record | required-item-count-2 | 726 | int |  |

### GuildUniformRegistrationQuota

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | increase-step | 3 | int |  |
| record | payment-item | 3 | string |  |
| record | payment-item-count | 3 | int |  |

### GuildUniformSlider

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | max-value | 7 | int |  |
| record | min-value | 7 | int |  |
| record | param-type | 7 | int |  |
| record | required-guild-level | 7 | int |  |

### InterDungeonList

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 58 | string |  |
| record | cave | 7 | string |  |
| record | dungeon | 42 | string |  |
| record | id | 58 | int |  |
| record | party-battle-field | 3 | string |  |
| record | public-raid | 6 | string |  |
| record | use-guild-combat | 3 | enum-like string ❓ | y |
| record | use-pc-contents-type-dungeon | 58 | enum-like string ❓ | n |

### ItemExchange

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 1296 | string |  |
| record | normal-item-1 | 1296 | string |  |
| record | normal-item-2 | 50 | string |  |
| record | normal-item-3 | 9 | string |  |
| record | normal-item-stack-count-1 | 1296 | int |  |
| record | normal-item-stack-count-2 | 50 | int |  |
| record | normal-item-stack-count-3 | 9 | int |  |
| record | required-item-1 | 1296 | string |  |
| record | required-item-min-level-1 | 1296 | int |  |
| record | required-item-stack-count-1 | 1296 | int |  |
| record | rule-usage | 1296 | enum-like string ❓ | antique-exchange, crystallization |

### ItemGrowth

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | exp | 4120 | int |  |
| record | id | 4120 | int |  |
| record | level | 4120 | int |  |

### ItemSkill

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 10782 | string |  |
| record | check-effect-flag-1 | 3843 | string |  |
| record | check-effect-flag-2 | 3 | string |  |
| record | check-effect-flag-op | 5764 | enum-like string ❓ | and, or |
| record | check-skill-result | 9045 | enum-like string ❓ | all, hit-critical-hit |
| record | include-inheritance-skill | 9022 | string |  |
| record | item-sim-skill | 1057 | string |  |
| record | item-sim-skill-target | 1057 | enum-like string ❓ | caster, target |
| record | item-skill-effect-1 | 8672 | string |  |
| record | item-skill-effect-2 | 3063 | string |  |
| record | item-skill-effect-3 | 1177 | string |  |
| record | item-skill-effect-4 | 1026 | string |  |
| record | item-skill-effect-5 | 997 | string |  |
| record | item-skill-effect-6 | 21 | string |  |
| record | item-skill-effect-7 | 1 | string |  |
| record | skill-id | 8885 | int |  |
| record | skill-variation-id-1 | 1135 | int |  |
| record | skill-variation-id-2 | 538 | int |  |
| record | skill-variation-id-3 | 73 | int |  |
| record | skill-variation-id-4 | 20 | int |  |
| record | target-skill-step | 9045 | string |  |
| record | use-exec-caster-effect | 13 | enum-like string ❓ | y |

### ItemSpirit

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | ability-max-1 | 58 | int |  |
| record | ability-max-2 | 55 | int |  |
| record | ability-min-1 | 58 | int |  |
| record | ability-min-2 | 55 | int |  |
| record | alias | 58 | string |  |
| record | applicable-part-1 | 58 | string |  |
| record | applicable-part-2 | 20 | string |  |
| record | applicable-part-3 | 6 | string |  |
| record | applicable-part-4 | 6 | string |  |
| record | attach-ability-1 | 58 | enum-like string ❓ | attack-attribute-value, attack-critical-damage-value, attack-power-equip-min-and-max, max-hp, pve-boss-level-npc-attack-power-equip-min-and-max, pvp-attack-power-equip-min-and-max |
| record | attach-ability-2 | 55 | enum-like string ❓ | attack-attribute-value, attack-critical-damage-value, attack-critical-value, attack-damage-modify-diff, attack-power-equip-min-and-max, defend-damage-modify-diff, pvp-defend-power-equip-value |
| record | attach-distribution-bundle | 58 | enum-like string ❓ | n |
| record | distribution-type | 58 | string |  |
| record | fixed-ingredient-1 | 58 | string |  |
| record | fixed-ingredient-2 | 58 | string |  |
| record | fixed-ingredient-3 | 2 | string |  |
| record | fixed-ingredient-stack-count-1 | 58 | int |  |
| record | fixed-ingredient-stack-count-2 | 58 | int |  |
| record | fixed-ingredient-stack-count-3 | 2 | int |  |
| record | id | 58 | int |  |
| record | level | 58 | int |  |
| record | main-ingredient | 58 | string |  |
| record | money-cost | 58 | int |  |
| record | once-attach-ability-max-1 | 58 | int |  |
| record | once-attach-ability-max-2 | 55 | int |  |
| record | once-attach-ability-min-1 | 58 | int |  |
| record | once-attach-ability-min-2 | 55 | int |  |
| record | success-probability | 58 | int |  |
| record | use-random-ability-value | 58 | enum-like string ❓ | n |

### ItemTransformRecipe

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 35027 | string |  |
| record | category | 35027 | string |  |
| record | consume-fixed-ingredient | 2595 | enum-like string ❓ | y |
| record | consume-sub-ingredient | 34577 | enum-like string ❓ | n, y |
| record | definite-discount-minimum-value | 1544 | int |  |
| record | fixed-ingredient-1 | 13443 | string |  |
| record | fixed-ingredient-2 | 6921 | string |  |
| record | fixed-ingredient-3 | 6531 | string |  |
| record | fixed-ingredient-4 | 5257 | string |  |
| record | fixed-ingredient-5 | 4765 | string |  |
| record | fixed-ingredient-6 | 3979 | string |  |
| record | fixed-ingredient-7 | 2753 | string |  |
| record | fixed-ingredient-8 | 1056 | string |  |
| record | fixed-ingredient-stack-count-1 | 13443 | int |  |
| record | fixed-ingredient-stack-count-2 | 6921 | int |  |
| record | fixed-ingredient-stack-count-3 | 6531 | int |  |
| record | fixed-ingredient-stack-count-4 | 5257 | int |  |
| record | fixed-ingredient-stack-count-5 | 4765 | int |  |
| record | fixed-ingredient-stack-count-6 | 3979 | int |  |
| record | fixed-ingredient-stack-count-7 | 2753 | int |  |
| record | fixed-ingredient-stack-count-8 | 1056 | int |  |
| record | keep-main-ingredient-spirit | 33042 | enum-like string ❓ | y |
| record | keep-main-ingredient-weapon-appearance | 7350 | enum-like string ❓ | y |
| record | keep-main-ingredient-weapon-gem-slot | 5100 | enum-like string ❓ | y |
| record | main-ingredient | 35027 | string |  |
| record | main-ingredient-condition-type | 34693 | string |  |
| record | main-ingredient-min-level | 34514 | int |  |
| record | main-ingredient-stack-count | 35027 | int |  |
| record | main-ingredient-weapon-gem-grade | 72 | int |  |
| record | main-ingredient-weapon-gem-level | 72 | int |  |
| record | main-ingredient-weapon-gem-type | 72 | enum-like string ❓ | amber, amethyst, aquamarine, emerald, garnet, jade, obsidian, ruby, sapphire, topaz |
| record | money-cost | 33762 | int |  |
| record | normal-item-1 | 3857 | string |  |
| record | normal-item-2 | 174 | string |  |
| record | normal-item-select-count | 3857 | int |  |
| record | normal-item-stack-count-1 | 3857 | int |  |
| record | normal-item-stack-count-2 | 174 | int |  |
| record | normal-item-success-probability | 3857 | int |  |
| record | normal-item-total-count | 3857 | int |  |
| record | random-failure-mileage-distribution-type | 4231 | string |  |
| record | random-failure-mileage-influence-recipe-1 | 483 | string |  |
| record | random-failure-mileage-save | 4231 | enum-like string ❓ | y |
| record | random-item-1 | 31097 | string |  |
| record | random-item-2 | 5505 | string |  |
| record | random-item-3 | 2625 | string |  |
| record | random-item-4 | 1694 | string |  |
| record | random-item-5 | 1229 | string |  |
| record | random-item-6 | 922 | string |  |
| record | random-item-7 | 614 | string |  |
| record | random-item-8 | 307 | string |  |
| record | random-item-select-prop-weight-1 | 31097 | int |  |
| record | random-item-select-prop-weight-2 | 5505 | int |  |
| record | random-item-select-prop-weight-3 | 2625 | int |  |
| record | random-item-select-prop-weight-4 | 1694 | int |  |
| record | random-item-select-prop-weight-5 | 1229 | int |  |
| record | random-item-select-prop-weight-6 | 922 | int |  |
| record | random-item-select-prop-weight-7 | 614 | int |  |
| record | random-item-select-prop-weight-8 | 307 | int |  |
| record | random-item-stack-count-1 | 31097 | int |  |
| record | random-item-stack-count-2 | 5510 | int |  |
| record | random-item-stack-count-3 | 2625 | int |  |
| record | random-item-stack-count-4 | 1694 | int |  |
| record | random-item-stack-count-5 | 1229 | int |  |
| record | random-item-stack-count-6 | 922 | int |  |
| record | random-item-stack-count-7 | 614 | int |  |
| record | random-item-stack-count-8 | 307 | int |  |
| record | random-item-success-probability | 31097 | int |  |
| record | random-item-total-count | 31097 | int |  |
| record | random-result-1 | 808 | enum-like string ❓ | lower |
| record | random-result-2 | 1010 | enum-like string ❓ | lower |
| record | random-retry-cost | 3875 | string |  |
| record | rare-item-1 | 5 | string |  |
| record | rare-item-2 | 5 | string |  |
| record | rare-item-select-count | 5 | int |  |
| record | rare-item-stack-count-1 | 5 | int |  |
| record | rare-item-stack-count-2 | 5 | int |  |
| record | rare-item-success-probability | 5 | int |  |
| record | rare-item-total-count | 5 | int |  |
| record | required-inven-capacity | 35027 | int |  |
| record | sub-ingredient-1 | 34989 | string |  |
| record | sub-ingredient-2 | 79 | string |  |
| record | sub-ingredient-3 | 5 | string |  |
| record | sub-ingredient-4 | 3 | string |  |
| record | sub-ingredient-5 | 3 | string |  |
| record | sub-ingredient-6 | 2 | string |  |
| record | sub-ingredient-7 | 1 | string |  |
| record | sub-ingredient-condition-type-1 | 34655 | string |  |
| record | sub-ingredient-condition-type-2 | 77 | enum-like string ❓ | all |
| record | sub-ingredient-condition-type-3 | 3 | enum-like string ❓ | all |
| record | sub-ingredient-condition-type-4 | 3 | enum-like string ❓ | all |
| record | sub-ingredient-condition-type-5 | 3 | enum-like string ❓ | all |
| record | sub-ingredient-condition-type-6 | 2 | enum-like string ❓ | all |
| record | sub-ingredient-condition-type-7 | 2 | enum-like string ❓ | all |
| record | sub-ingredient-min-level-1 | 34605 | int |  |
| record | sub-ingredient-min-level-2 | 73 | int |  |
| record | sub-ingredient-stack-count-1 | 34989 | int |  |
| record | sub-ingredient-stack-count-2 | 79 | int |  |
| record | sub-ingredient-stack-count-3 | 5 | int |  |
| record | sub-ingredient-stack-count-4 | 3 | int |  |
| record | sub-ingredient-stack-count-5 | 3 | int |  |
| record | sub-ingredient-stack-count-6 | 2 | int |  |
| record | sub-ingredient-stack-count-7 | 2 | int |  |
| record | sub-ingredient-weapon-gem-grade-1 | 72 | int |  |
| record | sub-ingredient-weapon-gem-grade-2 | 72 | int |  |
| record | sub-ingredient-weapon-gem-level-1 | 72 | int |  |
| record | sub-ingredient-weapon-gem-level-2 | 72 | int |  |
| record | sub-ingredient-weapon-gem-type-1 | 72 | enum-like string ❓ | amber, amethyst, aquamarine, emerald, garnet, jade, obsidian, ruby, sapphire, topaz |
| record | sub-ingredient-weapon-gem-type-2 | 72 | enum-like string ❓ | amber, amethyst, aquamarine, emerald, garnet, jade, obsidian, ruby, sapphire, topaz |
| record | title-item | 35027 | string |  |
| record | use-random | 31169 | enum-like string ❓ | y |
| record | weapon-gem-transform-result | 72 | int |  |

### ItemUsableRelation

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 74 | string |  |
| record | all | 74 | enum-like string ❓ | y |
| record | attraction-group | 74 | string |  |
| record | beakchung | 74 | enum-like string ❓ | n, y |
| record | cave-dungeon | 74 | enum-like string ❓ | n, y |
| record | deasamak | 74 | enum-like string ❓ | n, y |
| record | default | 74 | enum-like string ❓ | n, y |
| record | duel | 74 | enum-like string ❓ | n |
| record | epichigh | 74 | enum-like string ❓ | n, y |
| record | gunwon | 74 | enum-like string ❓ | n, y |
| record | jeryoungrim | 74 | enum-like string ❓ | n, y |
| record | musin1-15 | 74 | enum-like string ❓ | n, y |
| record | pacheonseongdo | 74 | enum-like string ❓ | n, y |
| record | potion | 74 | enum-like string ❓ | n, y |
| record | suwal | 74 | enum-like string ❓ | n, y |
| record | talisman | 74 | enum-like string ❓ | n, y |
| record | unusable | 74 | enum-like string ❓ | n |

### JumpingCharacter

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 13 | string |  |
| record | condition-level | 13 | int |  |
| record | creation-enable | 13 | enum-like string ❓ | n, y |
| record | equip-costume | 13 | string |  |
| record | equip-weapon | 13 | string |  |
| record | job | 13 | string |  |
| record | jumping-level | 13 | int |  |
| record | tutorial-zone-1 | 1 | string |  |
| record | tutorial-zone-2 | 1 | string |  |
| record | tutorial-zone-3 | 1 | string |  |

### JumpingCharacter2

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | faction1-item | 1 | string |  |
| record | faction2-item | 1 | string |  |
| record | last-epic-quest-to-complete | 1 | int |  |
| record | required-level | 1 | int |  |
| record | required-preceding-epic-quest | 1 | int |  |
| record | reward-exp | 1 | int |  |
| record | reward-item-1 | 1 | string |  |
| record | reward-item-2 | 1 | string |  |
| record | reward-item-3 | 1 | string |  |
| record | reward-item-4 | 1 | string |  |
| record | reward-item-count-1 | 1 | int |  |
| record | reward-item-count-2 | 1 | int |  |
| record | reward-item-count-3 | 1 | int |  |
| record | reward-item-count-4 | 1 | int |  |
| record | reward-money | 1 | int |  |

### JumpingQuest

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | job | 2520 | string |  |
| record | quest-id | 2520 | int |  |

### JumpingQuest2

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | job | 1008 | string |  |
| record | quest-id | 1008 | int |  |

### JumpingTeleport

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | job | 1254 | string |  |
| record | teleport-id | 1254 | int |  |

### JumpingTeleport2

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | teleport | 132 | int |  |

### Mentoring

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 1 | string |  |
| record | enable | 1 | enum-like string ❓ | n |
| record | first-quest-id | 1 | int |  |
| record | last-quest-id | 1 | int |  |
| record | limit-finish-count | 1 | int |  |
| record | max-mentee-count | 1 | int |  |
| record | version | 1 | int |  |

### NpcConvoy

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 51 | string |  |
| record | arrived-act-seq | 50 | string |  |
| record | arrived-attackable | 50 | enum-like string ❓ | n, y |
| record | arrived-brain | 51 | string |  |
| record | arrived-hate-on | 50 | enum-like string ❓ | n, y |
| record | combat-assist | 51 | enum-like string ❓ | n, y |
| record | destination-env2-1 | 51 | string |  |
| record | failed-act-seq | 16 | string |  |
| record | failed-attackable | 51 | enum-like string ❓ | n |
| record | failed-brain | 51 | string |  |
| record | failed-hate-on | 51 | enum-like string ❓ | n |
| record | follow-detect | 16 | string |  |
| record | next-stop-social-max | 13 | int |  |
| record | next-stop-social-min | 13 | int |  |
| record | start-act-seq | 49 | string |  |
| record | start-attackable | 51 | enum-like string ❓ | n, y |
| record | start-brain | 51 | string |  |
| record | start-hate-on | 51 | enum-like string ❓ | n, y |
| record | stop-social-1 | 10 | string |  |
| record | stop-social-2 | 4 | string |  |
| record | stop-social-3 | 4 | string |  |
| record | stop-social-4 | 1 | string |  |

### NpcSquadCombatPattern

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 14 | string |  |
| record | squad-index-1 | 14 | int |  |
| record | squad-index-2 | 11 | int |  |
| record | squad-index-3 | 5 | int |  |
| record | squad-index-4 | 4 | int |  |
| record | squad-index-5 | 1 | int |  |
| record | squad-index-6 | 1 | int |  |
| record | squad-index-7 | 1 | int |  |
| record | target-rule-1 | 14 | string |  |
| record | target-rule-2 | 11 | string |  |
| record | target-rule-3 | 5 | string |  |
| record | target-rule-4 | 4 | string |  |
| record | target-rule-5 | 1 | enum-like string ❓ | stalker-destroyer |
| record | target-rule-6 | 1 | enum-like string ❓ | stalker-assassin |
| record | target-rule-7 | 1 | enum-like string ❓ | stalker-blade-master |

### PcCafeBenefits

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | code | 12 | enum-like string ❓ | ncsoft-korea |
| record | detach-weapon-gem-money-cost-discount-rate | 12 | int |  |
| record | duel-effect | 12 | string |  |
| record | duel-exp-bonus-percent | 12 | int |  |
| record | duel-point-bonus-percent | 12 | int |  |
| record | effect | 12 | string |  |
| record | field-play-point-bonus-percent | 12 | int |  |
| record | item-awakening-money-cost-discount-rate | 12 | int |  |
| record | job | 12 | string |  |
| record | npc-looting-money-reward-percent | 12 | int |  |
| record | party-battle-exp-bonus-percent | 12 | int |  |
| record | party-battle-point-bonus-percent | 12 | int |  |
| record | quest-reward | 12 | enum-like string ❓ | y |
| record | unlocated-warehouse | 12 | enum-like string ❓ | n |
| record | weapon-accessory-transform-money-cost-discount-rate | 12 | int |  |
| record | weapon-gem-transform-money-cost-discount-rate | 12 | int |  |

### PcInitialSetting

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | inventory-size | 93 | int |  |
| record | job | 93 | string |  |
| record | level | 93 | int |  |
| record | quest-1 | 93 | string |  |
| record | race | 93 | string |  |
| record | zone | 93 | string |  |

### PcSkill3

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 1258 | string |  |
| record | job-style | 1258 | string |  |
| record | skill3-1 | 1032 | string |  |
| record | skill3-10 | 9 | string |  |
| record | skill3-11 | 8 | string |  |
| record | skill3-12 | 8 | string |  |
| record | skill3-13 | 7 | string |  |
| record | skill3-14 | 6 | string |  |
| record | skill3-15 | 5 | string |  |
| record | skill3-16 | 5 | string |  |
| record | skill3-17 | 4 | string |  |
| record | skill3-18 | 4 | string |  |
| record | skill3-19 | 4 | string |  |
| record | skill3-2 | 763 | string |  |
| record | skill3-20 | 4 | string |  |
| record | skill3-21 | 4 | string |  |
| record | skill3-22 | 4 | string |  |
| record | skill3-23 | 4 | string |  |
| record | skill3-24 | 4 | string |  |
| record | skill3-25 | 4 | string |  |
| record | skill3-26 | 2 | string |  |
| record | skill3-27 | 2 | string |  |
| record | skill3-28 | 2 | string |  |
| record | skill3-29 | 2 | string |  |
| record | skill3-3 | 303 | string |  |
| record | skill3-30 | 2 | string |  |
| record | skill3-31 | 1 | string |  |
| record | skill3-32 | 1 | string |  |
| record | skill3-33 | 1 | string |  |
| record | skill3-4 | 171 | string |  |
| record | skill3-5 | 85 | string |  |
| record | skill3-6 | 44 | string |  |
| record | skill3-7 | 24 | string |  |
| record | skill3-8 | 23 | string |  |
| record | skill3-9 | 15 | string |  |

### QuestReward

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 2339 | string |  |
| record | basic-account-exp | 198 | int |  |
| record | basic-duel-point | 13 | int |  |
| record | basic-exp | 2337 | int |  |
| record | basic-faction | 174 | string |  |
| record | basic-faction-reputation | 168 | int |  |
| record | basic-field-play-point | 1 | int |  |
| record | basic-mastery-level | 13 | int |  |
| record | basic-money | 2009 | int |  |
| record | basic-party-battle-point | 1 | int |  |
| record | dayofweek-fri-fixed-item-1 | 4 | string |  |
| record | dayofweek-fri-fixed-item-count-1 | 4 | int |  |
| record | dayofweek-mon-fixed-item-1 | 4 | string |  |
| record | dayofweek-mon-fixed-item-count-1 | 4 | int |  |
| record | dayofweek-sat-fixed-item-1 | 4 | string |  |
| record | dayofweek-sat-fixed-item-count-1 | 4 | int |  |
| record | dayofweek-sun-fixed-item-1 | 2 | string |  |
| record | dayofweek-sun-fixed-item-count-1 | 2 | int |  |
| record | dayofweek-thu-fixed-item-1 | 2 | string |  |
| record | dayofweek-thu-fixed-item-count-1 | 2 | int |  |
| record | dayofweek-tue-fixed-item-1 | 2 | string |  |
| record | dayofweek-tue-fixed-item-count-1 | 2 | int |  |
| record | dayofweek-wed-fixed-item-1 | 4 | string |  |
| record | dayofweek-wed-fixed-item-count-1 | 4 | int |  |
| record | elevate-job-style | 1 | enum-like string ❓ | y |
| record | fixed-1-difficulty-type | 41 | enum-like string ❓ | easy, normal |
| record | fixed-1-item-count-1 | 65 | int |  |
| record | fixed-1-item-count-2 | 19 | int |  |
| record | fixed-1-item-count-3 | 5 | int |  |
| record | fixed-1-item-count-4 | 3 | int |  |
| record | fixed-1-job-1 | 24 | string |  |
| record | fixed-1-slot-1 | 65 | string |  |
| record | fixed-1-slot-2 | 19 | string |  |
| record | fixed-1-slot-3 | 5 | string |  |
| record | fixed-1-slot-4 | 3 | string |  |
| record | fixed-10-item-count-1 | 13 | int |  |
| record | fixed-10-item-count-2 | 9 | int |  |
| record | fixed-10-item-count-3 | 4 | int |  |
| record | fixed-10-item-count-4 | 3 | int |  |
| record | fixed-10-job-1 | 13 | string |  |
| record | fixed-10-slot-1 | 13 | string |  |
| record | fixed-10-slot-2 | 9 | string |  |
| record | fixed-10-slot-3 | 4 | string |  |
| record | fixed-10-slot-4 | 3 | string |  |
| record | fixed-11-item-count-1 | 13 | int |  |
| record | fixed-11-item-count-2 | 9 | int |  |
| record | fixed-11-item-count-3 | 4 | int |  |
| record | fixed-11-item-count-4 | 3 | int |  |
| record | fixed-11-job-1 | 13 | string |  |
| record | fixed-11-slot-1 | 13 | string |  |
| record | fixed-11-slot-2 | 9 | string |  |
| record | fixed-11-slot-3 | 4 | string |  |
| record | fixed-11-slot-4 | 3 | string |  |
| record | fixed-12-item-count-1 | 13 | int |  |
| record | fixed-12-item-count-2 | 9 | int |  |
| record | fixed-12-item-count-3 | 4 | int |  |
| record | fixed-12-item-count-4 | 3 | int |  |
| record | fixed-12-job-1 | 13 | string |  |
| record | fixed-12-slot-1 | 13 | string |  |
| record | fixed-12-slot-2 | 9 | string |  |
| record | fixed-12-slot-3 | 4 | string |  |
| record | fixed-12-slot-4 | 3 | string |  |
| record | fixed-2-difficulty-type | 36 | enum-like string ❓ | hard, normal |
| record | fixed-2-item-count-1 | 57 | int |  |
| record | fixed-2-item-count-2 | 18 | int |  |
| record | fixed-2-item-count-3 | 5 | int |  |
| record | fixed-2-item-count-4 | 3 | int |  |
| record | fixed-2-job-1 | 22 | string |  |
| record | fixed-2-slot-1 | 57 | string |  |
| record | fixed-2-slot-2 | 18 | string |  |
| record | fixed-2-slot-3 | 5 | string |  |
| record | fixed-2-slot-4 | 3 | string |  |
| record | fixed-3-difficulty-type | 8 | enum-like string ❓ | hard |
| record | fixed-3-item-count-1 | 27 | int |  |
| record | fixed-3-item-count-2 | 19 | int |  |
| record | fixed-3-item-count-3 | 10 | int |  |
| record | fixed-3-item-count-4 | 3 | int |  |
| record | fixed-3-job-1 | 22 | string |  |
| record | fixed-3-slot-1 | 27 | string |  |
| record | fixed-3-slot-2 | 19 | string |  |
| record | fixed-3-slot-3 | 10 | string |  |
| record | fixed-3-slot-4 | 3 | string |  |
| record | fixed-4-item-count-1 | 22 | int |  |
| record | fixed-4-item-count-2 | 14 | int |  |
| record | fixed-4-item-count-3 | 5 | int |  |
| record | fixed-4-item-count-4 | 3 | int |  |
| record | fixed-4-job-1 | 22 | string |  |
| record | fixed-4-slot-1 | 22 | string |  |
| record | fixed-4-slot-2 | 14 | string |  |
| record | fixed-4-slot-3 | 5 | string |  |
| record | fixed-4-slot-4 | 3 | string |  |
| record | fixed-5-item-count-1 | 22 | int |  |
| record | fixed-5-item-count-2 | 14 | int |  |
| record | fixed-5-item-count-3 | 5 | int |  |
| record | fixed-5-item-count-4 | 3 | int |  |
| record | fixed-5-job-1 | 22 | string |  |
| record | fixed-5-slot-1 | 22 | string |  |
| record | fixed-5-slot-2 | 14 | string |  |
| record | fixed-5-slot-3 | 5 | string |  |
| record | fixed-5-slot-4 | 3 | string |  |
| record | fixed-6-item-count-1 | 22 | int |  |
| record | fixed-6-item-count-2 | 14 | int |  |
| record | fixed-6-item-count-3 | 5 | int |  |
| record | fixed-6-item-count-4 | 3 | int |  |
| record | fixed-6-job-1 | 22 | string |  |
| record | fixed-6-slot-1 | 22 | string |  |
| record | fixed-6-slot-2 | 14 | string |  |
| record | fixed-6-slot-3 | 5 | string |  |
| record | fixed-6-slot-4 | 3 | string |  |
| record | fixed-7-item-count-1 | 22 | int |  |
| record | fixed-7-item-count-2 | 14 | int |  |
| record | fixed-7-item-count-3 | 5 | int |  |
| record | fixed-7-item-count-4 | 3 | int |  |
| record | fixed-7-job-1 | 22 | string |  |
| record | fixed-7-slot-1 | 22 | string |  |
| record | fixed-7-slot-2 | 14 | string |  |
| record | fixed-7-slot-3 | 5 | string |  |
| record | fixed-7-slot-4 | 3 | string |  |
| record | fixed-8-item-count-1 | 20 | int |  |
| record | fixed-8-item-count-2 | 12 | int |  |
| record | fixed-8-item-count-3 | 4 | int |  |
| record | fixed-8-item-count-4 | 3 | int |  |
| record | fixed-8-job-1 | 22 | string |  |
| record | fixed-8-slot-1 | 20 | string |  |
| record | fixed-8-slot-2 | 12 | string |  |
| record | fixed-8-slot-3 | 4 | string |  |
| record | fixed-8-slot-4 | 3 | string |  |
| record | fixed-9-item-count-1 | 15 | int |  |
| record | fixed-9-item-count-2 | 9 | int |  |
| record | fixed-9-item-count-3 | 4 | int |  |
| record | fixed-9-item-count-4 | 3 | int |  |
| record | fixed-9-job-1 | 15 | string |  |
| record | fixed-9-slot-1 | 15 | string |  |
| record | fixed-9-slot-2 | 9 | string |  |
| record | fixed-9-slot-3 | 4 | string |  |
| record | fixed-9-slot-4 | 3 | string |  |
| record | fixed-common-item-count-1 | 1208 | int |  |
| record | fixed-common-item-count-2 | 455 | int |  |
| record | fixed-common-item-count-3 | 190 | int |  |
| record | fixed-common-item-count-4 | 74 | int |  |
| record | fixed-common-slot-1 | 1202 | string |  |
| record | fixed-common-slot-2 | 453 | string |  |
| record | fixed-common-slot-3 | 190 | string |  |
| record | fixed-common-slot-4 | 74 | string |  |
| record | fixed-quest-reward-job-style-skill3-1 | 54 | string |  |
| record | fixed-quest-reward-job-style-skill3-2 | 54 | string |  |
| record | fixed-quest-reward-job-style-skill3-3 | 19 | string |  |
| record | fixed-quest-reward-job-style-skill3-6 | 54 | string |  |
| record | fixed-quest-reward-job-style-skill3-7 | 54 | string |  |
| record | fixed-quest-reward-job-style-skill3-8 | 19 | string |  |
| record | hongmoon-item-1 | 4 | string |  |
| record | hongmoon-item-count-1 | 4 | int |  |
| record | id | 2339 | int |  |
| record | membership-item-1 | 17 | string |  |
| record | membership-item-count-1 | 17 | int |  |
| record | newbie-party-fixed-item-1 | 6 | string |  |
| record | newbie-party-fixed-item-count-1 | 6 | int |  |
| record | oldbie-party-fixed-item-1 | 6 | string |  |
| record | oldbie-party-fixed-item-count-1 | 6 | int |  |
| record | optional-1-difficulty-type | 16 | enum-like string ❓ | easy, normal |
| record | optional-1-item-count-1 | 16 | int |  |
| record | optional-1-item-count-2 | 16 | int |  |
| record | optional-1-slot-1 | 16 | string |  |
| record | optional-1-slot-2 | 16 | string |  |
| record | optional-2-difficulty-type | 8 | enum-like string ❓ | hard |
| record | optional-2-item-count-1 | 5 | int |  |
| record | optional-2-item-count-2 | 5 | int |  |
| record | optional-2-slot-1 | 5 | string |  |
| record | optional-2-slot-2 | 5 | string |  |
| record | optional-3-difficulty-type | 8 | enum-like string ❓ | easy |
| record | optional-3-item-count-1 | 8 | int |  |
| record | optional-3-item-count-2 | 8 | int |  |
| record | optional-3-slot-1 | 8 | string |  |
| record | optional-3-slot-2 | 8 | string |  |
| record | optional-common-item-count-1 | 283 | int |  |
| record | optional-common-item-count-2 | 283 | int |  |
| record | optional-common-item-count-3 | 26 | int |  |
| record | optional-common-item-count-4 | 2 | int |  |
| record | optional-common-slot-1 | 283 | string |  |
| record | optional-common-slot-2 | 283 | string |  |
| record | optional-common-slot-3 | 26 | string |  |
| record | optional-common-slot-4 | 2 | string |  |
| record | quest-first-progress | 918 | enum-like string ❓ | n, y |

### QuestRewardSkill3

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 235 | string |  |
| record | job-style | 235 | string |  |
| record | skill3-1 | 227 | string |  |
| record | skill3-2 | 138 | string |  |
| record | skill3-3 | 92 | string |  |
| record | skill3-4 | 40 | string |  |

### RandomStore

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | acquire-draw-reward-set-repeat-count | 1 | int |  |
| record | alias | 2 | string |  |
| record | charge-of-item-instant-payment-draw | 1 | string |  |
| record | charge-of-item-instant-payment-five-times-draw | 1 | string |  |
| record | charge-of-money-draw | 1 | int |  |
| record | default-prob-group | 2 | string |  |
| record | default-slot-count | 2 | int |  |
| record | free-draw-count-for-fri | 2 | int |  |
| record | free-draw-count-for-mon | 2 | int |  |
| record | free-draw-count-for-sat | 2 | int |  |
| record | free-draw-count-for-sun | 2 | int |  |
| record | free-draw-count-for-thu | 2 | int |  |
| record | free-draw-count-for-tue | 2 | int |  |
| record | free-draw-count-for-wed | 2 | int |  |
| record | free-draw-enable | 2 | enum-like string ❓ | n, y |
| record | free-reward-draw-add-value | 1 | int |  |
| record | free-reward-draw-enable | 1 | enum-like string ❓ | y |
| record | instant-payment-draw-enable | 2 | enum-like string ❓ | n, y |
| record | item-draw-enable | 2 | enum-like string ❓ | n |
| record | limit-level | 2 | int |  |
| record | money-draw-enable | 2 | enum-like string ❓ | n, y |
| record | premium-reward-draw-add-prob-1 | 1 | int |  |
| record | premium-reward-draw-add-prob-2 | 1 | int |  |
| record | premium-reward-draw-add-prob-3 | 1 | int |  |
| record | premium-reward-draw-add-prob-4 | 1 | int |  |
| record | premium-reward-draw-add-prob-5 | 1 | int |  |
| record | premium-reward-draw-add-value-1 | 1 | int |  |
| record | premium-reward-draw-add-value-2 | 1 | int |  |
| record | premium-reward-draw-add-value-3 | 1 | int |  |
| record | premium-reward-draw-add-value-4 | 1 | int |  |
| record | premium-reward-draw-add-value-5 | 1 | int |  |
| record | premium-reward-draw-enable | 2 | enum-like string ❓ | n, y |
| record | premium-reward-draw-premium-draw-count | 2 | int |  |
| record | premium-reward-draw-prob | 2 | int |  |
| record | random-store-number | 2 | string |  |

### RandomStoreDrawReward

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 5 | string |  |
| record | fixed-reward-1 | 3 | string |  |
| record | fixed-reward-count-1 | 3 | int |  |
| record | id | 5 | int |  |
| record | optional-reward-1 | 2 | string |  |
| record | optional-reward-2 | 2 | string |  |
| record | optional-reward-3 | 2 | string |  |
| record | optional-reward-4 | 2 | string |  |
| record | optional-reward-5 | 2 | string |  |
| record | optional-reward-6 | 2 | string |  |
| record | optional-reward-7 | 1 | string |  |
| record | optional-reward-8 | 1 | string |  |
| record | optional-reward-count-1 | 2 | int |  |
| record | optional-reward-count-2 | 2 | int |  |
| record | optional-reward-count-3 | 2 | int |  |
| record | optional-reward-count-4 | 2 | int |  |
| record | optional-reward-count-5 | 2 | int |  |
| record | optional-reward-count-6 | 2 | int |  |
| record | optional-reward-count-7 | 1 | int |  |
| record | optional-reward-count-8 | 1 | int |  |
| record | random-store-number | 5 | string |  |
| record | required-draw-count | 5 | int |  |

### RankingPvEStage

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 2 | string |  |
| record | id | 2 | int |  |
| record | ranking-npc-1 | 2 | string |  |
| record | ranking-npc-geozone-id-1 | 2 | int |  |
| record | ranking-npc-grade-1 | 2 | int |  |
| record | ranking-npc-max-time-1 | 2 | int |  |

### RankingReward

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 1212 | string |  |
| record | id | 1212 | int |  |
| record | season-reward-exp | 1112 | int |  |
| record | season-reward-guild-bank-item-1 | 78 | enum-like string ❓ | gemstone, moonstone, tokengold, tokeniron, tokensilver |
| record | season-reward-guild-bank-item-2 | 76 | enum-like string ❓ | moonstone, tokengold, tokeniron, tokensilver |
| record | season-reward-guild-bank-item-3 | 73 | enum-like string ❓ | tokengold, tokeniron, tokensilver |
| record | season-reward-guild-bank-item-4 | 69 | enum-like string ❓ | tokengold, tokensilver |
| record | season-reward-guild-bank-item-5 | 64 | enum-like string ❓ | tokengold |
| record | season-reward-guild-bank-item-amount-1 | 78 | int |  |
| record | season-reward-guild-bank-item-amount-2 | 76 | int |  |
| record | season-reward-guild-bank-item-amount-3 | 73 | int |  |
| record | season-reward-guild-bank-item-amount-4 | 69 | int |  |
| record | season-reward-guild-bank-item-amount-5 | 64 | int |  |
| record | season-reward-guild-bank-money | 78 | int |  |
| record | season-reward-item-1 | 1111 | string |  |
| record | season-reward-item-2 | 821 | string |  |
| record | season-reward-item-3 | 19 | string |  |
| record | season-reward-item-4 | 3 | string |  |
| record | season-reward-item-amount-1 | 1111 | int |  |
| record | season-reward-item-amount-2 | 821 | int |  |
| record | season-reward-item-amount-3 | 19 | int |  |
| record | season-reward-item-amount-4 | 23 | int |  |
| record | season-reward-money | 1104 | int |  |
| record | week-reward-combat-exp-bonus-rate | 94 | int |  |
| record | week-reward-duel-point-bonus-rate | 90 | int |  |
| record | week-reward-exp | 1097 | int |  |
| record | week-reward-field-point-bonus-rate | 94 | int |  |
| record | week-reward-item-1 | 951 | string |  |
| record | week-reward-item-2 | 660 | string |  |
| record | week-reward-item-3 | 16 | string |  |
| record | week-reward-item-amount-1 | 951 | int |  |
| record | week-reward-item-amount-2 | 660 | int |  |
| record | week-reward-item-amount-3 | 16 | int |  |
| record | week-reward-money | 1089 | int |  |
| record | week-reward-party-battle-point-bonus-rate | 90 | int |  |

### RankingRewardGroup

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | reward-data-1 | 72 | string |  |
| record | reward-data-10 | 60 | string |  |
| record | reward-data-11 | 55 | string |  |
| record | reward-data-12 | 55 | string |  |
| record | reward-data-13 | 55 | string |  |
| record | reward-data-14 | 55 | string |  |
| record | reward-data-15 | 55 | string |  |
| record | reward-data-16 | 53 | string |  |
| record | reward-data-17 | 53 | string |  |
| record | reward-data-18 | 53 | string |  |
| record | reward-data-19 | 53 | string |  |
| record | reward-data-2 | 72 | string |  |
| record | reward-data-20 | 53 | string |  |
| record | reward-data-3 | 72 | string |  |
| record | reward-data-4 | 72 | string |  |
| record | reward-data-5 | 72 | string |  |
| record | reward-data-6 | 71 | string |  |
| record | reward-data-7 | 71 | string |  |
| record | reward-data-8 | 69 | string |  |
| record | reward-data-9 | 60 | string |  |
| record | reward-group-bottom-1 | 72 | int |  |
| record | reward-group-bottom-10 | 60 | int |  |
| record | reward-group-bottom-11 | 55 | int |  |
| record | reward-group-bottom-12 | 55 | int |  |
| record | reward-group-bottom-13 | 55 | int |  |
| record | reward-group-bottom-14 | 55 | int |  |
| record | reward-group-bottom-15 | 55 | int |  |
| record | reward-group-bottom-16 | 53 | int |  |
| record | reward-group-bottom-17 | 53 | int |  |
| record | reward-group-bottom-18 | 53 | int |  |
| record | reward-group-bottom-19 | 53 | int |  |
| record | reward-group-bottom-2 | 72 | int |  |
| record | reward-group-bottom-20 | 53 | int |  |
| record | reward-group-bottom-3 | 72 | int |  |
| record | reward-group-bottom-4 | 72 | int |  |
| record | reward-group-bottom-5 | 72 | int |  |
| record | reward-group-bottom-6 | 71 | int |  |
| record | reward-group-bottom-7 | 71 | int |  |
| record | reward-group-bottom-8 | 69 | int |  |
| record | reward-group-bottom-9 | 60 | int |  |
| record | reward-group-id | 72 | int |  |
| record | reward-group-top-1 | 72 | int |  |
| record | reward-group-top-10 | 60 | int |  |
| record | reward-group-top-11 | 55 | int |  |
| record | reward-group-top-12 | 55 | int |  |
| record | reward-group-top-13 | 55 | int |  |
| record | reward-group-top-14 | 55 | int |  |
| record | reward-group-top-15 | 55 | int |  |
| record | reward-group-top-16 | 53 | int |  |
| record | reward-group-top-17 | 53 | int |  |
| record | reward-group-top-18 | 53 | int |  |
| record | reward-group-top-19 | 53 | int |  |
| record | reward-group-top-2 | 72 | int |  |
| record | reward-group-top-20 | 53 | int |  |
| record | reward-group-top-3 | 72 | int |  |
| record | reward-group-top-4 | 72 | int |  |
| record | reward-group-top-5 | 72 | int |  |
| record | reward-group-top-6 | 71 | int |  |
| record | reward-group-top-7 | 71 | int |  |
| record | reward-group-top-8 | 69 | int |  |
| record | reward-group-top-9 | 60 | int |  |

### SkillCastCondition3

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 12830 | string |  |
| record | check-hp-type | 267 | enum-like string ❓ | point-below, total-max-percent |
| record | check-hp-value | 324 | int |  |
| record | check-repeat-sp-type-1 | 3 | enum-like string ❓ | point |
| record | check-repeat-sp-type-2 | 3 | enum-like string ❓ | point |
| record | check-repeat-sp-value-1 | 3 | int |  |
| record | check-repeat-sp-value-2 | 3 | int |  |
| record | check-sp-type-1 | 1989 | enum-like string ❓ | point, point-above, point-below |
| record | check-sp-type-2 | 4 | enum-like string ❓ | point, point-above |
| record | check-sp-value-1 | 2017 | int |  |
| record | check-sp-value-2 | 4 | int |  |
| record | check-summoned-hp-type | 6 | enum-like string ❓ | total-max-percent |
| record | check-summoned-hp-value | 6 | int |  |
| record | fail-caster-effect-attribute-1 | 1455 | string |  |
| record | fail-caster-effect-attribute-2 | 301 | string |  |
| record | fail-caster-effect-attribute-3 | 101 | string |  |
| record | fail-caster-effect-attribute-4 | 50 | string |  |
| record | fail-caster-effect-attribute-op | 159 | enum-like string ❓ | and, or |
| record | fail-caster-flag-1 | 1787 | string |  |
| record | fail-caster-flag-2 | 849 | string |  |
| record | fail-caster-flag-3 | 371 | string |  |
| record | fail-caster-flag-4 | 232 | string |  |
| record | fail-caster-op | 422 | enum-like string ❓ | and, or |
| record | fail-soulmask-except-normal | 579 | enum-like string ❓ | n |
| record | fail-target-catch-linked | 230 | enum-like string ❓ | n, y |
| record | fail-target-catch-linker | 181 | enum-like string ❓ | n, y |
| record | fail-target-effect-attribute-1 | 1063 | string |  |
| record | fail-target-effect-attribute-2 | 21 | string |  |
| record | fail-target-effect-attribute-3 | 2 | string |  |
| record | fail-target-effect-attribute-op | 52 | enum-like string ❓ | or |
| record | fail-target-flag-1 | 1161 | string |  |
| record | fail-target-flag-2 | 1088 | string |  |
| record | fail-target-flag-3 | 1064 | string |  |
| record | fail-target-flag-4 | 64 | string |  |
| record | fail-target-inhalation-linked | 229 | enum-like string ❓ | n, y |
| record | fail-target-inhalation-linker | 181 | enum-like string ❓ | n, y |
| record | fail-target-mount-linked | 316 | enum-like string ❓ | n, y |
| record | fail-target-mount-linker | 216 | enum-like string ❓ | n, y |
| record | fail-target-op | 1062 | enum-like string ❓ | or |
| record | immune-caster-block-skill | 3277 | enum-like string ❓ | n, y |
| record | is-special-skill | 48 | enum-like string ❓ | n, y |
| record | last-event-1-result-1 | 12 | enum-like string ❓ | bounce, counter, critical-hit, miss |
| record | last-event-1-result-2 | 2 | enum-like string ❓ | dodge, perfect-parry |
| record | last-event-type-1 | 12 | enum-like string ❓ | attack, attacked |
| record | last-event-type-2 | 1 | enum-like string ❓ | attacked |
| record | link-stage | 586 | string |  |
| record | link-state | 4491 | enum-like string ❓ | link, linked, not-link |
| record | link-type | 4491 | enum-like string ❓ | catch, catch-friend, catch-none-human, inhalation, inhalation-catch, mount, none |
| record | move-type | 11253 | enum-like string ❓ | cast-block, free, whole-block |
| record | prev-combo-1-result-1 | 32 | enum-like string ❓ | hit |
| record | prev-combo-1-result-2 | 32 | enum-like string ❓ | critical-hit |
| record | prev-combo-1-result-3 | 32 | enum-like string ❓ | parry |
| record | prev-combo-1-result-4 | 32 | enum-like string ❓ | dodge, perfect-parry |
| record | prev-combo-1-result-5 | 32 | enum-like string ❓ | dodge, miss, perfect-parry |
| record | prev-combo-1-result-6 | 32 | enum-like string ❓ | dodge, miss |
| record | prev-combo-2-result-1 | 34 | enum-like string ❓ | critical-hit, hit |
| record | prev-combo-2-result-2 | 30 | enum-like string ❓ | critical-hit |
| record | prev-combo-2-result-3 | 30 | enum-like string ❓ | parry |
| record | prev-combo-2-result-4 | 30 | enum-like string ❓ | dodge, perfect-parry |
| record | prev-combo-2-result-5 | 30 | enum-like string ❓ | miss, perfect-parry |
| record | prev-combo-2-result-6 | 12 | enum-like string ❓ | miss |
| record | prev-combo-3-result-1 | 7 | enum-like string ❓ | critical-hit, hit |
| record | prev-combo-3-result-2 | 6 | enum-like string ❓ | critical-hit |
| record | prev-combo-3-result-3 | 6 | enum-like string ❓ | parry |
| record | prev-combo-3-result-4 | 6 | enum-like string ❓ | dodge |
| record | prev-combo-3-result-5 | 6 | enum-like string ❓ | perfect-parry |
| record | prev-combo-3-result-6 | 6 | enum-like string ❓ | miss |
| record | prev-combo-4-result-1 | 12 | enum-like string ❓ | hit |
| record | prev-combo-4-result-2 | 12 | enum-like string ❓ | critical-hit |
| record | prev-combo-4-result-3 | 12 | enum-like string ❓ | parry |
| record | prev-combo-4-result-4 | 12 | enum-like string ❓ | perfect-parry |
| record | prev-combo-4-result-5 | 12 | enum-like string ❓ | dodge |
| record | prev-combo-4-result-6 | 12 | enum-like string ❓ | miss |
| record | prev-combo-check-point-1 | 611 | enum-like string ❓ | exec-first, fire |
| record | prev-combo-check-point-2 | 156 | enum-like string ❓ | fire |
| record | prev-combo-check-point-3 | 67 | enum-like string ❓ | fire |
| record | prev-combo-check-point-4 | 30 | enum-like string ❓ | exec-first, fire |
| record | prev-combo-event-type-1 | 608 | enum-like string ❓ | attack |
| record | prev-combo-event-type-2 | 156 | enum-like string ❓ | attack |
| record | prev-combo-event-type-3 | 67 | enum-like string ❓ | attack |
| record | prev-combo-event-type-4 | 30 | enum-like string ❓ | attack |
| record | prev-combo-id-1 | 608 | int |  |
| record | prev-combo-id-2 | 157 | int |  |
| record | prev-combo-id-3 | 68 | int |  |
| record | prev-combo-id-4 | 30 | int |  |
| record | required-weapon-type | 3826 | enum-like string ❓ | aura-bangle, dagger, gauntlet, great-sword, gun, long-bow, staff, sword, two-handed-axe |
| record | soul-mask-1 | 471 | string |  |
| record | soul-mask-2 | 314 | string |  |
| record | stance | 1415 | string |  |
| record | success-caster-combat-mode | 2 | enum-like string ❓ | not-combat-mode |
| record | success-caster-effect-attribute-1 | 441 | string |  |
| record | success-caster-effect-attribute-2 | 41 | string |  |
| record | success-caster-effect-attribute-3 | 3 | string |  |
| record | success-caster-effect-attribute-4 | 2 | string |  |
| record | success-caster-effect-attribute-op | 64 | enum-like string ❓ | and, or |
| record | success-caster-filter-1 | 35 | string |  |
| record | success-caster-filter-2 | 6 | string |  |
| record | success-caster-filter-op | 5 | enum-like string ❓ | or |
| record | success-caster-flag-1 | 1805 | string |  |
| record | success-caster-flag-2 | 582 | string |  |
| record | success-caster-flag-3 | 208 | string |  |
| record | success-caster-flag-4 | 49 | string |  |
| record | success-caster-op | 702 | enum-like string ❓ | and, or |
| record | success-target-counter | 70 | enum-like string ❓ | n |
| record | success-target-dead-state | 12723 | enum-like string ❓ | all, dead, dead-exhaustion, exhaustion, not-dead, not-dead-restoration, not-dead-restoration-exhaustion, restoration |
| record | success-target-effect-attribute-1 | 52 | string |  |
| record | success-target-effect-attribute-2 | 1 | string |  |
| record | success-target-effect-attribute-op | 5 | enum-like string ❓ | or |
| record | success-target-flag-1 | 437 | string |  |
| record | success-target-flag-2 | 227 | string |  |
| record | success-target-flag-3 | 131 | string |  |
| record | success-target-flag-4 | 10 | string |  |
| record | success-target-op | 287 | enum-like string ❓ | or |
| record | summoned-use-only | 65 | enum-like string ❓ | y |
| record | target-type | 12537 | enum-like string ❓ | all, enemy, friend, my-party |
| record | unnecessary-acquired | 734 | enum-like string ❓ | n, y |

### SkillDashAttribute3

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 37 | string |  |
| record | calculate-dash-position-include-radius | 3 | string |  |
| record | dash-margin | 36 | int |  |
| record | dash-type | 36 | enum-like string ❓ | dash, left-dash, position-dash, rear-dash, right-dash, zero-point-dash |

### SkillGatherRange3

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 3783 | string |  |
| record | cast-dir-caster | 3783 | string |  |
| record | cast-dir-target | 3783 | string |  |
| record | exec-dir-caster-1 | 3719 | string |  |
| record | exec-dir-caster-2 | 3720 | string |  |
| record | exec-dir-caster-3 | 3720 | string |  |
| record | exec-dir-caster-4 | 3720 | string |  |
| record | exec-dir-caster-5 | 3720 | string |  |
| record | exec-dir-target-1 | 3723 | string |  |
| record | exec-dir-target-2 | 3724 | string |  |
| record | exec-dir-target-3 | 3724 | string |  |
| record | exec-dir-target-4 | 3724 | string |  |
| record | exec-dir-target-5 | 3724 | string |  |
| record | fire-dir-caster-1 | 104 | string |  |
| record | fire-dir-caster-2 | 104 | string |  |
| record | fire-dir-caster-3 | 104 | string |  |
| record | fire-dir-caster-4 | 104 | string |  |
| record | fire-dir-caster-5 | 104 | string |  |
| record | fire-dir-target-1 | 106 | int |  |
| record | fire-dir-target-2 | 106 | int |  |
| record | fire-dir-target-3 | 106 | int |  |
| record | fire-dir-target-4 | 106 | int |  |
| record | fire-dir-target-5 | 106 | int |  |
| record | gather-depth-1 | 3759 | int |  |
| record | gather-depth-2 | 3759 | int |  |
| record | gather-depth-3 | 3759 | int |  |
| record | gather-depth-4 | 3759 | int |  |
| record | gather-depth-5 | 3756 | int |  |
| record | gather-height-1 | 3760 | int |  |
| record | gather-height-2 | 3760 | int |  |
| record | gather-height-3 | 3760 | int |  |
| record | gather-height-4 | 3760 | int |  |
| record | gather-height-5 | 3760 | int |  |
| record | gather-laser-back-distance-max-1 | 1342 | int |  |
| record | gather-laser-back-distance-max-2 | 1338 | int |  |
| record | gather-laser-back-distance-max-3 | 1337 | int |  |
| record | gather-laser-back-distance-max-4 | 1335 | int |  |
| record | gather-laser-back-distance-max-5 | 1335 | int |  |
| record | gather-laser-back-distance-min-1 | 1302 | int |  |
| record | gather-laser-back-distance-min-2 | 1303 | int |  |
| record | gather-laser-back-distance-min-3 | 1300 | int |  |
| record | gather-laser-back-distance-min-4 | 1304 | int |  |
| record | gather-laser-back-distance-min-5 | 1304 | int |  |
| record | gather-laser-front-distance-max-1 | 2075 | int |  |
| record | gather-laser-front-distance-max-2 | 2070 | int |  |
| record | gather-laser-front-distance-max-3 | 2057 | int |  |
| record | gather-laser-front-distance-max-4 | 2051 | int |  |
| record | gather-laser-front-distance-max-5 | 2061 | int |  |
| record | gather-laser-front-distance-min-1 | 1963 | int |  |
| record | gather-laser-front-distance-min-2 | 1980 | int |  |
| record | gather-laser-front-distance-min-3 | 1978 | int |  |
| record | gather-laser-front-distance-min-4 | 1979 | int |  |
| record | gather-laser-front-distance-min-5 | 1974 | int |  |
| record | gather-laser-width-max-1 | 2091 | int |  |
| record | gather-laser-width-max-2 | 2089 | int |  |
| record | gather-laser-width-max-3 | 2086 | int |  |
| record | gather-laser-width-max-4 | 2085 | int |  |
| record | gather-laser-width-max-5 | 2086 | int |  |
| record | gather-laser-width-min-1 | 1950 | int |  |
| record | gather-laser-width-min-2 | 1951 | int |  |
| record | gather-laser-width-min-3 | 1951 | int |  |
| record | gather-laser-width-min-4 | 1961 | int |  |
| record | gather-laser-width-min-5 | 1961 | int |  |
| record | gather-radius-max-1 | 2389 | int |  |
| record | gather-radius-max-2 | 2387 | int |  |
| record | gather-radius-max-3 | 2383 | int |  |
| record | gather-radius-max-4 | 2378 | int |  |
| record | gather-radius-max-5 | 2377 | int |  |
| record | gather-radius-min-1 | 2180 | int |  |
| record | gather-radius-min-2 | 2180 | int |  |
| record | gather-radius-min-3 | 2182 | int |  |
| record | gather-radius-min-4 | 2176 | int |  |
| record | gather-radius-min-5 | 2176 | int |  |
| record | range-cast-depth | 3783 | int |  |
| record | range-cast-height | 3783 | int |  |
| record | range-cast-laser-width | 973 | int |  |
| record | range-cast-max | 3783 | int |  |
| record | range-cast-min | 3783 | int |  |
| record | range-exec-depth-1 | 3741 | int |  |
| record | range-exec-depth-2 | 3741 | int |  |
| record | range-exec-depth-3 | 3741 | int |  |
| record | range-exec-depth-4 | 3741 | int |  |
| record | range-exec-depth-5 | 3741 | int |  |
| record | range-exec-height-1 | 3741 | int |  |
| record | range-exec-height-2 | 3741 | int |  |
| record | range-exec-height-3 | 3741 | int |  |
| record | range-exec-height-4 | 3741 | int |  |
| record | range-exec-height-5 | 3741 | int |  |
| record | range-exec-laser-width-1 | 1164 | int |  |
| record | range-exec-laser-width-2 | 1165 | int |  |
| record | range-exec-laser-width-3 | 1165 | int |  |
| record | range-exec-laser-width-4 | 1165 | int |  |
| record | range-exec-laser-width-5 | 1165 | int |  |
| record | range-exec-max-1 | 3739 | int |  |
| record | range-exec-max-2 | 3739 | int |  |
| record | range-exec-max-3 | 3739 | int |  |
| record | range-exec-max-4 | 3739 | int |  |
| record | range-exec-max-5 | 3739 | int |  |
| record | range-exec-min-1 | 3579 | int |  |
| record | range-exec-min-2 | 3579 | int |  |
| record | range-exec-min-3 | 3579 | int |  |
| record | range-exec-min-4 | 3579 | int |  |
| record | range-exec-min-5 | 3579 | int |  |
| record | range-fire-depth-1 | 3740 | int |  |
| record | range-fire-depth-2 | 3740 | int |  |
| record | range-fire-depth-3 | 3740 | int |  |
| record | range-fire-depth-4 | 3740 | int |  |
| record | range-fire-depth-5 | 3740 | int |  |
| record | range-fire-height-1 | 3741 | int |  |
| record | range-fire-height-2 | 3741 | int |  |
| record | range-fire-height-3 | 3741 | int |  |
| record | range-fire-height-4 | 3741 | int |  |
| record | range-fire-height-5 | 3741 | int |  |
| record | range-fire-laser-width-1 | 1197 | int |  |
| record | range-fire-laser-width-2 | 1197 | int |  |
| record | range-fire-laser-width-3 | 1197 | int |  |
| record | range-fire-laser-width-4 | 1197 | int |  |
| record | range-fire-laser-width-5 | 1197 | int |  |
| record | range-fire-max-1 | 3737 | int |  |
| record | range-fire-max-2 | 3737 | int |  |
| record | range-fire-max-3 | 3737 | int |  |
| record | range-fire-max-4 | 3737 | int |  |
| record | range-fire-max-5 | 3737 | int |  |
| record | range-fire-min-1 | 3118 | int |  |
| record | range-fire-min-2 | 3118 | int |  |
| record | range-fire-min-3 | 3118 | int |  |
| record | range-fire-min-4 | 3118 | int |  |
| record | range-fire-min-5 | 3118 | int |  |
| record | shifting-laser-depth | 5 | int |  |
| record | shifting-laser-distance-max | 5 | int |  |
| record | shifting-laser-distance-min | 4 | int |  |
| record | shifting-laser-height | 5 | int |  |
| record | shifting-laser-offset | 5 | int |  |
| record | shifting-laser-width-max | 5 | int |  |
| record | shifting-laser-width-min | 4 | int |  |
| record | target-gather-max-1 | 51 | int |  |
| record | target-gather-max-2 | 27 | int |  |
| record | target-gather-max-3 | 26 | int |  |
| record | target-gather-max-4 | 26 | int |  |
| record | target-gather-max-5 | 26 | int |  |

### SkillModifyLimit

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 14 | string |  |
| record | damage-power-percent-max-ratio-percent | 14 | int |  |
| record | damage-power-percent-min-ratio-percent | 14 | int |  |
| record | heal-percent-max-ratio-percent | 14 | int |  |
| record | heal-percent-min-ratio-percent | 14 | int |  |
| record | hp-drain-percent-max-ratio-percent | 14 | int |  |
| record | hp-drain-percent-min-ratio-percent | 14 | int |  |
| record | recycle-duration-max-ratio-percent | 14 | int |  |
| record | recycle-duration-min-ratio-percent | 14 | int |  |
| record | sp-consume-max-value-1 | 14 | int |  |
| record | sp-consume-max-value-2 | 14 | int |  |
| record | sp-consume-min-value-1 | 14 | int |  |
| record | sp-consume-min-value-2 | 14 | int |  |

### SkillResultControll3

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 57 | string |  |
| record | immune-caster-casting-bounce | 57 | enum-like string ❓ | n, y |
| record | immune-caster-casting-dodge | 57 | enum-like string ❓ | n, y |
| record | immune-caster-casting-immune | 57 | enum-like string ❓ | n, y |
| record | immune-caster-casting-parry | 57 | enum-like string ❓ | n, y |
| record | immune-caster-casting-perfect-parry | 57 | enum-like string ❓ | n, y |
| record | immune-caster-flying-bounce-1 | 57 | enum-like string ❓ | n, y |
| record | immune-caster-flying-bounce-2 | 57 | enum-like string ❓ | n, y |
| record | immune-caster-flying-bounce-3 | 57 | enum-like string ❓ | n, y |
| record | immune-caster-flying-bounce-4 | 57 | enum-like string ❓ | n, y |
| record | immune-caster-flying-bounce-5 | 57 | enum-like string ❓ | n, y |
| record | immune-caster-flying-dodge-1 | 57 | enum-like string ❓ | n, y |
| record | immune-caster-flying-dodge-2 | 57 | enum-like string ❓ | n, y |
| record | immune-caster-flying-dodge-3 | 57 | enum-like string ❓ | n, y |
| record | immune-caster-flying-dodge-4 | 57 | enum-like string ❓ | n, y |
| record | immune-caster-flying-dodge-5 | 57 | enum-like string ❓ | n, y |
| record | immune-caster-flying-immune-1 | 57 | enum-like string ❓ | n, y |
| record | immune-caster-flying-immune-2 | 57 | enum-like string ❓ | n, y |
| record | immune-caster-flying-immune-3 | 57 | enum-like string ❓ | n, y |
| record | immune-caster-flying-immune-4 | 57 | enum-like string ❓ | n, y |
| record | immune-caster-flying-immune-5 | 57 | enum-like string ❓ | n, y |
| record | immune-caster-flying-parry-1 | 57 | enum-like string ❓ | n, y |
| record | immune-caster-flying-parry-2 | 57 | enum-like string ❓ | n, y |
| record | immune-caster-flying-parry-3 | 57 | enum-like string ❓ | n, y |
| record | immune-caster-flying-parry-4 | 57 | enum-like string ❓ | n, y |
| record | immune-caster-flying-parry-5 | 57 | enum-like string ❓ | n, y |
| record | immune-caster-flying-perfect-parry-1 | 57 | enum-like string ❓ | n, y |
| record | immune-caster-flying-perfect-parry-2 | 57 | enum-like string ❓ | n, y |
| record | immune-caster-flying-perfect-parry-3 | 57 | enum-like string ❓ | n, y |
| record | immune-caster-flying-perfect-parry-4 | 57 | enum-like string ❓ | n, y |
| record | immune-caster-flying-perfect-parry-5 | 57 | enum-like string ❓ | n, y |
| record | immune-miss-result-1 | 57 | enum-like string ❓ | n, y |
| record | immune-miss-result-2 | 57 | enum-like string ❓ | n, y |
| record | immune-miss-result-3 | 57 | enum-like string ❓ | n, y |
| record | immune-miss-result-4 | 57 | enum-like string ❓ | n, y |
| record | immune-miss-result-5 | 57 | enum-like string ❓ | n, y |
| record | immune-target-bounce-skill-1 | 57 | enum-like string ❓ | n, y |
| record | immune-target-bounce-skill-2 | 57 | enum-like string ❓ | n, y |
| record | immune-target-bounce-skill-3 | 57 | enum-like string ❓ | n, y |
| record | immune-target-bounce-skill-4 | 57 | enum-like string ❓ | n, y |
| record | immune-target-bounce-skill-5 | 57 | enum-like string ❓ | n, y |
| record | immune-target-counter-skill-1 | 57 | enum-like string ❓ | n, y |
| record | immune-target-counter-skill-2 | 57 | enum-like string ❓ | n, y |
| record | immune-target-counter-skill-3 | 57 | enum-like string ❓ | n, y |
| record | immune-target-counter-skill-4 | 57 | enum-like string ❓ | n, y |
| record | immune-target-counter-skill-5 | 57 | enum-like string ❓ | n, y |
| record | immune-target-dodge-skill-1 | 57 | enum-like string ❓ | n, y |
| record | immune-target-dodge-skill-2 | 57 | enum-like string ❓ | n, y |
| record | immune-target-dodge-skill-3 | 57 | enum-like string ❓ | n, y |
| record | immune-target-dodge-skill-4 | 57 | enum-like string ❓ | n, y |
| record | immune-target-dodge-skill-5 | 57 | enum-like string ❓ | n, y |
| record | immune-target-parry-skill-1 | 57 | enum-like string ❓ | n, y |
| record | immune-target-parry-skill-2 | 57 | enum-like string ❓ | n, y |
| record | immune-target-parry-skill-3 | 57 | enum-like string ❓ | n, y |
| record | immune-target-parry-skill-4 | 57 | enum-like string ❓ | n, y |
| record | immune-target-parry-skill-5 | 57 | enum-like string ❓ | n, y |
| record | immune-target-perfect-parry-skill-1 | 57 | enum-like string ❓ | n, y |
| record | immune-target-perfect-parry-skill-2 | 57 | enum-like string ❓ | n, y |
| record | immune-target-perfect-parry-skill-3 | 57 | enum-like string ❓ | n, y |
| record | immune-target-perfect-parry-skill-4 | 57 | enum-like string ❓ | n, y |
| record | immune-target-perfect-parry-skill-5 | 57 | enum-like string ❓ | n, y |

### SkillSkin

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 3 | string |  |
| record | id | 3 | int |  |
| record | skill-skin-type | 3 | enum-like string ❓ | use-grocery |

### SkillTargetFilter3

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 138 | string |  |
| record | exec-filter-target-1 | 137 | int |  |
| record | exec-filter-target-2 | 114 | int |  |
| record | exec-filter-target-3 | 64 | string |  |
| record | exec-filter-target-4 | 9 | int |  |
| record | exec-filter-target-5 | 1 | int |  |
| record | exec-filter-target-6 | 1 | int |  |
| record | fire-filter-target-1 | 138 | int |  |
| record | fire-filter-target-2 | 116 | int |  |
| record | fire-filter-target-3 | 65 | string |  |
| record | fire-filter-target-4 | 9 | int |  |
| record | fire-filter-target-5 | 1 | int |  |
| record | fire-filter-target-6 | 1 | int |  |

### SlateScroll

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 18 | string |  |
| record | ingredient-count-1 | 18 | int |  |
| record | ingredient-count-2 | 18 | int |  |
| record | ingredient-count-3 | 18 | int |  |
| record | ingredient-count-4 | 18 | int |  |
| record | ingredient-item-1 | 18 | string |  |
| record | ingredient-item-2 | 18 | string |  |
| record | ingredient-item-3 | 18 | string |  |
| record | ingredient-item-4 | 18 | string |  |
| record | ingredient-money | 17 | int |  |
| record | secondary-cash | 12 | string |  |

### SlateScrollStone

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | scroll | 1395 | string |  |
| record | stone | 1395 | string |  |

### SlateStone

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 405 | string |  |
| record | duplication-permission | 405 | enum-like string ❓ | n, y |
| record | grade | 405 | int |  |
| record | group | 180 | int |  |
| record | id | 405 | int |  |
| record | init-ability-value-1 | 405 | int |  |
| record | init-ability-value-2 | 405 | int |  |
| record | max-ability-value-1 | 405 | int |  |
| record | max-ability-value-2 | 405 | int |  |
| record | modify-ability-1 | 405 | enum-like string ❓ | abnormal-attack-power-value, abnormal-defend-power-value, attack-attribute-value, attack-critical-damage-value, attack-hit-value, attack-pierce-value, attack-power-creature-min-max, defend-critical-value, defend-dodge-value, defend-parry-value, heal-power-value, hp-regen, max-hp, pve-boss-level-npc-attack-power-creature-min-max, pve-boss-level-npc-defend-power-creature-value, pvp-attack-power-creature-min-max, pvp-defend-power-creature-value |
| record | modify-ability-2 | 405 | enum-like string ❓ | abnormal-attack-power-value, abnormal-defend-power-value, attack-attribute-value, attack-critical-damage-value, attack-critical-value, attack-pierce-value, attack-power-creature-min-max, defend-critical-value, defend-dodge-value, defend-parry-value, defend-power-creature-value, heal-power-value, max-hp, pve-boss-level-npc-attack-power-creature-min-max, pve-boss-level-npc-defend-power-creature-value, pvp-attack-power-creature-min-max, pvp-defend-power-creature-value |
| record | unit-ability-value-1 | 405 | int |  |
| record | unit-ability-value-2 | 405 | int |  |

### SoulMask

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 25 | string |  |
| record | duration | 25 | int |  |
| record | effect-1 | 13 | string |  |
| record | effect-2 | 7 | string |  |
| record | not-dead-to-exhaustion-detach | 6 | enum-like string ❓ | y |
| record | run-speed | 25 | int |  |
| record | soulmask-priority | 4 | int |  |
| record | soulmask-type | 21 | enum-like string ❓ | heromask-duel, transform |

### SoulMaskSet

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 32 | string |  |
| record | run-speed | 32 | int |  |
| record | soul-mask-id-1 | 32 | int |  |
| record | soul-mask-id-2 | 32 | int |  |
| record | soulmask-type | 32 | enum-like string ❓ | normal, transform |

### SummonedBeautyShop

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 5 | string |  |
| record | beauty-item-1 | 5 | string |  |
| record | beauty-item-10 | 5 | string |  |
| record | beauty-item-11 | 5 | string |  |
| record | beauty-item-12 | 5 | string |  |
| record | beauty-item-13 | 5 | string |  |
| record | beauty-item-14 | 5 | string |  |
| record | beauty-item-15 | 5 | string |  |
| record | beauty-item-16 | 5 | string |  |
| record | beauty-item-17 | 5 | string |  |
| record | beauty-item-18 | 5 | string |  |
| record | beauty-item-19 | 5 | string |  |
| record | beauty-item-2 | 5 | string |  |
| record | beauty-item-20 | 5 | string |  |
| record | beauty-item-21 | 5 | string |  |
| record | beauty-item-22 | 5 | string |  |
| record | beauty-item-23 | 5 | string |  |
| record | beauty-item-24 | 5 | string |  |
| record | beauty-item-25 | 5 | string |  |
| record | beauty-item-26 | 5 | string |  |
| record | beauty-item-27 | 5 | string |  |
| record | beauty-item-28 | 5 | string |  |
| record | beauty-item-29 | 5 | string |  |
| record | beauty-item-3 | 5 | string |  |
| record | beauty-item-30 | 5 | string |  |
| record | beauty-item-31 | 5 | string |  |
| record | beauty-item-32 | 5 | string |  |
| record | beauty-item-33 | 5 | string |  |
| record | beauty-item-34 | 5 | string |  |
| record | beauty-item-35 | 5 | string |  |
| record | beauty-item-36 | 5 | string |  |
| record | beauty-item-37 | 5 | string |  |
| record | beauty-item-38 | 5 | string |  |
| record | beauty-item-39 | 5 | string |  |
| record | beauty-item-4 | 5 | string |  |
| record | beauty-item-40 | 5 | string |  |
| record | beauty-item-41 | 5 | string |  |
| record | beauty-item-42 | 5 | string |  |
| record | beauty-item-43 | 5 | string |  |
| record | beauty-item-44 | 5 | string |  |
| record | beauty-item-45 | 5 | string |  |
| record | beauty-item-46 | 5 | string |  |
| record | beauty-item-47 | 5 | string |  |
| record | beauty-item-48 | 5 | string |  |
| record | beauty-item-49 | 5 | string |  |
| record | beauty-item-5 | 5 | string |  |
| record | beauty-item-50 | 5 | string |  |
| record | beauty-item-51 | 5 | string |  |
| record | beauty-item-52 | 5 | string |  |
| record | beauty-item-53 | 5 | string |  |
| record | beauty-item-54 | 5 | string |  |
| record | beauty-item-55 | 5 | string |  |
| record | beauty-item-56 | 5 | string |  |
| record | beauty-item-57 | 5 | string |  |
| record | beauty-item-58 | 5 | string |  |
| record | beauty-item-59 | 5 | string |  |
| record | beauty-item-6 | 5 | string |  |
| record | beauty-item-60 | 5 | string |  |
| record | beauty-item-61 | 5 | string |  |
| record | beauty-item-62 | 5 | string |  |
| record | beauty-item-63 | 5 | string |  |
| record | beauty-item-64 | 5 | string |  |
| record | beauty-item-65 | 5 | string |  |
| record | beauty-item-66 | 5 | string |  |
| record | beauty-item-67 | 5 | string |  |
| record | beauty-item-68 | 5 | string |  |
| record | beauty-item-69 | 5 | string |  |
| record | beauty-item-7 | 5 | string |  |
| record | beauty-item-70 | 5 | string |  |
| record | beauty-item-71 | 5 | string |  |
| record | beauty-item-72 | 5 | string |  |
| record | beauty-item-73 | 5 | string |  |
| record | beauty-item-74 | 5 | string |  |
| record | beauty-item-75 | 5 | string |  |
| record | beauty-item-76 | 5 | string |  |
| record | beauty-item-77 | 5 | string |  |
| record | beauty-item-8 | 5 | string |  |
| record | beauty-item-9 | 5 | string |  |
| record | sell-rate | 5 | int |  |

### SummonedDesignerPreset

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 5 | string |  |
| record | body | 5 | int |  |
| record | body-color | 5 | int |  |
| record | body-shape | 5 | int |  |
| record | decal | 5 | int |  |
| record | eye | 5 | int |  |
| record | eye-color | 5 | int |  |
| record | face | 5 | int |  |
| record | index | 5 | int |  |
| record | race | 5 | string |  |

### SummonedMasteryLevel

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | abnormal-attack-base-percent | 160 | int |  |
| record | abnormal-attack-power-modify | 160 | int |  |
| record | abnormal-attack-power-value | 160 | int |  |
| record | abnormal-defend-base-percent | 160 | int |  |
| record | abnormal-defend-power-modify | 160 | int |  |
| record | abnormal-defend-power-value | 160 | int |  |
| record | aoe-defend-base-percent | 560 | int |  |
| record | aoe-defend-power-modify | 560 | int |  |
| record | aoe-defend-power-value | 560 | int |  |
| record | attack-abnormal-hit-base-percent | 160 | int |  |
| record | attack-abnormal-hit-value | 160 | int |  |
| record | attack-aoe-pierce-value | 160 | int |  |
| record | attack-attribute-base-percent | 560 | int |  |
| record | attack-attribute-modify | 560 | int |  |
| record | attack-attribute-value | 560 | int |  |
| record | attack-concentrate-value | 560 | int |  |
| record | attack-counter-damage-modify | 560 | int |  |
| record | attack-critical-base-percent | 560 | int |  |
| record | attack-critical-damage-modify | 560 | int |  |
| record | attack-critical-damage-percent | 560 | int |  |
| record | attack-critical-damage-value | 160 | int |  |
| record | attack-critical-value | 560 | int |  |
| record | attack-critical-value-modify | 560 | int |  |
| record | attack-damage-modify-diff | 560 | int |  |
| record | attack-damage-modify-percent | 560 | int |  |
| record | attack-hit-base-percent | 560 | int |  |
| record | attack-hit-value | 560 | int |  |
| record | attack-hit-value-modify | 560 | int |  |
| record | attack-parry-pierce-modify | 560 | int |  |
| record | attack-perfect-parry-damage-modify | 560 | int |  |
| record | attack-pierce-base-percent | 160 | int |  |
| record | attack-pierce-modify | 560 | int |  |
| record | attack-pierce-value | 560 | int |  |
| record | attack-power-creature-max | 560 | int |  |
| record | attack-power-creature-min | 560 | int |  |
| record | attack-power-equip-max | 560 | int |  |
| record | attack-power-equip-min | 560 | int |  |
| record | attack-precise-creature-value | 160 | int |  |
| record | attack-stiff-duration-base-percent | 560 | int |  |
| record | attack-stiff-duration-value | 560 | int |  |
| record | attack-stiff-duration-value-modify | 560 | int |  |
| record | cast-duration-base-percent | 560 | int |  |
| record | cast-duration-value | 560 | int |  |
| record | cast-duration-value-modify | 560 | int |  |
| record | defend-abnormal-dodge-base-percent | 160 | int |  |
| record | defend-abnormal-dodge-value | 160 | int |  |
| record | defend-bounce-percent | 560 | int |  |
| record | defend-counter-reduce-modify | 560 | int |  |
| record | defend-counter-reduce-percent | 560 | int |  |
| record | defend-critical-base-percent | 560 | int |  |
| record | defend-critical-damage-modify | 560 | int |  |
| record | defend-critical-damage-percent | 160 | int |  |
| record | defend-critical-value | 560 | int |  |
| record | defend-critical-value-modify | 560 | int |  |
| record | defend-damage-modify-diff | 560 | int |  |
| record | defend-damage-modify-percent | 560 | int |  |
| record | defend-dodge-base-percent | 560 | int |  |
| record | defend-dodge-value | 560 | int |  |
| record | defend-dodge-value-modify | 560 | int |  |
| record | defend-force-damage-reduce-percent | 560 | int |  |
| record | defend-force-value-modify | 560 | int |  |
| record | defend-immune-base-percent | 560 | int |  |
| record | defend-miss-base-percent | 560 | int |  |
| record | defend-parry-base-percent | 560 | int |  |
| record | defend-parry-reduce-diff | 560 | int |  |
| record | defend-parry-reduce-modify | 560 | int |  |
| record | defend-parry-reduce-percent | 560 | int |  |
| record | defend-parry-value | 560 | int |  |
| record | defend-parry-value-modify | 560 | int |  |
| record | defend-perfect-parry-base-percent | 560 | int |  |
| record | defend-perfect-parry-reduce-modify | 560 | int |  |
| record | defend-perfect-parry-reduce-percent | 560 | int |  |
| record | defend-physical-damage-reduce-percent | 560 | int |  |
| record | defend-physical-value-modify | 560 | int |  |
| record | defend-power-creature-value | 560 | int |  |
| record | defend-power-equip-value | 560 | int |  |
| record | defend-resist-power-creature-value | 560 | int |  |
| record | defend-resist-power-equip-value | 560 | int |  |
| record | defend-stiff-duration-base-percent | 560 | int |  |
| record | defend-stiff-duration-value | 560 | int |  |
| record | defend-stiff-duration-value-modify | 560 | int |  |
| record | defend-strength-creature-value | 160 | int |  |
| record | heal-power-base-percent | 560 | int |  |
| record | heal-power-modify | 560 | int |  |
| record | hp-regen | 560 | int |  |
| record | hp-regen-combat | 560 | int |  |
| record | is-duelbot-summoned | 560 | enum-like string ❓ | n, y |
| record | job | 560 | string |  |
| record | mastery-level | 560 | int |  |
| record | max-hp | 560 | int |  |
| record | modify-cast-speed-percent | 560 | int |  |
| record | r-attack-abnormal-hit-modify | 160 | int |  |
| record | r-attack-aoe-pierce-modify | 160 | int |  |
| record | r-defend-abnormal-dodge-modify | 160 | int |  |
| record | r-support-power-modify | 160 | int |  |
| record | race | 560 | string |  |
| record | support-power-base-percent | 160 | int |  |
| record | support-power-value | 160 | int |  |

### SummonedPreset

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 78 | string |  |
| record | part-type | 78 | enum-like string ❓ | body, body-color, body-shape, decal, eye, eye-color, face |
| record | part-value | 78 | int |  |
| record | price | 77 | int |  |
| record | race | 78 | string |  |

### Terrain

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 164 | string |  |
| record | count-x | 164 | int |  |
| record | count-y | 164 | int |  |
| record | description | 164 | string |  |
| record | id | 164 | int |  |
| record | terrain-origin-x | 164 | int |  |
| record | terrain-origin-y | 164 | int |  |
| record | terrain-start-x | 164 | int |  |
| record | terrain-start-y | 164 | int |  |
| record | umap-name | 164 | string |  |

### Title

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |

### TrainSkillVersion

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | job | 14 | string |  |
| record | version | 14 | int |  |

### WeaponGemSlot

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | delete-slot-consume-item-1 | 5 | string |  |
| record | delete-slot-consume-item-stack-count-1 | 5 | int |  |
| record | level | 5 | int |  |

### WeaponGemTransformResult

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | id | 325 | int |  |
| record | item | 325 | string |  |
| record | weapon-gem-ability-1 | 325 | int |  |
| record | weapon-gem-ability-2 | 325 | int |  |
| record | weapon-gem-type | 325 | string |  |

### WeeklyTimeTable

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 55 | string |  |
| record | enable | 55 | enum-like string ❓ | n, y |
| record | end-minute | 10 | int |  |
| record | fri-end-hour-1 | 31 | int |  |
| record | fri-end-hour-2 | 5 | int |  |
| record | fri-start-hour-1 | 31 | int |  |
| record | fri-start-hour-2 | 5 | int |  |
| record | mon-end-hour-1 | 26 | int |  |
| record | mon-end-hour-2 | 5 | int |  |
| record | mon-start-hour-1 | 26 | int |  |
| record | mon-start-hour-2 | 5 | int |  |
| record | sat-end-hour-1 | 34 | int |  |
| record | sat-end-hour-2 | 5 | int |  |
| record | sat-start-hour-1 | 34 | int |  |
| record | sat-start-hour-2 | 5 | int |  |
| record | start-minute | 10 | int |  |
| record | sun-end-hour-1 | 30 | int |  |
| record | sun-end-hour-2 | 5 | int |  |
| record | sun-start-hour-1 | 30 | int |  |
| record | sun-start-hour-2 | 5 | int |  |
| record | thu-end-hour-1 | 27 | int |  |
| record | thu-end-hour-2 | 5 | int |  |
| record | thu-start-hour-1 | 27 | int |  |
| record | thu-start-hour-2 | 5 | int |  |
| record | tue-end-hour-1 | 29 | int |  |
| record | tue-end-hour-2 | 5 | int |  |
| record | tue-start-hour-1 | 29 | int |  |
| record | tue-start-hour-2 | 5 | int |  |
| record | wed-end-hour-1 | 26 | int |  |
| record | wed-end-hour-2 | 5 | int |  |
| record | wed-start-hour-1 | 26 | int |  |
| record | wed-start-hour-2 | 5 | int |  |

### ZoneArea

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 76936 | string |  |
| record | box | 76936 | string |  |
| record | id | 76936 | int |  |
| record | overlap | 76936 | enum-like string ❓ | n, y |
| record | zone | 76936 | int |  |

### ZoneAreaGroup

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | additional-area-id-1 | 1 | int |  |
| record | alias | 160 | string |  |
| record | area-id-max-1 | 159 | int |  |
| record | area-id-min-1 | 159 | int |  |
| record | id | 160 | int |  |
| record | zone | 160 | int |  |

### ZoneBaseCamp

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 68 | string |  |
| record | area | 68 | float |  |
| record | id | 68 | int |  |
| record | zone | 68 | int |  |

### ZoneCampfire

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 466 | string |  |
| record | campfire | 466 | string |  |
| record | id | 466 | int |  |
| record | position | 466 | string |  |
| record | zone | 466 | int |  |

### ZoneCellAttribute

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 4 | string |  |
| record | cell-attribute | 4 | enum-like string ❓ | safe |
| record | id | 4 | int |  |
| record | pos-count | 4 | int |  |
| record | pos-x-1 | 4 | int |  |
| record | pos-x-2 | 4 | int |  |
| record | pos-x-3 | 4 | int |  |
| record | pos-x-4 | 4 | int |  |
| record | pos-x-5 | 4 | int |  |
| record | pos-x-6 | 4 | int |  |
| record | pos-x-7 | 4 | int |  |
| record | pos-x-8 | 4 | int |  |
| record | pos-x-9 | 3 | int |  |
| record | pos-y-1 | 4 | int |  |
| record | pos-y-2 | 4 | int |  |
| record | pos-y-3 | 4 | int |  |
| record | pos-y-4 | 4 | int |  |
| record | pos-y-5 | 4 | int |  |
| record | pos-y-6 | 4 | int |  |
| record | pos-y-7 | 4 | int |  |
| record | pos-y-8 | 4 | int |  |
| record | pos-y-9 | 3 | int |  |
| record | pos-z-end | 4 | int |  |
| record | pos-z-start | 4 | int |  |
| record | zone | 4 | int |  |

### ZoneConvoy

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 39 | string |  |
| record | convoy-1 | 39 | string |  |
| record | convoy-2 | 6 | string |  |
| record | convoy-3 | 3 | string |  |
| record | convoy-4 | 3 | string |  |
| record | fail-radius | 39 | int |  |
| record | id | 39 | int |  |
| record | move-radius | 39 | int |  |
| record | spawn-1 | 39 | string |  |
| record | spawn-2 | 6 | string |  |
| record | spawn-3 | 3 | string |  |
| record | spawn-4 | 3 | string |  |
| record | zone | 39 | int |  |

### ZoneEnv

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | action-desc2 | 39 | string |  |
| record | action-name2 | 29 | string |  |
| record | action-point | 65 | string |  |
| record | action-radius | 51 | int |  |
| record | airdash-level | 10 | int |  |
| record | alias | 98 | string |  |
| record | cannot-manipulate-in-combat | 97 | enum-like string ❓ | n |
| record | cannot-manipulate-in-max-instant-heart-count | 97 | enum-like string ❓ | n |
| record | caster-animname | 40 | string |  |
| record | caster-picking-animname | 33 | string |  |
| record | caster-reaction-attach | 11 | string |  |
| record | caster-showname | 51 | string |  |
| record | category | 97 | enum-like string ❓ | collect, default, manipulate |
| record | dice-result-broadcast | 97 | enum-like string ❓ | n |
| record | duration | 10 | int |  |
| record | effect-1 | 14 | string |  |
| record | effect-target | 14 | enum-like string ❓ | pc |
| record | enable-create-solo-party | 1 | enum-like string ❓ | n |
| record | enable-toggle-interval | 14 | int |  |
| record | end-pos | 10 | string |  |
| record | env-animname | 22 | string |  |
| record | height | 15 | int |  |
| record | hide-close | 97 | enum-like string ❓ | n |
| record | hide-disable | 97 | enum-like string ❓ | n |
| record | hide-empty | 97 | enum-like string ❓ | n, y |
| record | id | 98 | int |  |
| record | init-enable | 97 | enum-like string ❓ | n, y |
| record | init-enable-duration | 97 | int |  |
| record | kismet | 10 | string |  |
| record | manipulate-by-item | 14 | string |  |
| record | manipulate-by-item-count | 51 | int |  |
| record | manipulate-duration | 51 | int |  |
| record | max-hp | 97 | int |  |
| record | name2 | 97 | string |  |
| record | phase-pc-spawn-1 | 14 | int |  |
| record | phase-pc-spawn-2 | 14 | int |  |
| record | phase-pc-spawn-3 | 14 | int |  |
| record | phase-pc-spawn-4 | 14 | int |  |
| record | phase-pc-spawn-5 | 14 | int |  |
| record | phase-pc-spawn-6 | 14 | int |  |
| record | phase-pc-spawn-7 | 14 | int |  |
| record | phase-zone-enter-cinematic-1 | 14 | string |  |
| record | phase-zone-enter-cinematic-2 | 14 | string |  |
| record | phase-zone-enter-cinematic-3 | 14 | string |  |
| record | phase-zone-enter-cinematic-4 | 14 | string |  |
| record | phase-zone-enter-cinematic-5 | 14 | string |  |
| record | phase-zone-enter-cinematic-6 | 14 | string |  |
| record | phase-zone-enter-cinematic-7 | 14 | string |  |
| record | phase-zone-leave-cinematic-1 | 14 | string |  |
| record | phase-zone-leave-cinematic-2 | 14 | string |  |
| record | phase-zone-leave-cinematic-3 | 14 | string |  |
| record | phase-zone-leave-cinematic-4 | 14 | string |  |
| record | phase-zone-leave-cinematic-5 | 14 | string |  |
| record | phase-zone-leave-cinematic-6 | 14 | string |  |
| record | phase-zone-leave-cinematic-7 | 14 | string |  |
| record | portal-type | 1 | enum-like string ❓ | door |
| record | region-point-1 | 46 | string |  |
| record | region-point-2 | 46 | string |  |
| record | region-point-3 | 46 | string |  |
| record | region-point-4 | 46 | string |  |
| record | required-fielditem | 3 | string |  |
| record | respawn-duration | 65 | int |  |
| record | reward | 36 | string |  |
| record | reward-faction-score-max | 97 | int |  |
| record | reward-faction-score-min | 97 | int |  |
| record | rollback | 97 | enum-like string ❓ | n, y |
| record | save-authorizer | 97 | enum-like string ❓ | n, y |
| record | script | 93 | string |  |
| record | show-quest-indicator | 97 | enum-like string ❓ | n |
| record | summon-end-pos | 10 | string |  |
| record | tooltip-option | 97 | enum-like string ❓ | n, y |
| record | transit-pc-spawn | 1 | int |  |
| record | transit-reentrance-pcspawn | 1 | enum-like string ❓ | n |
| record | transit-zone | 14 | int |  |
| record | type | 98 | enum-like string ❓ | airdash, button, chest, effect-region, fall-death, foot-switch, portal |
| record | zone | 98 | int |  |

### ZoneEnv2

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | accroach-duration | 3 | int |  |
| record | acquire-reward-to-inventory | 31 | enum-like string ❓ | y |
| record | airdash-level | 510 | int |  |
| record | alias | 11626 | string |  |
| record | attack-skill-1 | 69 | string |  |
| record | attack-skill3-1 | 69 | string |  |
| record | base-refining-duration | 3 | int |  |
| record | cannot-manipulate-in-combat | 10751 | enum-like string ❓ | n |
| record | cannot-manipulate-in-max-instant-heart-count | 10710 | enum-like string ❓ | n, y |
| record | consume-faction-score | 6 | enum-like string ❓ | n |
| record | consume-item-by-manipulate | 1 | enum-like string ❓ | y |
| record | dice-result-broadcast | 11408 | enum-like string ❓ | n, y |
| record | effect-1 | 395 | string |  |
| record | effect-target | 418 | enum-like string ❓ | all, npc, pc |
| record | enable-toggle-interval | 416 | int |  |
| record | fielditemdrop | 497 | string |  |
| record | fishing-field | 8 | string |  |
| record | give-faction-score-duration | 3 | int |  |
| record | hide-close | 11408 | enum-like string ❓ | n, y |
| record | hide-disable | 11393 | enum-like string ❓ | n, y |
| record | hide-empty | 11408 | string |  |
| record | init-enable | 11461 | string |  |
| record | init-enable-duration | 11440 | int |  |
| record | manipulate-by-item | 588 | string |  |
| record | manipulate-by-item-count | 3544 | int |  |
| record | manipulate-duration | 5090 | int |  |
| record | manipulate-notification-range | 12 | string |  |
| record | manipulated-effect | 119 | string |  |
| record | manipulation-drop-field-item | 6 | string |  |
| record | manipulation-required-field-item | 6 | string |  |
| record | max-hp | 11237 | int |  |
| record | occputation-effect | 3 | string |  |
| record | occputation-effect-distance | 3 | int |  |
| record | occupation-bonus-zone-score-per-sec | 9 | int |  |
| record | occupation-duration | 13 | int |  |
| record | occupation-zone-score | 3 | int |  |
| record | occurrence-event-in-respawn | 1 | enum-like string ❓ | y |
| record | phase-pc-spawn-1 | 2709 | int |  |
| record | phase-pc-spawn-10 | 5 | int |  |
| record | phase-pc-spawn-11 | 4 | int |  |
| record | phase-pc-spawn-12 | 5 | int |  |
| record | phase-pc-spawn-13 | 1 | int |  |
| record | phase-pc-spawn-14 | 1 | int |  |
| record | phase-pc-spawn-2 | 2663 | int |  |
| record | phase-pc-spawn-3 | 2683 | int |  |
| record | phase-pc-spawn-4 | 2684 | int |  |
| record | phase-pc-spawn-5 | 2681 | int |  |
| record | phase-pc-spawn-6 | 2666 | int |  |
| record | phase-pc-spawn-7 | 2666 | int |  |
| record | phase-pc-spawn-8 | 5 | int |  |
| record | phase-pc-spawn-9 | 5 | int |  |
| record | phase-zone-leave-social-1 | 13 | string |  |
| record | phase-zone-pc-spawn-1 | 31 | int |  |
| record | phase-zone-pc-spawn-2 | 31 | int |  |
| record | phase-zone-pc-spawn-3 | 31 | int |  |
| record | phase-zone-pc-spawn-4 | 31 | int |  |
| record | phase-zone-pc-spawn-5 | 31 | int |  |
| record | phase-zone-pc-spawn-6 | 31 | int |  |
| record | phase-zone-pc-spawn-7 | 31 | int |  |
| record | portal-type | 2232 | enum-like string ❓ | button, door |
| record | refine-faction-score | 3 | int |  |
| record | refined-effect | 6 | string |  |
| record | refined-effect-distance | 6 | int |  |
| record | refined-reward-item | 3 | string |  |
| record | refined-reward-item-base-count | 3 | int |  |
| record | required-activated-faction | 24 | int |  |
| record | required-activated-main-faction | 20 | enum-like string ❓ | y |
| record | required-faction-score | 9 | int |  |
| record | required-faction-score-max | 4 | int |  |
| record | required-fielditem | 134 | string |  |
| record | required-joined-main-faction | 2 | enum-like string ❓ | y |
| record | required-level | 7 | int |  |
| record | respawn-duration | 6556 | int |  |
| record | respawn-duration-in-refined | 3 | int |  |
| record | reward | 407 | string |  |
| record | reward-faction-score | 30 | int |  |
| record | rollback | 11237 | enum-like string ❓ | n, y |
| record | save-authorizer | 11408 | enum-like string ❓ | n, y |
| record | script | 8151 | string |  |
| record | social | 1 | string |  |
| record | transit-dungeon | 2 | string |  |
| record | transit-leave-social | 25 | string |  |
| record | transit-pc-spawn | 781 | int |  |
| record | transit-pc-spawn-in-arena | 312 | string |  |
| record | transit-reentrance-pcspawn | 2109 | enum-like string ❓ | n |
| record | transit-zone | 2846 | int |  |
| record | try-refining-duration | 3 | int |  |
| record | type | 11626 | string |  |

### ZoneEnv2Place

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | action-point | 10478 | string |  |
| record | action-radius | 7888 | int |  |
| record | alias | 11682 | string |  |
| record | duration | 487 | int |  |
| record | end-pos | 487 | string |  |
| record | height | 623 | int |  |
| record | manipulation-height-lower | 6825 | int |  |
| record | manipulation-height-upper | 6825 | int |  |
| record | manipulation-point | 6825 | string |  |
| record | manipulation-radius | 6825 | int |  |
| record | region-point-1 | 5105 | string |  |
| record | region-point-2 | 5105 | string |  |
| record | region-point-3 | 5105 | string |  |
| record | region-point-4 | 5105 | string |  |
| record | summon-end-pos | 487 | string |  |
| record | tool-zone-1 | 10813 | int |  |
| record | tool-zone-10 | 72 | int |  |
| record | tool-zone-11 | 72 | int |  |
| record | tool-zone-12 | 72 | int |  |
| record | tool-zone-2 | 3857 | int |  |
| record | tool-zone-3 | 1449 | int |  |
| record | tool-zone-4 | 569 | int |  |
| record | tool-zone-5 | 265 | int |  |
| record | tool-zone-6 | 204 | int |  |
| record | tool-zone-7 | 135 | int |  |
| record | tool-zone-8 | 128 | int |  |
| record | tool-zone-9 | 72 | int |  |
| record | type | 11682 | enum-like string ❓ | airdash, airdash-leave, button, chest, control-point, deck, effect-region, fall-death, fishing-point, foot-switch, gate, oceanic-region, portal, pot, refiner, wall |
| record | use-manipulation-point | 6825 | enum-like string ❓ | n, y |

### ZoneEnv2Spawn

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 11832 | string |  |
| record | change-state-env | 3 | string |  |
| record | env2 | 11825 | string |  |
| record | env2place | 11825 | string |  |
| record | id | 11832 | int |  |
| record | zone | 11832 | int |  |

### ZoneGatherSource

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |

### ZoneNpcRandomSpawn

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | act-sequence | 159 | string |  |
| record | alias | 159 | string |  |
| record | area-group | 159 | string |  |
| record | id | 159 | int |  |
| record | npc-1 | 159 | string |  |
| record | npc-2 | 101 | string |  |
| record | npc-3 | 91 | string |  |
| record | npc-4 | 65 | string |  |
| record | npc-count-1 | 159 | int |  |
| record | npc-count-2 | 101 | int |  |
| record | npc-count-3 | 91 | int |  |
| record | npc-count-4 | 65 | int |  |
| record | npc-move-distance-1 | 159 | int |  |
| record | npc-move-distance-2 | 101 | int |  |
| record | npc-move-distance-3 | 91 | int |  |
| record | npc-move-distance-4 | 65 | int |  |
| record | respawn | 159 | enum-like string ❓ | y |
| record | respawn-delay-max | 159 | int |  |
| record | respawn-delay-min | 159 | int |  |
| record | zone | 159 | int |  |

### ZoneNpcSpawn

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | act-area-1 | 51281 | string |  |
| record | act-area-2 | 345 | string |  |
| record | act-area-3 | 60 | string |  |
| record | act-area-4 | 40 | string |  |
| record | act-detect-1 | 67935 | string |  |
| record | act-detect-2 | 3 | string |  |
| record | act-pathway-1 | 12905 | string |  |
| record | act-pathway-2 | 2244 | string |  |
| record | act-pathway-3 | 581 | string |  |
| record | act-pathway-4 | 256 | string |  |
| record | act-sequence | 68454 | string |  |
| record | act-social-1 | 61603 | string |  |
| record | act-social-2 | 61293 | string |  |
| record | act-social-3 | 537 | string |  |
| record | alias | 68455 | string |  |
| record | area | 68455 | string |  |
| record | despawn-delay | 5877 | int |  |
| record | id | 68455 | int |  |
| record | init-spawn | 68455 | string |  |
| record | npc | 68455 | string |  |
| record | respawn | 68107 | string |  |
| record | respawn-delay-max | 66419 | int |  |
| record | respawn-delay-min | 66419 | int |  |
| record | yaw-max | 68452 | string |  |
| record | yaw-min | 68452 | string |  |
| record | zone | 68455 | int |  |

### ZoneNpcSpawnEvent

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 1 | string |  |
| record | id | 1 | int |  |
| record | kill-count | 1 | int |  |
| record | npc-spawn | 1 | string |  |
| record | reset-kill-count-zone-mode | 1 | int |  |
| record | target-npc-1 | 1 | string |  |
| record | target-npc-2 | 1 | string |  |
| record | target-npc-3 | 1 | string |  |
| record | target-npc-4 | 1 | string |  |
| record | target-npc-5 | 1 | string |  |
| record | target-npc-6 | 1 | string |  |
| record | type | 1 | string |  |
| record | zone | 1 | int |  |

### ZoneNpcSquad

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 10795 | string |  |
| record | combat-pattern | 51 | string |  |
| record | combat-start-hate-share | 6240 | string |  |
| record | enable-damage-transfer-to-leader | 13 | enum-like string ❓ | y |
| record | group-move | 990 | string |  |
| record | hate-share-distance | 10795 | int |  |
| record | id | 10795 | int |  |
| record | leader | 3593 | int |  |
| record | npc-bleeding-one-hate-share | 59 | int |  |
| record | npc-bleeding-three-hate-share | 21 | int |  |
| record | npc-bleeding-two-hate-share | 30 | string |  |
| record | spawn-1 | 10795 | string |  |
| record | spawn-10 | 786 | string |  |
| record | spawn-11 | 63 | string |  |
| record | spawn-12 | 56 | string |  |
| record | spawn-13 | 35 | string |  |
| record | spawn-14 | 34 | string |  |
| record | spawn-15 | 30 | string |  |
| record | spawn-16 | 1 | string |  |
| record | spawn-17 | 1 | string |  |
| record | spawn-2 | 8946 | string |  |
| record | spawn-3 | 6078 | string |  |
| record | spawn-4 | 4148 | string |  |
| record | spawn-5 | 3029 | string |  |
| record | spawn-6 | 2119 | string |  |
| record | spawn-7 | 1584 | string |  |
| record | spawn-8 | 1313 | string |  |
| record | spawn-9 | 1027 | string |  |
| record | zone | 10795 | int |  |

### ZonePathway

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 14442 | string |  |
| record | id | 77786 | int |  |
| record | move-type | 77786 | enum-like string ❓ | run, sprint, walk, warp |
| record | pos | 77786 | string |  |
| record | sync | 77786 | int |  |
| record | warp-social | 617 | string |  |
| record | waypoint | 77786 | int |  |
| record | zone | 77786 | int |  |

### ZonePcSpawn

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 2948 | string |  |
| record | id | 2951 | int |  |
| record | pos | 2951 | string |  |
| record | summon-pos | 75 | string |  |
| record | yaw | 2951 | int |  |
| record | zone | 2951 | int |  |

### ZoneRespawn

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | activated-faction-1 | 20 | string |  |
| record | alias | 1652 | string |  |
| record | area | 1652 | float |  |
| record | id | 1652 | int |  |
| record | zone | 1652 | int |  |

### ZoneRule

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | allow-invite-party-in-arena | 17 | enum-like string ❓ | n, y |
| record | apply-content-ban-id | 5 | string |  |
| record | enable-co-ownership-pouch | 17 | enum-like string ❓ | n, y |
| record | enable-infinite-hyper-energy | 17 | enum-like string ❓ | n, y |
| record | enable-party-auction | 7 | enum-like string ❓ | n |
| record | enable-pvp | 17 | enum-like string ❓ | n, y |
| record | enable-reset-combat-mode | 17 | enum-like string ❓ | n, y |
| record | infinite-durability | 7 | enum-like string ❓ | y |
| record | is-immediately-dead | 7 | enum-like string ❓ | n, y |
| record | max-persistent-channel-count | 2 | int |  |
| record | required-available-weekly-time | 2 | string |  |
| record | required_inactivated_main_faction | 17 | enum-like string ❓ | n |
| record | reset-combat-mode-recycle-duration | 4 | int |  |
| record | town | 8 | enum-like string ❓ | n |
| record | zone | 17 | int |  |

### ZoneTeleportPosition

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 156 | string |  |
| record | position | 156 | string |  |
| record | yaw | 156 | int |  |
| record | zone | 156 | int |  |

### ZoneTeleportSwitch

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | activate-radius | 190 | int |  |
| record | activate-teleport | 190 | string |  |
| record | alias | 190 | string |  |
| record | description | 180 | string |  |
| record | id | 190 | int |  |
| record | position | 190 | string |  |
| record | zone | 190 | int |  |

### ZoneTriggerEventCond

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | acquisition-score | 4 | int |  |
| record | acquisition-score-type | 4 | string |  |
| record | alias | 29 | string |  |
| record | event-day-of-week | 6 | enum-like string ❓ | entire-week, thu-fri, weekend |
| record | event-hour-1 | 6 | int |  |
| record | event-hour-2 | 4 | int |  |
| record | event-hour-3 | 3 | int |  |
| record | event-hour-4 | 2 | int |  |
| record | event-hour-5 | 2 | int |  |
| record | event-hour-6 | 2 | int |  |
| record | event-hour-7 | 2 | int |  |
| record | event-hour-8 | 2 | int |  |
| record | event-minute-1 | 6 | int |  |
| record | event-minute-2 | 4 | int |  |
| record | event-minute-3 | 3 | int |  |
| record | event-minute-4 | 2 | int |  |
| record | event-minute-5 | 2 | int |  |
| record | event-minute-6 | 2 | int |  |
| record | event-minute-7 | 2 | int |  |
| record | event-minute-8 | 2 | int |  |
| record | goal-score | 4 | int |  |
| record | goal-score-type | 4 | enum-like string ❓ | all |
| record | kill-count | 10 | int |  |
| record | op | 4 | enum-like string ❓ | ge |
| record | survived-count | 4 | int |  |
| record | target-npc-1 | 10 | string |  |
| record | target-npc-2 | 7 | string |  |
| record | target-npc-3 | 3 | string |  |
| record | target-npc-spawn-1 | 4 | string |  |
| record | target-npc-spawn-2 | 4 | string |  |
| record | type | 29 | string |  |
| record | win-score-type | 4 | string |  |

### ZoneTriggerEventStage

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 16 | string |  |
| record | all-channel | 1 | enum-like string ❓ | y |
| record | branch-id | 16 | int |  |
| record | broadcast-context | 11 | enum-like string ❓ | boss-challenge-attraction-round, timeout-time |
| record | channel-entry-max-level | 4 | int |  |
| record | channel-entry-min-level | 4 | int |  |
| record | clear-score | 2 | enum-like string ❓ | y |
| record | fail-cond-1 | 5 | string |  |
| record | fail-cond-2 | 2 | string |  |
| record | fail-cond-3 | 2 | string |  |
| record | fail-stage-add-timeout-duration-minute | 3 | int |  |
| record | fail-stage-setup | 5 | string |  |
| record | max-channel | 2 | int |  |
| record | next-cond-1 | 13 | string |  |
| record | next-cond-2 | 3 | string |  |
| record | next-cond-branch-id-1 | 14 | int |  |
| record | next-cond-branch-id-2 | 3 | int |  |
| record | next-stage-setup-1 | 9 | string |  |
| record | next-stage-setup-2 | 4 | string |  |
| record | timeout-duration-minute | 8 | int |  |
| record | timeout-duration-second | 1 | int |  |
| record | total-duration-minute-1 | 2 | int |  |
| record | type | 16 | enum-like string ❓ | stage, stage-stand-by-instant-zone, stage-stand-by-persistant-zone |
| record | zone | 16 | int |  |
| record | zone-mode | 16 | int |  |
| record | zone-mode-set-id | 16 | int |  |

### account-post-charge

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 2 | string |  |
| record | charge-item-1 | 2 | string |  |
| record | charge-item-amount-1 | 2 | int |  |

### achievement

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 1558 | string |  |
| record | complete-from-client | 3 | enum-like string ❓ | y |
| record | completed-effect | 34 | string |  |
| record | current-step-score | 1558 | int |  |
| record | deprecated | 72 | enum-like string ❓ | y |
| record | id | 1558 | int |  |
| record | job | 1558 | string |  |
| record | register-ref-1 | 1555 | string |  |
| record | register-ref-2 | 87 | string |  |
| record | register-ref-3 | 71 | string |  |
| record | register-ref-4 | 32 | string |  |
| record | register-ref-5 | 19 | string |  |
| record | register-type-1 | 1555 | enum-like string ❓ | above, bits-on |
| record | register-type-2 | 87 | enum-like string ❓ | above, bits-on |
| record | register-type-3 | 71 | enum-like string ❓ | above, bits-on |
| record | register-type-4 | 32 | enum-like string ❓ | above, bits-on |
| record | register-type-5 | 19 | enum-like string ❓ | above, bits-on |
| record | register-value-1 | 1555 | int |  |
| record | register-value-2 | 87 | int |  |
| record | register-value-3 | 71 | int |  |
| record | register-value-4 | 32 | int |  |
| record | register-value-5 | 19 | int |  |
| record | step | 1558 | int |  |
| record | step-complete-reward-game-cash | 11 | int |  |
| record | step-complete-reward-item-1 | 96 | string |  |
| record | step-complete-reward-item-2 | 96 | string |  |
| record | step-complete-reward-item-count-1 | 96 | int |  |
| record | step-complete-reward-item-count-2 | 96 | int |  |
| record | step-complete-reward-skill-build-up-point | 158 | int |  |

### achievement-register

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 1067 | string |  |
| record | boss-npc | 202 | string |  |
| record | env2-1 | 15 | string |  |
| record | env2-2 | 9 | string |  |
| record | env2-3 | 7 | string |  |
| record | env2-4 | 3 | string |  |
| record | env2-5 | 3 | string |  |
| record | env2-6 | 1 | string |  |
| record | env2-7 | 1 | string |  |
| record | env2-8 | 1 | string |  |
| record | env2-state-1 | 2 | string |  |
| record | env2-state-2 | 2 | string |  |
| record | env2-state-3 | 2 | string |  |
| record | equip-gem-only | 1 | enum-like string ❓ | y |
| record | for-newbie-care | 15 | enum-like string ❓ | y |
| record | gadget | 16 | string |  |
| record | id | 1067 | int |  |
| record | item | 1 | string |  |
| record | item-brand | 34 | string |  |
| record | job | 1067 | string |  |
| record | max-value | 1067 | int |  |
| record | npc | 16 | string |  |
| record | npc-1 | 84 | string |  |
| record | npc-2 | 21 | string |  |
| record | npc-3 | 5 | string |  |
| record | npc-4 | 4 | string |  |
| record | npc-5 | 1 | string |  |
| record | persistant | 1055 | enum-like string ❓ | y |
| record | quest | 53 | string |  |
| record | seed-item-brand | 4 | string |  |
| record | skill-training-room-group | 96 | int |  |
| record | team-broadcast | 34 | enum-like string ❓ | y |
| record | tendency-id | 6 | int |  |
| record | type | 1067 | string |  |
| record | version | 1067 | int |  |

### act-sequence

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| act-sequence | alias | 9091 | string |  |
| act-sequence | detect | 1900 | string |  |
| act-sequence | detect-1 | 2 | enum-like string ❓ | none |
| act-sequence | indexed-detect | 3599 | int |  |
| act-sequence | type | 9091 | enum-like string ❓ | act, peace |
| act-sequence/action | area | 224 | string |  |
| act-sequence/action | detect | 48 | string |  |
| act-sequence/action | druation | 1 | int |  |
| act-sequence/action | duration | 11063 | string |  |
| act-sequence/action | duratoin | 1 | int |  |
| act-sequence/action | enter-prob | 556 | int |  |
| act-sequence/action | enter_prob | 479 | int |  |
| act-sequence/action | hide | 233 | enum-like string ❓ | n, y |
| act-sequence/action | max-count | 187 | int |  |
| act-sequence/action | max-idle-sec | 224 | int |  |
| act-sequence/action | max-move-count | 226 | int |  |
| act-sequence/action | min-count | 187 | int |  |
| act-sequence/action | min-idle-sec | 224 | int |  |
| act-sequence/action | min-move-count | 226 | int |  |
| act-sequence/action | pathway | 4400 | string |  |
| act-sequence/action | prob | 58 | int |  |
| act-sequence/action | repeat | 4 | int |  |
| act-sequence/action | respawn | 957 | enum-like string ❓ | n, y |
| act-sequence/action | social | 6589 | string |  |
| act-sequence/action | step | 2489 | int |  |
| act-sequence/action | target | 26 | string |  |
| act-sequence/action | type | 27393 | enum-like string ❓ | despawn, hide, indexed-movearound, indexed-pathway, indexed-social, loop, movearound, movearound-for-fieldbossspawn, movearound-for-randomspawn, pathway, pause, select, social, stay |
| act-sequence/action/action | area | 12 | int |  |
| act-sequence/action/action | detect | 132 | enum-like string ❓ | none |
| act-sequence/action/action | duration | 1637 | int |  |
| act-sequence/action/action | enter_prob | 15 | int |  |
| act-sequence/action/action | max-count | 12 | int |  |
| act-sequence/action/action | max-idle-sec | 12 | int |  |
| act-sequence/action/action | max-move-count | 12 | int |  |
| act-sequence/action/action | min-count | 12 | int |  |
| act-sequence/action/action | min-idle-sec | 12 | int |  |
| act-sequence/action/action | min-move-count | 12 | int |  |
| act-sequence/action/action | pathway | 71 | string |  |
| act-sequence/action/action | prob | 3295 | int |  |
| act-sequence/action/action | social | 968 | string |  |
| act-sequence/action/action | type | 3580 | enum-like string ❓ | indexed-movearound, indexed-pathway, indexed-social, loop, pathway, select, social, stay |
| act-sequence/action/action/action | area | 50 | int |  |
| act-sequence/action/action/action | duration | 667 | int |  |
| act-sequence/action/action/action | max-count | 3 | int |  |
| act-sequence/action/action/action | max-idle-sec | 50 | int |  |
| act-sequence/action/action/action | max-move-count | 50 | int |  |
| act-sequence/action/action/action | min-count | 3 | int |  |
| act-sequence/action/action/action | min-idle-sec | 50 | int |  |
| act-sequence/action/action/action | min-move-count | 50 | int |  |
| act-sequence/action/action/action | pathway | 6 | int |  |
| act-sequence/action/action/action | prob | 44 | int |  |
| act-sequence/action/action/action | social | 1220 | string |  |
| act-sequence/action/action/action | step | 1 | int |  |
| act-sequence/action/action/action | type | 1969 | enum-like string ❓ | indexed-movearound, indexed-pathway, indexed-social, loop, pause, social, stay |
| act-sequence/action/action/action/action | duration | 23 | int |  |
| act-sequence/action/action/action/action | social | 32 | string |  |
| act-sequence/action/action/action/action | type | 55 | enum-like string ❓ | indexed-social, social, stay |
| act-sequence/action/random-social | max-delay-sec | 3 | int |  |
| act-sequence/action/random-social | min-delay-sec | 22 | int |  |
| act-sequence/action/random-social | social-prob | 1 | int |  |
| act-sequence/action/random-social | social-prop | 22 | int |  |
| act-sequence/action/random-social/action | prob | 2 | int |  |
| act-sequence/action/random-social/action | social | 2 | int |  |
| act-sequence/action/random-social/action | type | 2 | enum-like string ❓ | indexed-social |
| act-sequence/action/random-social/indexed-social | prob | 16 | int |  |
| act-sequence/action/random-social/indexed-social | social | 18 | int |  |
| act-sequence/action/random-social/social | prob | 71 | int |  |
| act-sequence/action/random-social/social | social | 71 | string |  |
| act-sequence/action/social | prob | 2 | int |  |
| act-sequence/action/social | social | 2 | string |  |
| act-sequence/indexed-social | social | 8 | int |  |
| act-sequence/reaction | duration | 1 | int |  |
| act-sequence/reaction | type | 1 | enum-like string ❓ | stay |

### attraction-group

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 74 | string |  |

### attraction-reward

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | id | 80 | int |  |
| record | reward-contents-type | 80 | enum-like string ❓ | battle-royal-field, battle-royal-field-for-party |
| record | reward-exp | 80 | int |  |
| record | reward-item-1 | 75 | string |  |
| record | reward-item-2 | 55 | string |  |
| record | reward-item-3 | 9 | string |  |
| record | reward-item-count-1 | 75 | int |  |
| record | reward-item-count-2 | 55 | int |  |
| record | reward-item-count-3 | 9 | int |  |

### badge-set-item

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 384 | string |  |
| record | effect-1 | 384 | string |  |
| record | effect-2 | 313 | string |  |
| record | score | 384 | int |  |

### badge-synthesis-cost

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 512 | string |  |
| record | money-cost | 512 | int |  |
| record | required-item-1 | 512 | string |  |
| record | required-item-2 | 512 | string |  |
| record | required-item-count-1 | 512 | int |  |
| record | required-item-count-2 | 512 | int |  |
| record | score | 512 | int |  |

### battle-field-grade

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | grade | 15 | int |  |
| record | point-bonus-percent | 15 | int |  |
| record | score | 15 | int |  |
| record | season-type | 15 | enum-like string ❓ | party-battle-capture-the-flag, party-battle-lead-the-ball, party-battle-occupation-war |

### battle-royal-field

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 1 | string |  |
| record | available-party-mode-matching-weekly-time | 1 | string |  |
| record | change-state-env-by-start | 1 | string |  |
| record | enable-infinite-hyper-energy | 1 | enum-like string ❓ | y |
| record | enter-pc-spawn | 1 | string |  |
| record | event-alias | 1 | string |  |
| record | finish-pc-respawn | 1 | string |  |
| record | group | 1 | string |  |
| record | id | 1 | int |  |
| record | pc-kill-effect-1 | 1 | string |  |
| record | public-raid | 1 | string |  |
| record | required-level | 1 | int |  |
| record | required-mastery-level | 1 | int |  |
| record | required-member-count-party-mode-matching | 1 | int |  |
| record | required-party-count-party-mode-matching | 1 | int |  |
| record | stand-by-duration | 1 | int |  |
| record | waiting-duration-second | 1 | int |  |
| record | waiting-time-before-combat | 1 | int |  |
| record | waiting-time-before-start | 1 | int |  |
| record | zone | 1 | int |  |

### battle-royal-field-effect-pouch

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 25 | string |  |
| record | category | 25 | enum-like string ❓ | party-mode, single-mode |
| record | effect-1 | 25 | string |  |
| record | effect-2 | 25 | string |  |
| record | effect-3 | 21 | string |  |
| record | grade | 25 | int |  |
| record | group | 25 | int |  |
| record | id | 25 | int |  |
| record | max-count | 25 | int |  |
| record | min-count | 25 | int |  |

### battle-royal-field-event

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 1 | string |  |
| record | center-range-radius-1 | 1 | int |  |
| record | center-range-radius-2 | 1 | int |  |
| record | center-range-radius-3 | 1 | int |  |
| record | center-range-radius-4 | 1 | int |  |
| record | center-range-radius-5 | 1 | int |  |
| record | center-range-radius-6 | 1 | int |  |
| record | center-range-radius-7 | 1 | int |  |
| record | default-event-area-radius | 1 | int |  |
| record | duration-1 | 1 | int |  |
| record | duration-2 | 1 | int |  |
| record | duration-3 | 1 | int |  |
| record | duration-4 | 1 | int |  |
| record | duration-5 | 1 | int |  |
| record | duration-6 | 1 | int |  |
| record | duration-7 | 1 | int |  |
| record | id | 1 | int |  |
| record | map-center-x | 1 | int |  |
| record | map-center-y | 1 | int |  |
| record | notice-time-1 | 1 | int |  |
| record | notice-time-2 | 1 | int |  |
| record | notice-time-3 | 1 | int |  |
| record | notice-time-4 | 1 | int |  |
| record | notice-time-5 | 1 | int |  |
| record | notice-time-6 | 1 | int |  |
| record | notice-time-7 | 1 | int |  |
| record | start-time-1 | 1 | int |  |
| record | start-time-2 | 1 | int |  |
| record | start-time-3 | 1 | int |  |
| record | start-time-4 | 1 | int |  |
| record | start-time-5 | 1 | int |  |
| record | start-time-6 | 1 | int |  |
| record | start-time-7 | 1 | int |  |
| record | toxic-event-area-radius-1 | 1 | int |  |
| record | toxic-event-area-radius-2 | 1 | int |  |
| record | toxic-event-area-radius-3 | 1 | int |  |
| record | toxic-event-area-radius-4 | 1 | int |  |
| record | toxic-event-area-radius-5 | 1 | int |  |
| record | toxic-event-area-radius-6 | 1 | int |  |
| record | toxic-event-area-radius-7 | 1 | int |  |
| record | toxic-event-effect-1 | 1 | string |  |
| record | toxic-event-effect-2 | 1 | string |  |
| record | toxic-event-effect-3 | 1 | string |  |
| record | toxic-event-effect-4 | 1 | string |  |
| record | toxic-event-effect-5 | 1 | string |  |
| record | toxic-event-effect-6 | 1 | string |  |
| record | toxic-event-effect-7 | 1 | string |  |
| record | toxic-event-effect2-1 | 1 | string |  |
| record | toxic-event-effect2-2 | 1 | string |  |
| record | toxic-event-effect2-3 | 1 | string |  |
| record | toxic-event-effect2-4 | 1 | string |  |
| record | toxic-event-effect2-5 | 1 | string |  |
| record | toxic-event-effect2-6 | 1 | string |  |
| record | toxic-event-effect2-7 | 1 | string |  |

### battle-royal-field-item-spawn-position

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 1400 | string |  |
| record | id | 1400 | int |  |
| record | pos-grade | 1400 | int |  |
| record | spawn-pos-x | 1400 | int |  |
| record | spawn-pos-y | 1400 | int |  |
| record | spawn-pos-z | 1400 | int |  |

### battle-royal-field-score-conversion

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | conversion-value-1 | 6 | int |  |
| record | conversion-value-10 | 6 | int |  |
| record | conversion-value-11 | 6 | int |  |
| record | conversion-value-12 | 6 | int |  |
| record | conversion-value-13 | 6 | int |  |
| record | conversion-value-14 | 6 | int |  |
| record | conversion-value-15 | 6 | int |  |
| record | conversion-value-16 | 6 | int |  |
| record | conversion-value-17 | 6 | int |  |
| record | conversion-value-18 | 6 | int |  |
| record | conversion-value-19 | 6 | int |  |
| record | conversion-value-2 | 6 | int |  |
| record | conversion-value-20 | 6 | int |  |
| record | conversion-value-21 | 3 | int |  |
| record | conversion-value-22 | 3 | int |  |
| record | conversion-value-23 | 3 | int |  |
| record | conversion-value-24 | 3 | int |  |
| record | conversion-value-25 | 3 | int |  |
| record | conversion-value-26 | 3 | int |  |
| record | conversion-value-27 | 3 | int |  |
| record | conversion-value-28 | 3 | int |  |
| record | conversion-value-29 | 3 | int |  |
| record | conversion-value-3 | 6 | int |  |
| record | conversion-value-30 | 3 | int |  |
| record | conversion-value-31 | 3 | int |  |
| record | conversion-value-32 | 3 | int |  |
| record | conversion-value-33 | 3 | int |  |
| record | conversion-value-34 | 3 | int |  |
| record | conversion-value-35 | 3 | int |  |
| record | conversion-value-36 | 3 | int |  |
| record | conversion-value-37 | 3 | int |  |
| record | conversion-value-38 | 3 | int |  |
| record | conversion-value-39 | 3 | int |  |
| record | conversion-value-4 | 6 | int |  |
| record | conversion-value-40 | 3 | int |  |
| record | conversion-value-41 | 3 | int |  |
| record | conversion-value-42 | 3 | int |  |
| record | conversion-value-43 | 3 | int |  |
| record | conversion-value-44 | 3 | int |  |
| record | conversion-value-45 | 3 | int |  |
| record | conversion-value-46 | 3 | int |  |
| record | conversion-value-47 | 3 | int |  |
| record | conversion-value-48 | 3 | int |  |
| record | conversion-value-49 | 3 | int |  |
| record | conversion-value-5 | 6 | int |  |
| record | conversion-value-50 | 3 | int |  |
| record | conversion-value-51 | 3 | int |  |
| record | conversion-value-52 | 3 | int |  |
| record | conversion-value-53 | 3 | int |  |
| record | conversion-value-54 | 3 | int |  |
| record | conversion-value-55 | 3 | int |  |
| record | conversion-value-56 | 3 | int |  |
| record | conversion-value-57 | 3 | int |  |
| record | conversion-value-58 | 3 | int |  |
| record | conversion-value-59 | 3 | int |  |
| record | conversion-value-6 | 6 | int |  |
| record | conversion-value-60 | 3 | int |  |
| record | conversion-value-7 | 6 | int |  |
| record | conversion-value-8 | 6 | int |  |
| record | conversion-value-9 | 6 | int |  |
| record | game-mode | 6 | enum-like string ❓ | battle-royal-field, battle-royal-field-party |
| record | range-top-1 | 6 | int |  |
| record | range-top-10 | 6 | int |  |
| record | range-top-11 | 6 | int |  |
| record | range-top-12 | 6 | int |  |
| record | range-top-13 | 6 | int |  |
| record | range-top-14 | 6 | int |  |
| record | range-top-15 | 6 | int |  |
| record | range-top-16 | 6 | int |  |
| record | range-top-17 | 6 | int |  |
| record | range-top-18 | 6 | int |  |
| record | range-top-19 | 6 | int |  |
| record | range-top-2 | 6 | int |  |
| record | range-top-20 | 6 | int |  |
| record | range-top-21 | 3 | int |  |
| record | range-top-22 | 3 | int |  |
| record | range-top-23 | 3 | int |  |
| record | range-top-24 | 3 | int |  |
| record | range-top-25 | 3 | int |  |
| record | range-top-26 | 3 | int |  |
| record | range-top-27 | 3 | int |  |
| record | range-top-28 | 3 | int |  |
| record | range-top-29 | 3 | int |  |
| record | range-top-3 | 6 | int |  |
| record | range-top-30 | 3 | int |  |
| record | range-top-31 | 3 | int |  |
| record | range-top-32 | 3 | int |  |
| record | range-top-33 | 3 | int |  |
| record | range-top-34 | 3 | int |  |
| record | range-top-35 | 3 | int |  |
| record | range-top-36 | 3 | int |  |
| record | range-top-37 | 3 | int |  |
| record | range-top-38 | 3 | int |  |
| record | range-top-39 | 3 | int |  |
| record | range-top-4 | 6 | int |  |
| record | range-top-40 | 3 | int |  |
| record | range-top-41 | 3 | int |  |
| record | range-top-42 | 3 | int |  |
| record | range-top-43 | 3 | int |  |
| record | range-top-44 | 3 | int |  |
| record | range-top-45 | 3 | int |  |
| record | range-top-46 | 3 | int |  |
| record | range-top-47 | 3 | int |  |
| record | range-top-48 | 3 | int |  |
| record | range-top-49 | 3 | int |  |
| record | range-top-5 | 6 | int |  |
| record | range-top-50 | 3 | int |  |
| record | range-top-51 | 3 | int |  |
| record | range-top-52 | 3 | int |  |
| record | range-top-53 | 3 | int |  |
| record | range-top-54 | 3 | int |  |
| record | range-top-55 | 3 | int |  |
| record | range-top-56 | 3 | int |  |
| record | range-top-57 | 3 | int |  |
| record | range-top-58 | 3 | int |  |
| record | range-top-59 | 3 | int |  |
| record | range-top-6 | 6 | int |  |
| record | range-top-60 | 3 | int |  |
| record | range-top-7 | 6 | int |  |
| record | range-top-8 | 6 | int |  |
| record | range-top-9 | 6 | int |  |
| record | score-type | 6 | enum-like string ❓ | combat-score, kill-score, ranking-score |

### board-gacha

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | cost-item | 1 | string |  |
| record | cost-item-count | 1 | int |  |
| record | instant-payment-enable | 1 | enum-like string ❓ | n |
| record | participation-count-per-character | 1 | int |  |
| record | result-top-rank-count | 1 | int |  |
| record | round-duration | 1 | int |  |
| record | select-position-enable | 1 | enum-like string ❓ | n |
| record | square-matrix-n | 1 | int |  |
| record | top-rank-history-count | 1 | int |  |

### board-gacha-reward

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | participation-count | 5 | int |  |
| record | rank-max | 12 | int |  |
| record | rank-min | 12 | int |  |
| record | reward-item | 18 | string |  |
| record | reward-item-count | 18 | int |  |
| record | reward-type | 18 | enum-like string ❓ | line, participation, rank |

### boast-event

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 57 | string |  |
| record | boast-achievement | 4 | string |  |
| record | boast-from-random-store | 29 | string |  |
| record | boast-from-recipe | 24 | string |  |
| record | boast-item | 53 | string |  |
| record | broadcast | 57 | enum-like string ❓ | world |
| record | id | 57 | int |  |
| record | type | 57 | enum-like string ❓ | buy-random-store-item, complete-achievement, transform-item |

### boss-challenge

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 1 | string |  |
| record | effect-npc-group1-1 | 1 | string |  |
| record | effect-npc-group1-2 | 1 | string |  |
| record | effect-npc-group1-3 | 1 | string |  |
| record | effect-npc-group2-1 | 1 | string |  |
| record | effect-npc-group2-2 | 1 | string |  |
| record | effect-npc-group2-3 | 1 | string |  |
| record | effect-npc-group3-1 | 1 | string |  |
| record | enable-infinite-hyper-energy | 1 | enum-like string ❓ | y |
| record | group | 1 | string |  |
| record | id | 1 | int |  |
| record | require-score | 1 | int |  |
| record | required-level | 1 | int |  |
| record | zone | 1 | int |  |

### boss-challenge-round

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 6 | string |  |
| record | boss-npc | 6 | string |  |
| record | boss-spawn-delay | 6 | int |  |
| record | effect-by-npc-killed-group1-1 | 6 | string |  |
| record | effect-by-npc-killed-group2-1 | 6 | string |  |
| record | effect-by-npc-killed-group3-1 | 6 | string |  |
| record | id | 6 | int |  |
| record | prob-weight-by-npc-killed-group1-1 | 6 | int |  |
| record | prob-weight-by-npc-killed-group2-1 | 6 | int |  |
| record | prob-weight-by-npc-killed-group3-1 | 6 | int |  |
| record | round | 6 | int |  |
| record | spawn-effect-1 | 6 | string |  |

### campfire

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | additional-heal-percent | 103 | int |  |
| record | alias | 103 | string |  |
| record | party-match | 101 | string |  |
| record | remain-duration | 103 | int |  |

### cave

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 21 | string |  |
| record | id | 21 | int |  |
| record | zone-1 | 18 | int |  |
| record | zone-10 | 4 | int |  |
| record | zone-11 | 2 | int |  |
| record | zone-12 | 2 | int |  |
| record | zone-13 | 2 | int |  |
| record | zone-14 | 2 | int |  |
| record | zone-15 | 2 | int |  |
| record | zone-16 | 2 | int |  |
| record | zone-17 | 2 | int |  |
| record | zone-18 | 2 | int |  |
| record | zone-19 | 2 | int |  |
| record | zone-2 | 13 | int |  |
| record | zone-20 | 2 | int |  |
| record | zone-21 | 2 | int |  |
| record | zone-22 | 2 | int |  |
| record | zone-23 | 2 | int |  |
| record | zone-24 | 2 | int |  |
| record | zone-25 | 2 | int |  |
| record | zone-26 | 2 | int |  |
| record | zone-27 | 2 | int |  |
| record | zone-28 | 2 | int |  |
| record | zone-29 | 2 | int |  |
| record | zone-3 | 13 | int |  |
| record | zone-30 | 2 | int |  |
| record | zone-31 | 2 | int |  |
| record | zone-32 | 2 | int |  |
| record | zone-33 | 1 | int |  |
| record | zone-34 | 1 | int |  |
| record | zone-35 | 1 | int |  |
| record | zone-36 | 1 | int |  |
| record | zone-37 | 1 | int |  |
| record | zone-38 | 1 | int |  |
| record | zone-4 | 11 | int |  |
| record | zone-5 | 11 | int |  |
| record | zone-6 | 11 | int |  |
| record | zone-7 | 9 | int |  |
| record | zone-8 | 8 | int |  |
| record | zone-9 | 5 | int |  |

### cave2

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 710 | string |  |
| record | arena-disable-zone-phase | 12 | enum-like string ❓ | y |
| record | arena-entrance-zone | 25 | string |  |
| record | enable-heart-count | 14 | enum-like string ❓ | y |
| record | enable-skill-score | 25 | enum-like string ❓ | y |
| record | group | 706 | string |  |
| record | max-instant-heart-count | 14 | int |  |
| record | required-level | 28 | int |  |
| record | skill-score-boss-npc | 25 | string |  |
| record | skill-score-boss-npc-bonus | 25 | int |  |
| record | skill-score-recommand-party-member-count-bonus | 25 | int |  |
| record | skill-score-recommend-clear-time-bonus-1 | 25 | int |  |
| record | skill-score-recommend-clear-time-bonus-2 | 25 | int |  |
| record | skill-score-recommend-clear-time-bonus-3 | 25 | int |  |
| record | skill-score-recommend-clear-time-bonus-4 | 25 | int |  |
| record | skill-score-recommend-clear-time-bonus-5 | 25 | int |  |
| record | skill-score-recommend-clear-time-step-1 | 25 | int |  |
| record | skill-score-recommend-clear-time-step-2 | 25 | int |  |
| record | skill-score-recommend-clear-time-step-3 | 25 | int |  |
| record | skill-score-recommend-clear-time-step-4 | 25 | int |  |
| record | skill-score-recommend-clear-time-step-5 | 25 | int |  |
| record | skill-score-recommend-gear-score | 25 | int |  |
| record | skill-score-recommend-hp-bonus-1 | 25 | int |  |
| record | skill-score-recommend-hp-bonus-2 | 25 | int |  |
| record | skill-score-recommend-hp-bonus-3 | 25 | int |  |
| record | skill-score-recommend-hp-bonus-4 | 25 | int |  |
| record | skill-score-recommend-hp-bonus-5 | 25 | int |  |
| record | skill-score-recommend-hp-step-1 | 25 | int |  |
| record | skill-score-recommend-hp-step-2 | 25 | int |  |
| record | skill-score-recommend-hp-step-3 | 25 | int |  |
| record | skill-score-recommend-hp-step-4 | 25 | int |  |
| record | skill-score-recommend-hp-step-5 | 25 | int |  |
| record | skill-score-recommend-use-heart-count | 25 | int |  |
| record | skill-score-recommend-use-heart-count-bonus | 25 | int |  |
| record | skill-score-recommend-use-item-bonus | 25 | int |  |
| record | skill-score-recommend-use-item-point | 25 | int |  |
| record | zone-1 | 710 | int |  |
| record | zone-10 | 6 | int |  |
| record | zone-11 | 6 | int |  |
| record | zone-12 | 6 | int |  |
| record | zone-13 | 1 | int |  |
| record | zone-2 | 26 | int |  |
| record | zone-3 | 9 | int |  |
| record | zone-4 | 7 | int |  |
| record | zone-5 | 6 | int |  |
| record | zone-6 | 6 | int |  |
| record | zone-7 | 6 | int |  |
| record | zone-8 | 6 | int |  |
| record | zone-9 | 6 | int |  |

### challenge-party

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 1 | string |  |
| record | base-damage | 1 | int |  |
| record | base-pattern-time-1 | 1 | int |  |
| record | base-play-time | 1 | int |  |
| record | boss-npc | 1 | string |  |
| record | challenge-mode-env | 1 | string |  |
| record | enter-pc-spawn | 1 | string |  |
| record | group | 1 | string |  |
| record | kill-score | 1 | int |  |
| record | max-score-damage | 1 | int |  |
| record | pattern-time-1 | 1 | int |  |
| record | play-time | 1 | int |  |
| record | remain-effect-type-1 | 1 | string |  |
| record | remain-effect-type-10 | 1 | string |  |
| record | remain-effect-type-11 | 1 | string |  |
| record | remain-effect-type-12 | 1 | string |  |
| record | remain-effect-type-13 | 1 | enum-like string ❓ | boutique-free |
| record | remain-effect-type-14 | 1 | string |  |
| record | remain-effect-type-2 | 1 | string |  |
| record | remain-effect-type-3 | 1 | string |  |
| record | remain-effect-type-4 | 1 | string |  |
| record | remain-effect-type-5 | 1 | string |  |
| record | remain-effect-type-6 | 1 | string |  |
| record | remain-effect-type-7 | 1 | string |  |
| record | remain-effect-type-8 | 1 | string |  |
| record | remain-effect-type-9 | 1 | string |  |
| record | resurrection-effect-1 | 1 | string |  |
| record | resurrection-effect-2 | 1 | string |  |
| record | resurrection-effect-3 | 1 | string |  |
| record | score-pattern-section-1-1 | 1 | int |  |
| record | score-pattern-section-2-1 | 1 | int |  |
| record | score-pattern-section-3-1 | 1 | int |  |
| record | score-per-damage | 1 | int |  |
| record | score-per-evaluate | 1 | int |  |
| record | score-per-pattern-time-1 | 1 | int |  |
| record | score-per-play-time | 1 | int |  |
| record | use-training-mode | 1 | enum-like string ❓ | n |
| record | zone | 1 | string |  |

### cinematic

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 321 | string |  |
| record | duration | 1 | int |  |
| record | skippable | 321 | enum-like string ❓ | none, party-skip, skip |

### classic-field-zone

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 12 | string |  |
| record | group | 12 | string |  |
| record | id | 12 | int |  |
| record | zone-1 | 12 | int |  |
| record | zone-2 | 3 | int |  |

### closet-collecting-grade

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | grade | 7 | int |  |
| record | score | 7 | int |  |

### closet-group

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | charge-of-item-for-instant-payment | 805 | string |  |
| record | id | 1856 | int |  |
| record | item-to-be-paid | 805 | string |  |

### collecting

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | ability-1 | 165 | enum-like string ❓ | abnormal-attack-power-value, abnormal-defend-power-value, attack-attribute-value, attack-critical-damage-value, attack-critical-value, attack-hit-value, attack-pierce-value, attack-power-creature-min-max, defend-critical-value, defend-dodge-value, defend-parry-value, defend-power-creature-value, heal-power-value, hp-regen, max-hp, pve-boss-level-npc-attack-power-creature-min-max, pve-boss-level-npc-defend-power-creature-value, pvp-attack-power-creature-min-max, pvp-defend-power-creature-value |
| record | ability-2 | 145 | enum-like string ❓ | attack-attribute-value, attack-critical-damage-value, attack-critical-value, attack-pierce-value, defend-critical-value, defend-power-creature-value, hp-regen, pve-boss-level-npc-attack-power-creature-min-max |
| record | ability-3 | 110 | enum-like string ❓ | attack-attribute-value, attack-critical-damage-value, attack-power-creature-min-max, defend-critical-value, heal-power-value, hp-regen, pvp-attack-power-creature-min-max |
| record | ability-value-1 | 165 | int |  |
| record | ability-value-2 | 145 | int |  |
| record | ability-value-3 | 110 | int |  |
| record | alias | 177 | string |  |
| record | can-not-used | 37 | enum-like string ❓ | y |
| record | category | 177 | string |  |
| record | collect-closet-1 | 175 | int |  |
| record | collect-closet-2 | 101 | int |  |
| record | collect-closet-3 | 59 | int |  |
| record | collect-closet-4 | 19 | int |  |
| record | collect-closet-5 | 7 | int |  |
| record | collect-closet-6 | 4 | int |  |
| record | collect-closet-7 | 3 | int |  |
| record | collect-closet-8 | 2 | int |  |
| record | collect-closet-replace-1 | 135 | int |  |
| record | collect-closet-replace-2 | 65 | int |  |
| record | collect-closet-replace-3 | 38 | int |  |
| record | collect-closet-replace-4 | 7 | int |  |
| record | collect-skill-skin-1 | 3 | string |  |
| record | collect-skill-skin-2 | 3 | string |  |
| record | id | 177 | int |  |
| record | reward-collecting-score | 172 | int |  |
| record | reward-item-1 | 37 | string |  |
| record | reward-item-2 | 31 | string |  |
| record | reward-item-3 | 28 | string |  |
| record | reward-item-4 | 25 | string |  |
| record | reward-item-5 | 21 | string |  |
| record | reward-item-6 | 18 | string |  |
| record | reward-item-count-1 | 37 | int |  |
| record | reward-item-count-2 | 31 | int |  |
| record | reward-item-count-3 | 28 | int |  |
| record | reward-item-count-4 | 25 | int |  |
| record | reward-item-count-5 | 21 | int |  |
| record | reward-item-count-6 | 18 | int |  |
| record | reward-money | 32 | int |  |
| record | type | 177 | enum-like string ❓ | closet |

### combat-sequence

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| combat-sequence | alias | 11533 | string |  |
| combat-sequence | hide-seq-start | 2670 | enum-like string ❓ | n, y |
| combat-sequence | script-debug-enable | 894 | enum-like string ❓ | true, y |
| combat-sequence | seq-start-social | 2624 | int |  |
| combat-sequence | special-1 | 2 | int |  |
| combat-sequence | transit-action-1 | 367 | string |  |
| combat-sequence | transit-action-2 | 54 | string |  |
| combat-sequence | transit-cond-1 | 2925 | string |  |
| combat-sequence | transit-cond-2 | 72 | string |  |
| combat-sequence | transit-social-1 | 100 | int |  |
| combat-sequence | transit-social-2 | 6 | int |  |
| combat-sequence/normal | attack-limit | 865 | int |  |
| combat-sequence/normal | change-min-msec | 2 | int |  |
| combat-sequence/normal | combat-distance | 16714 | int |  |
| combat-sequence/normal | except-summoned | 2 | enum-like string ❓ | y |
| combat-sequence/normal | hide-seq-start | 1 | enum-like string ❓ | y |
| combat-sequence/normal | id | 16714 | string |  |
| combat-sequence/normal | max-count | 36 | int |  |
| combat-sequence/normal | range-attack-limit | 7 | int |  |
| combat-sequence/normal | sepcial-1 | 11 | int |  |
| combat-sequence/normal | seq-start-social | 4 | int |  |
| combat-sequence/normal | setup-dir | 85 | enum-like string ❓ | target |
| combat-sequence/normal | setup-position | 3 | enum-like string ❓ | y |
| combat-sequence/normal | spcial-1 | 2 | int |  |
| combat-sequence/normal | special-1 | 9055 | int |  |
| combat-sequence/normal | special-2 | 2656 | int |  |
| combat-sequence/normal | special-3 | 3 | int |  |
| combat-sequence/normal | speical-1 | 1 | int |  |
| combat-sequence/normal | speicla-1 | 1 | int |  |
| combat-sequence/normal/action | arrived-effect | 129 | string |  |
| combat-sequence/normal/action | blocked-effect | 41 | string |  |
| combat-sequence/normal/action | combo | 714 | enum-like string ❓ | y |
| combat-sequence/normal/action | comby | 2 | enum-like string ❓ | y |
| combat-sequence/normal/action | duration | 2586 | int |  |
| combat-sequence/normal/action | enter-prob | 69 | int |  |
| combat-sequence/normal/action | enter_prob | 7 | int |  |
| combat-sequence/normal/action | event-step | 43 | int |  |
| combat-sequence/normal/action | except-summoned | 139 | enum-like string ❓ | n, y |
| combat-sequence/normal/action | exclude-summoned | 18 | enum-like string ❓ | n, y |
| combat-sequence/normal/action | ground-pattern-1 | 61 | string |  |
| combat-sequence/normal/action | ground-pattern-2 | 19 | string |  |
| combat-sequence/normal/action | ground-pattern-3 | 15 | string |  |
| combat-sequence/normal/action | ground-pattern-4 | 15 | string |  |
| combat-sequence/normal/action | ground-pattern-5 | 14 | string |  |
| combat-sequence/normal/action | immune-breaker-disable | 11610 | enum-like string ❓ | n, y |
| combat-sequence/normal/action | is-target-skill-gather | 1 | enum-like string ❓ | n |
| combat-sequence/normal/action | move-msec | 98 | int |  |
| combat-sequence/normal/action | move-type | 98 | string |  |
| combat-sequence/normal/action | multi-skill3-1 | 1 | string |  |
| combat-sequence/normal/action | multi-target-1 | 1 | string |  |
| combat-sequence/normal/action | origin-pos | 57 | string |  |
| combat-sequence/normal/action | pass-through | 131 | enum-like string ❓ | n, true, y |
| combat-sequence/normal/action | prob | 97 | int |  |
| combat-sequence/normal/action | range-within | 98 | int |  |
| combat-sequence/normal/action | repeat | 28380 | int |  |
| combat-sequence/normal/action | replace-top-hate | 48 | enum-like string ❓ | y |
| combat-sequence/normal/action | skill | 19262 | string |  |
| combat-sequence/normal/action | skill-1 | 16 | string |  |
| combat-sequence/normal/action | skill-2 | 16 | string |  |
| combat-sequence/normal/action | skill-3 | 16 | string |  |
| combat-sequence/normal/action | skill3 | 36160 | string |  |
| combat-sequence/normal/action | skill3-1 | 50 | string |  |
| combat-sequence/normal/action | skill3-10 | 4 | string |  |
| combat-sequence/normal/action | skill3-2 | 50 | string |  |
| combat-sequence/normal/action | skill3-3 | 50 | string |  |
| combat-sequence/normal/action | skill3-4 | 34 | string |  |
| combat-sequence/normal/action | skill3-5 | 4 | string |  |
| combat-sequence/normal/action | skill3-6 | 4 | string |  |
| combat-sequence/normal/action | skill3-7 | 4 | string |  |
| combat-sequence/normal/action | skill3-8 | 4 | string |  |
| combat-sequence/normal/action | skill3-9 | 4 | string |  |
| combat-sequence/normal/action | social | 1655 | string |  |
| combat-sequence/normal/action | sp-1 | 16 | int |  |
| combat-sequence/normal/action | sp-2 | 16 | int |  |
| combat-sequence/normal/action | sp-3 | 16 | int |  |
| combat-sequence/normal/action | speed | 154 | int |  |
| combat-sequence/normal/action | sskill3 | 2 | string |  |
| combat-sequence/normal/action | stance | 406 | int |  |
| combat-sequence/normal/action | start-effect-from-1 | 1 | string |  |
| combat-sequence/normal/action | start-effect-from-2 | 1 | string |  |
| combat-sequence/normal/action | start-effect-from-3 | 1 | string |  |
| combat-sequence/normal/action | start-effect-from-4 | 1 | string |  |
| combat-sequence/normal/action | start-effect-to-1 | 1 | string |  |
| combat-sequence/normal/action | start-effect-to-2 | 1 | string |  |
| combat-sequence/normal/action | start-effect-to-3 | 1 | string |  |
| combat-sequence/normal/action | start-effect-to-4 | 1 | string |  |
| combat-sequence/normal/action | target | 1022 | string |  |
| combat-sequence/normal/action | target-area | 152 | string |  |
| combat-sequence/normal/action | target-from | 2 | string |  |
| combat-sequence/normal/action | target-to | 2 | string |  |
| combat-sequence/normal/action | terminate-cond-1 | 1 | enum-like string ❓ | distance-max |
| combat-sequence/normal/action | terminate-value-1 | 1 | int |  |
| combat-sequence/normal/action | type | 44437 | enum-like string ❓ | boss-gp-select-attack, boss-link-laser-attack, boss-multiground-attack, boss-repeater-attack, boss-rush-attack, boss-simultaneous-caster-attack, boss-simultaneous-ground-attack, boss-sp-select-attack, change-set, combat-move, do-indexed-social, do-social, select, stay, use-ground-skill, use-indexed-skill, use-skill |
| combat-sequence/normal/action | weapon | 54 | int |  |
| combat-sequence/normal/action/action | action | 6 | string |  |
| combat-sequence/normal/action/action | arrived-effect | 9 | string |  |
| combat-sequence/normal/action/action | blocked-effect | 13 | string |  |
| combat-sequence/normal/action/action | combo | 34 | enum-like string ❓ | y |
| combat-sequence/normal/action/action | duration | 569 | int |  |
| combat-sequence/normal/action/action | except-summoned | 40 | enum-like string ❓ | y |
| combat-sequence/normal/action/action | ground-pattern-1 | 57 | string |  |
| combat-sequence/normal/action/action | ground-pattern-2 | 53 | string |  |
| combat-sequence/normal/action/action | ground-pattern-3 | 53 | string |  |
| combat-sequence/normal/action/action | ground-pattern-4 | 53 | string |  |
| combat-sequence/normal/action/action | ground-pattern-5 | 49 | string |  |
| combat-sequence/normal/action/action | immune-breaker-disable | 3065 | enum-like string ❓ | n, y |
| combat-sequence/normal/action/action | kill3 | 1 | string |  |
| combat-sequence/normal/action/action | move-msec | 11 | int |  |
| combat-sequence/normal/action/action | move-type | 11 | string |  |
| combat-sequence/normal/action/action | multi-skill3-1 | 14 | string |  |
| combat-sequence/normal/action/action | multi-skill3-2 | 14 | string |  |
| combat-sequence/normal/action/action | multi-skill3-3 | 14 | string |  |
| combat-sequence/normal/action/action | multi-skill3-4 | 14 | string |  |
| combat-sequence/normal/action/action | multi-skill3-5 | 14 | string |  |
| combat-sequence/normal/action/action | multi-skill3-6 | 14 | string |  |
| combat-sequence/normal/action/action | multi-skill3-7 | 14 | string |  |
| combat-sequence/normal/action/action | multi-skill3-8 | 14 | string |  |
| combat-sequence/normal/action/action | multi-target-1 | 14 | string |  |
| combat-sequence/normal/action/action | multi-target-2 | 14 | string |  |
| combat-sequence/normal/action/action | multi-target-3 | 14 | string |  |
| combat-sequence/normal/action/action | multi-target-4 | 14 | string |  |
| combat-sequence/normal/action/action | multi-target-5 | 14 | string |  |
| combat-sequence/normal/action/action | multi-target-6 | 14 | string |  |
| combat-sequence/normal/action/action | multi-target-7 | 14 | string |  |
| combat-sequence/normal/action/action | multi-target-8 | 14 | string |  |
| combat-sequence/normal/action/action | origin-pos | 57 | string |  |
| combat-sequence/normal/action/action | pass-through | 15 | enum-like string ❓ | n, true, y |
| combat-sequence/normal/action/action | prob | 10258 | int |  |
| combat-sequence/normal/action/action | range-within | 11 | int |  |
| combat-sequence/normal/action/action | repeat | 5511 | int |  |
| combat-sequence/normal/action/action | rpob | 2 | int |  |
| combat-sequence/normal/action/action | skill | 3194 | string |  |
| combat-sequence/normal/action/action | skill3 | 7510 | string |  |
| combat-sequence/normal/action/action | social | 2230 | string |  |
| combat-sequence/normal/action/action | speed | 15 | int |  |
| combat-sequence/normal/action/action | target | 285 | string |  |
| combat-sequence/normal/action/action | target-area | 6 | string |  |
| combat-sequence/normal/action/action | target-area-ref | 8 | int |  |
| combat-sequence/normal/action/action | type | 10309 | enum-like string ❓ | boss-multiground-attack, boss-repeater-attack, boss-rush-attack, boss-simultaneous-ground-attack, combat-move, do-indexed-social, do-social, stay, use-ground-skill, use-indexed-skill, use-skill |
| combat-sequence/special | attack-limit | 2 | int |  |
| combat-sequence/special | combat-destance | 2 | int |  |
| combat-sequence/special | combat-disatnce | 2 | int |  |
| combat-sequence/special | combat-distance | 13310 | string |  |
| combat-sequence/special | condition-1 | 3363 | string |  |
| combat-sequence/special | condition-2 | 16 | string |  |
| combat-sequence/special | condition-subscription-1 | 891 | string |  |
| combat-sequence/special | id | 13320 | int |  |
| combat-sequence/special | max-count | 12432 | int |  |
| combat-sequence/special | min-delay | 2831 | int |  |
| combat-sequence/special | script-debug-enable | 2 | enum-like string ❓ | y |
| combat-sequence/special | special-1 | 1 | int |  |
| combat-sequence/special | start-delay | 1925 | int |  |
| combat-sequence/special/action | arrived-effect | 1690 | string |  |
| combat-sequence/special/action | blocked-effect | 1550 | string |  |
| combat-sequence/special/action | combo | 1058 | enum-like string ❓ | y |
| combat-sequence/special/action | condition | 19 | string |  |
| combat-sequence/special/action | duration | 3779 | int |  |
| combat-sequence/special/action | enter-prob | 3 | int |  |
| combat-sequence/special/action | enter_prob | 5 | int |  |
| combat-sequence/special/action | event-step | 9014 | int |  |
| combat-sequence/special/action | except-condition | 181 | string |  |
| combat-sequence/special/action | except-link-laser | 44 | enum-like string ❓ | n, y |
| combat-sequence/special/action | except-summoned | 2601 | enum-like string ❓ | y |
| combat-sequence/special/action | except-top-hate | 521 | enum-like string ❓ | n, y |
| combat-sequence/special/action | exclude-summoned | 171 | enum-like string ❓ | n, y |
| combat-sequence/special/action | gather-count | 630 | int |  |
| combat-sequence/special/action | gather-rule | 630 | enum-like string ❓ | far, min-hate, near, random, rotate |
| combat-sequence/special/action | ground-pattern-1 | 711 | string |  |
| combat-sequence/special/action | ground-pattern-2 | 201 | string |  |
| combat-sequence/special/action | ground-pattern-3 | 161 | string |  |
| combat-sequence/special/action | ground-pattern-4 | 155 | string |  |
| combat-sequence/special/action | ground-pattern-5 | 155 | string |  |
| combat-sequence/special/action | height | 2 | int |  |
| combat-sequence/special/action | immune-breaker-disable | 28311 | enum-like string ❓ | n, y |
| combat-sequence/special/action | is-target-skill-gather | 77 | enum-like string ❓ | n, y |
| combat-sequence/special/action | move-msec | 104 | int |  |
| combat-sequence/special/action | move-type | 104 | string |  |
| combat-sequence/special/action | multi-skill3-1 | 18 | string |  |
| combat-sequence/special/action | multi-skill3-10 | 8 | string |  |
| combat-sequence/special/action | multi-skill3-11 | 6 | string |  |
| combat-sequence/special/action | multi-skill3-12 | 4 | string |  |
| combat-sequence/special/action | multi-skill3-2 | 15 | string |  |
| combat-sequence/special/action | multi-skill3-3 | 13 | string |  |
| combat-sequence/special/action | multi-skill3-4 | 13 | string |  |
| combat-sequence/special/action | multi-skill3-5 | 13 | string |  |
| combat-sequence/special/action | multi-skill3-6 | 13 | string |  |
| combat-sequence/special/action | multi-skill3-7 | 8 | string |  |
| combat-sequence/special/action | multi-skill3-8 | 8 | string |  |
| combat-sequence/special/action | multi-skill3-9 | 8 | string |  |
| combat-sequence/special/action | multi-target-1 | 18 | string |  |
| combat-sequence/special/action | multi-target-10 | 8 | string |  |
| combat-sequence/special/action | multi-target-11 | 6 | string |  |
| combat-sequence/special/action | multi-target-12 | 4 | string |  |
| combat-sequence/special/action | multi-target-2 | 15 | string |  |
| combat-sequence/special/action | multi-target-3 | 13 | string |  |
| combat-sequence/special/action | multi-target-4 | 13 | string |  |
| combat-sequence/special/action | multi-target-5 | 13 | string |  |
| combat-sequence/special/action | multi-target-6 | 13 | string |  |
| combat-sequence/special/action | multi-target-7 | 8 | string |  |
| combat-sequence/special/action | multi-target-8 | 8 | string |  |
| combat-sequence/special/action | multi-target-9 | 8 | string |  |
| combat-sequence/special/action | normal-bounce | 3 | enum-like string ❓ | y |
| combat-sequence/special/action | origin-pos | 616 | string |  |
| combat-sequence/special/action | pass-through | 2166 | enum-like string ❓ | n, true, y |
| combat-sequence/special/action | prob | 10 | int |  |
| combat-sequence/special/action | radius | 2 | int |  |
| combat-sequence/special/action | range-within | 104 | int |  |
| combat-sequence/special/action | repeat | 16730 | int |  |
| combat-sequence/special/action | replace-top-hate | 788 | enum-like string ❓ | n, y |
| combat-sequence/special/action | skill | 10515 | string |  |
| combat-sequence/special/action | skill-1 | 8 | string |  |
| combat-sequence/special/action | skill-2 | 8 | string |  |
| combat-sequence/special/action | skill-3 | 8 | string |  |
| combat-sequence/special/action | skill3 | 29735 | string |  |
| combat-sequence/special/action | skill3-1 | 51 | string |  |
| combat-sequence/special/action | skill3-10 | 3 | string |  |
| combat-sequence/special/action | skill3-2 | 51 | string |  |
| combat-sequence/special/action | skill3-3 | 51 | string |  |
| combat-sequence/special/action | skill3-4 | 41 | string |  |
| combat-sequence/special/action | skill3-5 | 41 | string |  |
| combat-sequence/special/action | skill3-6 | 41 | string |  |
| combat-sequence/special/action | skill3-7 | 32 | string |  |
| combat-sequence/special/action | skill3-8 | 32 | string |  |
| combat-sequence/special/action | skill3-9 | 3 | string |  |
| combat-sequence/special/action | social | 6908 | string |  |
| combat-sequence/special/action | sp-1 | 10 | int |  |
| combat-sequence/special/action | sp-2 | 10 | int |  |
| combat-sequence/special/action | sp-3 | 10 | int |  |
| combat-sequence/special/action | speed | 2255 | int |  |
| combat-sequence/special/action | stance | 403 | int |  |
| combat-sequence/special/action | stance-effect-1 | 9 | int |  |
| combat-sequence/special/action | start-effect-from-1 | 21 | string |  |
| combat-sequence/special/action | start-effect-from-2 | 7 | string |  |
| combat-sequence/special/action | start-effect-from-3 | 7 | string |  |
| combat-sequence/special/action | start-effect-from-4 | 7 | string |  |
| combat-sequence/special/action | start-effect-to-1 | 33 | string |  |
| combat-sequence/special/action | start-effect-to-2 | 19 | string |  |
| combat-sequence/special/action | start-effect-to-3 | 15 | string |  |
| combat-sequence/special/action | start-effect-to-4 | 7 | string |  |
| combat-sequence/special/action | target | 11191 | string |  |
| combat-sequence/special/action | target-area | 1615 | string |  |
| combat-sequence/special/action | target-area-ref | 5 | int |  |
| combat-sequence/special/action | target-from | 214 | string |  |
| combat-sequence/special/action | target-pos | 65 | string |  |
| combat-sequence/special/action | target-to | 214 | string |  |
| combat-sequence/special/action | terminate-cond-1 | 166 | enum-like string ❓ | distance-max, distance-min, target-invoked-effect-flag, target-leave, time-over |
| combat-sequence/special/action | terminate-cond-2 | 33 | enum-like string ❓ | distance-max |
| combat-sequence/special/action | terminate-flag-1 | 2 | string |  |
| combat-sequence/special/action | terminate-op | 5 | bool |  |
| combat-sequence/special/action | terminate-value-1 | 119 | int |  |
| combat-sequence/special/action | terminate-value-2 | 33 | int |  |
| combat-sequence/special/action | type | 41695 | enum-like string ❓ | boss-gp-select-attack, boss-link-laser-attack, boss-multiground-attack, boss-repeater-attack, boss-rush-attack, boss-simultaneous-caster-attack, boss-simultaneous-ground-attack, boss-sp-select-attack, change-set, combat-move, detect-creature, do-indexed-social, do-social, gather-targets, select, stay, use-ground-skill, use-indexed-skill, use-skill, use-soul-npc-skill |
| combat-sequence/special/action | weapon | 55 | int |  |
| combat-sequence/special/action/action | arrived-effect | 101 | string |  |
| combat-sequence/special/action/action | blocked-effect | 1 | string |  |
| combat-sequence/special/action/action | combo | 25 | enum-like string ❓ | y |
| combat-sequence/special/action/action | duration | 307 | int |  |
| combat-sequence/special/action/action | event-step | 7 | int |  |
| combat-sequence/special/action/action | except-condition | 4 | enum-like string ❓ | hide |
| combat-sequence/special/action/action | except-summoned | 24 | enum-like string ❓ | y |
| combat-sequence/special/action/action | ground-pattern-1 | 144 | string |  |
| combat-sequence/special/action/action | ground-pattern-2 | 106 | string |  |
| combat-sequence/special/action/action | ground-pattern-3 | 106 | string |  |
| combat-sequence/special/action/action | ground-pattern-4 | 106 | string |  |
| combat-sequence/special/action/action | ground-pattern-5 | 106 | string |  |
| combat-sequence/special/action/action | immune-breaker-disable | 1206 | enum-like string ❓ | n, y |
| combat-sequence/special/action/action | move-msec | 20 | int |  |
| combat-sequence/special/action/action | move-type | 20 | enum-like string ❓ | flee |
| combat-sequence/special/action/action | multi-skill3-1 | 10 | string |  |
| combat-sequence/special/action/action | multi-skill3-2 | 10 | string |  |
| combat-sequence/special/action/action | multi-skill3-3 | 10 | string |  |
| combat-sequence/special/action/action | multi-target-1 | 10 | string |  |
| combat-sequence/special/action/action | multi-target-2 | 10 | string |  |
| combat-sequence/special/action/action | multi-target-3 | 10 | string |  |
| combat-sequence/special/action/action | origin-pos | 144 | string |  |
| combat-sequence/special/action/action | pass-through | 101 | enum-like string ❓ | true, y |
| combat-sequence/special/action/action | prob | 4020 | int |  |
| combat-sequence/special/action/action | range-within | 20 | int |  |
| combat-sequence/special/action/action | repeat | 973 | int |  |
| combat-sequence/special/action/action | replace-top-hate | 16 | enum-like string ❓ | y |
| combat-sequence/special/action/action | skill | 620 | string |  |
| combat-sequence/special/action/action | skill3 | 1422 | string |  |
| combat-sequence/special/action/action | skill3-1 | 6 | string |  |
| combat-sequence/special/action/action | skill3-10 | 6 | string |  |
| combat-sequence/special/action/action | skill3-2 | 6 | string |  |
| combat-sequence/special/action/action | skill3-3 | 6 | string |  |
| combat-sequence/special/action/action | skill3-4 | 6 | string |  |
| combat-sequence/special/action/action | skill3-5 | 6 | string |  |
| combat-sequence/special/action/action | skill3-6 | 6 | string |  |
| combat-sequence/special/action/action | skill3-7 | 6 | string |  |
| combat-sequence/special/action/action | skill3-8 | 6 | string |  |
| combat-sequence/special/action/action | skill3-9 | 6 | string |  |
| combat-sequence/special/action/action | social | 2266 | string |  |
| combat-sequence/special/action/action | speed | 101 | int |  |
| combat-sequence/special/action/action | target | 481 | string |  |
| combat-sequence/special/action/action | target-area | 323 | string |  |
| combat-sequence/special/action/action | type | 4021 | enum-like string ❓ | boss-gp-select-attack, boss-multiground-attack, boss-repeater-attack, boss-rush-attack, combat-move, do-indexed-social, do-social, stay, use-ground-skill, use-skill |
| combat-sequence/special/special | combat-distance | 1 | int |  |
| combat-sequence/special/special | id | 1 | int |  |
| combat-sequence/special/special | max-count | 1 | int |  |
| combat-sequence/special/special/action | skill3 | 1 | string |  |
| combat-sequence/special/special/action | type | 1 | enum-like string ❓ | use-skill |

### content-quota

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 51 | string |  |
| record | charge-amount-per-interval | 51 | int |  |
| record | charge-day-of-week | 3 | enum-like string ❓ | fri, thu, wed |
| record | charge-interval | 51 | enum-like string ❓ | daily, hourly, none, weekly |
| record | charge-time | 27 | int |  |
| record | consume-key-record | 51 | enum-like string ❓ | y |
| record | expiration-time | 5 | string |  |
| record | id | 51 | int |  |
| record | max-value | 51 | int |  |
| record | min-value | 51 | int |  |
| record | target-type | 43 | enum-like string ❓ | account, character |
| record | type | 51 | string |  |
| record | version | 51 | int |  |

### contents-escape-penalty

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | contents-type | 4 | string |  |
| record | penalty-score | 4 | int |  |

### contents-reset

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | max-unbind-count | 15 | int |  |
| record | reset-item-1 | 21 | string |  |
| record | reset-item-count-1 | 21 | int |  |
| record | reset-money | 15 | int |  |
| record | reset-target | 21 | string |  |

### craft-case-recipe

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 711 | string |  |
| record | cost-material-1 | 471 | string |  |
| record | cost-material-2 | 471 | string |  |
| record | cost-material-3 | 447 | string |  |
| record | cost-material-4 | 398 | string |  |
| record | cost-material-5 | 380 | string |  |
| record | cost-material-6 | 186 | string |  |
| record | cost-material-7 | 186 | string |  |
| record | cost-material-8 | 14 | string |  |
| record | cost-material-amount-1 | 471 | int |  |
| record | cost-material-amount-2 | 471 | int |  |
| record | cost-material-amount-3 | 447 | int |  |
| record | cost-material-amount-4 | 398 | int |  |
| record | cost-material-amount-5 | 380 | int |  |
| record | cost-material-amount-6 | 186 | int |  |
| record | cost-material-amount-7 | 186 | int |  |
| record | cost-material-amount-8 | 14 | int |  |
| record | cost-money | 711 | int |  |
| record | craft-duration | 711 | int |  |
| record | group-alias | 711 | string |  |
| record | half-craft-exp-step | 711 | int |  |
| record | id | 711 | int |  |
| record | required-achievement-id | 353 | int |  |
| record | required-achievement-step-min | 353 | int |  |
| record | required-craft-exp | 711 | int |  |
| record | reward-craft-exp | 711 | int |  |
| record | reward-item-1 | 711 | string |  |
| record | reward-item-2 | 180 | string |  |
| record | reward-item-3 | 60 | string |  |
| record | reward-item-max-1 | 711 | int |  |
| record | reward-item-max-2 | 180 | int |  |
| record | reward-item-max-3 | 60 | int |  |
| record | reward-item-min-1 | 711 | int |  |
| record | reward-item-min-2 | 180 | int |  |
| record | reward-item-min-3 | 60 | int |  |
| record | reward-item-random-distribution-1 | 711 | string |  |
| record | reward-item-random-distribution-2 | 180 | string |  |
| record | reward-item-random-distribution-3 | 60 | string |  |
| record | use-state | 711 | enum-like string ❓ | in-use, no-more-use |
| record | zero-craft-exp-step | 711 | int |  |

### craft-group-recipe

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 168 | string |  |
| record | craft-union-sub-type | 168 | string |  |
| record | craft-union-type | 168 | string |  |
| record | required-craft-exp | 168 | int |  |

### craft-recipe

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 1299 | string |  |
| record | brand-material-1 | 13 | string |  |
| record | brand-material-amount-1 | 13 | int |  |
| record | brand-material-condition-type-1 | 13 | enum-like string ❓ | all, assassin-dagger, aura-bangle, axe, blade-master-sword, kung-fu-fighter-gauntlet, lyn-sword, staff, warlock-dagger |
| record | category | 1299 | string |  |
| record | half-production-exp-step | 1299 | int |  |
| record | id | 1299 | int |  |
| record | inner-category | 1299 | int |  |
| record | limit-production-exp | 1299 | int |  |
| record | material-1 | 1145 | string |  |
| record | material-2 | 999 | string |  |
| record | material-3 | 756 | string |  |
| record | material-4 | 458 | string |  |
| record | material-5 | 128 | string |  |
| record | material-amount-1 | 1145 | int |  |
| record | material-amount-2 | 999 | int |  |
| record | material-amount-3 | 756 | int |  |
| record | material-amount-4 | 458 | int |  |
| record | material-amount-5 | 128 | int |  |
| record | money-cost | 1299 | int |  |
| record | reward-production-exp | 1299 | int |  |
| record | step-1 | 1299 | string |  |
| record | step-2 | 171 | string |  |
| record | use-state | 1299 | enum-like string ❓ | no-more-use |
| record | zero-production-exp-step | 1299 | int |  |

### craft-recipe-step

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 1486 | string |  |
| record | output-amount-1 | 1486 | int |  |
| record | output-amount-2 | 187 | int |  |
| record | output-id-1 | 1486 | int |  |
| record | output-id-2 | 187 | int |  |
| record | time-cost | 1486 | int |  |

### custom-dress-design-cost

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 866 | string |  |
| record | customize-id | 866 | int |  |
| record | item-type | 866 | enum-like string ❓ | costume, costume-attach, face-attach, head-attach |
| record | required-item-1 | 866 | string |  |
| record | required-item-count-1 | 866 | int |  |
| record | reset-design | 11 | enum-like string ❓ | y |
| record | resource-index | 866 | int |  |
| record | usable | 866 | enum-like string ❓ | n, y |

### custom-dress-design-type-info

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 13 | string |  |
| record | customize-category | 13 | enum-like string ❓ | center, dressline, hair, leg, sub |
| record | customize-id | 13 | int |  |
| record | design-type | 13 | enum-like string ❓ | color, pattern, saturation |
| record | item-type | 13 | enum-like string ❓ | costume, costume-attach, face-attach, head-attach |
| record | max-value | 2 | int |  |
| record | min-value | 2 | int |  |
| record | price-ref | 2 | string |  |

### default-rating-score-group

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | default-rating-score | 17 | int |  |
| record | group-id | 17 | int |  |
| record | is-placement-test | 17 | enum-like string ❓ | n, y |
| record | min-rating-score | 17 | int |  |
| record | season-contents-type | 17 | string |  |

### difficulty-type

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 3 | enum-like string ❓ | easy, hard, normal |
| record | default | 1 | enum-like string ❓ | yes |
| record | id | 3 | enum-like string ❓ | easy, hard, normal |
| record | is-pvp | 1 | enum-like string ❓ | yes |
| record | max-party-member-count | 3 | int |  |

### difficulty-type-modify

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 102 | string |  |
| record | berserk-sequence-invoke-time-1 | 33 | int |  |
| record | berserk-sequence-invoke-time-2 | 31 | int |  |
| record | berserk-sequence-invoke-time-3 | 31 | int |  |
| record | difficulty-type-damage-percent-1 | 102 | int |  |
| record | difficulty-type-damage-percent-2 | 102 | int |  |
| record | difficulty-type-damage-percent-3 | 102 | int |  |
| record | mysterious-store-spawn-difficulty-type-2 | 11 | string |  |
| record | mysterious-store-spawn-difficulty-type-3 | 5 | string |  |
| record | mysterious-store-spawn-prob-difficulty-type-2 | 11 | int |  |
| record | mysterious-store-spawn-prob-difficulty-type-3 | 5 | int |  |

### duel

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 36 | string |  |
| record | arena-inside-alpha-side-pc-spawn-1 | 36 | string |  |
| record | arena-inside-alpha-side-pc-spawn-2 | 25 | string |  |
| record | arena-inside-alpha-side-pc-spawn-3 | 25 | string |  |
| record | arena-inside-beta-side-pc-spawn-1 | 36 | string |  |
| record | arena-inside-beta-side-pc-spawn-2 | 25 | string |  |
| record | arena-inside-beta-side-pc-spawn-3 | 25 | string |  |
| record | arena-inside-spectator-pc-spawn | 36 | string |  |
| record | arena-outside-alpha-side-pc-spawn-1 | 11 | string |  |
| record | arena-outside-alpha-side-pc-spawn-2 | 11 | string |  |
| record | arena-outside-alpha-side-pc-spawn-3 | 11 | string |  |
| record | arena-outside-beta-side-pc-spawn-1 | 11 | string |  |
| record | arena-outside-beta-side-pc-spawn-2 | 11 | string |  |
| record | arena-outside-beta-side-pc-spawn-3 | 11 | string |  |
| record | arena-outside-round-start-effect-1 | 11 | string |  |
| record | arena-outside-summoned-round-start-effect-1 | 11 | string |  |
| record | available-ingame-championship-matching-weekly-time | 3 | string |  |
| record | available-normal-matching-weekly-time | 6 | string |  |
| record | bonus-exp-percent-1 | 12 | int |  |
| record | bonus-exp-percent-2 | 6 | int |  |
| record | bonus-point-percent-1 | 12 | int |  |
| record | bonus-point-percent-2 | 6 | int |  |
| record | duel-type | 36 | string |  |
| record | effect | 18 | string |  |
| record | group | 36 | string |  |
| record | max-round-count | 36 | int |  |
| record | round-count | 36 | int |  |
| record | round-duration | 36 | int |  |
| record | round-finished-duration | 36 | int |  |
| record | round-finished-duration-by-decision | 36 | int |  |
| record | round-restart-duration | 36 | int |  |
| record | round-standby-duration | 36 | int |  |
| record | round-start-countdown-duration | 36 | int |  |
| record | round-win-count | 36 | int |  |
| record | standby-alpha-side-pc-spawn | 36 | string |  |
| record | standby-beta-side-pc-spawn | 36 | string |  |
| record | tag-match-interference-bonus-time-1 | 9 | int |  |
| record | tag-match-interference-bonus-time-2 | 9 | int |  |
| record | tag-match-interference-into-effect-1 | 11 | string |  |
| record | tag-match-interference-into-effect-2 | 11 | string |  |
| record | tag-match-interference-into-effect-3 | 11 | string |  |
| record | tag-match-interference-into-effect-4 | 11 | string |  |
| record | tag-match-interference-out-effect-1 | 11 | string |  |
| record | tag-match-interference-out-effect-2 | 11 | string |  |
| record | tag-match-interference-out-effect-3 | 11 | string |  |
| record | tag-match-interference-out-effect-4 | 11 | string |  |
| record | tag-match-summoned-interference-into-effect-1 | 11 | string |  |
| record | tag-match-summoned-interference-into-effect-2 | 11 | string |  |
| record | tag-match-summoned-interference-into-effect-3 | 11 | string |  |
| record | tag-match-summoned-interference-out-effect-1 | 11 | string |  |
| record | tag-match-summoned-interference-out-effect-2 | 11 | string |  |
| record | tag-match-summoned-interference-out-effect-3 | 11 | string |  |
| record | tag-match-summoned-tag-into-effect-by-skill-1 | 11 | string |  |
| record | tag-match-summoned-tag-into-effect-by-skill-2 | 11 | string |  |
| record | tag-match-summoned-tag-into-effect-by-skill-3 | 11 | string |  |
| record | tag-match-summoned-tag-into-effect-by-skill-4 | 11 | string |  |
| record | tag-match-summoned-tag-into-effect-by-system-1 | 11 | string |  |
| record | tag-match-summoned-tag-into-effect-by-system-2 | 11 | string |  |
| record | tag-match-summoned-tag-into-effect-by-system-3 | 11 | string |  |
| record | tag-match-summoned-tag-into-effect-by-system-4 | 11 | string |  |
| record | tag-match-summoned-tag-into-effect-by-system-5 | 11 | string |  |
| record | tag-match-summoned-tag-out-effect-1 | 11 | string |  |
| record | tag-match-summoned-tag-out-effect-2 | 11 | string |  |
| record | tag-match-summoned-tag-out-effect-3 | 11 | string |  |
| record | tag-match-tag-into-effect-by-skill-1 | 11 | string |  |
| record | tag-match-tag-into-effect-by-skill-2 | 11 | string |  |
| record | tag-match-tag-into-effect-by-skill-3 | 11 | string |  |
| record | tag-match-tag-into-effect-by-skill-4 | 11 | string |  |
| record | tag-match-tag-into-effect-by-skill-5 | 11 | string |  |
| record | tag-match-tag-into-effect-by-system-1 | 11 | string |  |
| record | tag-match-tag-into-effect-by-system-2 | 11 | string |  |
| record | tag-match-tag-into-effect-by-system-3 | 11 | string |  |
| record | tag-match-tag-into-effect-by-system-4 | 11 | string |  |
| record | tag-match-tag-into-effect-by-system-5 | 11 | string |  |
| record | tag-match-tag-into-effect-by-system-6 | 11 | string |  |
| record | tag-match-tag-out-effect-1 | 11 | string |  |
| record | tag-match-tag-out-effect-2 | 11 | string |  |
| record | tag-match-tag-out-effect-3 | 11 | string |  |
| record | tag-match-tag-out-effect-4 | 11 | string |  |
| record | tag-match-tag-out-effect-5 | 11 | string |  |
| record | weekly-time-table-for-added-reward-1 | 12 | string |  |
| record | weekly-time-table-for-added-reward-2 | 6 | string |  |
| record | zone | 36 | int |  |

### duel-bot

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | ability | 315 | string |  |
| record | alias | 315 | string |  |
| record | despawn-social | 315 | string |  |
| record | equipped-item | 315 | string |  |
| record | job | 315 | string |  |
| record | job-style | 315 | string |  |
| record | level | 315 | int |  |
| record | mastery-level | 315 | int |  |
| record | race | 315 | string |  |
| record | radius | 315 | int |  |
| record | sex | 315 | string |  |
| record | spawn-social | 315 | string |  |
| record | speed | 315 | int |  |
| record | trained-skill-set-vs-archer | 315 | string |  |
| record | trained-skill-set-vs-assassin | 315 | string |  |
| record | trained-skill-set-vs-blade-master | 315 | string |  |
| record | trained-skill-set-vs-destroyer | 315 | string |  |
| record | trained-skill-set-vs-force-master | 315 | string |  |
| record | trained-skill-set-vs-kung-fu-fighter | 315 | string |  |
| record | trained-skill-set-vs-shooter | 315 | string |  |
| record | trained-skill-set-vs-soul-fighter | 315 | string |  |
| record | trained-skill-set-vs-summoner | 315 | string |  |
| record | trained-skill-set-vs-sword-master | 315 | string |  |
| record | trained-skill-set-vs-warlock | 315 | string |  |
| record | trained-skill-set-vs-warrior | 315 | string |  |
| record | yaw | 315 | int |  |

### duel-bot-ability

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | abnormal-attack-base-percent | 285 | int |  |
| record | abnormal-attack-power-modify | 285 | int |  |
| record | abnormal-defend-power-modify | 285 | int |  |
| record | alias | 285 | string |  |
| record | aoe-defend-base-percent | 285 | int |  |
| record | aoe-defend-power-modify | 285 | int |  |
| record | aoe-defend-power-value | 285 | int |  |
| record | attack-abnormal-hit-base-percent | 285 | int |  |
| record | attack-abnormal-hit-value | 285 | int |  |
| record | attack-aoe-pierce-value | 285 | int |  |
| record | attack-attribute-base-percent | 285 | int |  |
| record | attack-attribute-modify | 285 | int |  |
| record | attack-concentrate-value | 285 | int |  |
| record | attack-counter-damage-modify | 285 | int |  |
| record | attack-critical-base-percent | 285 | int |  |
| record | attack-critical-damage-modify | 285 | int |  |
| record | attack-critical-damage-percent | 285 | int |  |
| record | attack-critical-damage-value | 280 | int |  |
| record | attack-critical-value | 285 | int |  |
| record | attack-critical-value-modify | 285 | int |  |
| record | attack-damage-modify-diff | 285 | int |  |
| record | attack-damage-modify-percent | 285 | int |  |
| record | attack-hit-base-percent | 285 | int |  |
| record | attack-hit-value | 285 | int |  |
| record | attack-hit-value-modify | 285 | int |  |
| record | attack-parry-pierce-modify | 285 | int |  |
| record | attack-parry-pierce-percent | 285 | int |  |
| record | attack-perfect-parry-damage-modify | 285 | int |  |
| record | attack-pierce-base-percent | 285 | int |  |
| record | attack-pierce-modify | 285 | int |  |
| record | attack-pierce-value | 285 | int |  |
| record | attack-power-creature-max | 285 | int |  |
| record | attack-power-creature-min | 285 | int |  |
| record | attack-power-equip-max | 285 | int |  |
| record | attack-power-equip-min | 285 | int |  |
| record | attack-stiff-duration-base-percent | 285 | int |  |
| record | attack-stiff-duration-value | 285 | int |  |
| record | attack-stiff-duration-value-modify | 285 | int |  |
| record | cast-duration-base-percent | 285 | int |  |
| record | cast-duration-value | 285 | int |  |
| record | cast-duration-value-modify | 285 | int |  |
| record | defend-abnormal-dodge-base-percent | 285 | int |  |
| record | defend-abnormal-dodge-value | 285 | int |  |
| record | defend-bounce-percent | 285 | int |  |
| record | defend-counter-reduce-modify | 285 | int |  |
| record | defend-counter-reduce-percent | 285 | int |  |
| record | defend-critical-base-percent | 285 | int |  |
| record | defend-critical-damage-modify | 285 | int |  |
| record | defend-critical-damage-percent | 285 | int |  |
| record | defend-critical-value | 285 | int |  |
| record | defend-critical-value-modify | 285 | int |  |
| record | defend-damage-modify-diff | 285 | int |  |
| record | defend-damage-modify-percent | 285 | int |  |
| record | defend-dodge-base-percent | 285 | int |  |
| record | defend-dodge-value | 285 | int |  |
| record | defend-dodge-value-modify | 285 | int |  |
| record | defend-force-damage-reduce-percent | 285 | int |  |
| record | defend-force-value-modify | 285 | int |  |
| record | defend-immune-base-percent | 285 | int |  |
| record | defend-miss-base-percent | 285 | int |  |
| record | defend-parry-base-percent | 285 | int |  |
| record | defend-parry-reduce-diff | 285 | int |  |
| record | defend-parry-reduce-modify | 285 | int |  |
| record | defend-parry-reduce-percent | 285 | int |  |
| record | defend-parry-value | 285 | int |  |
| record | defend-parry-value-modify | 285 | int |  |
| record | defend-perfect-parry-base-percent | 285 | int |  |
| record | defend-perfect-parry-reduce-modify | 285 | int |  |
| record | defend-perfect-parry-reduce-percent | 285 | int |  |
| record | defend-physical-damage-reduce-percent | 285 | int |  |
| record | defend-physical-value-modify | 285 | int |  |
| record | defend-power-creature-value | 285 | int |  |
| record | defend-power-equip-value | 285 | int |  |
| record | defend-resist-power-creature-value | 285 | int |  |
| record | defend-resist-power-equip-value | 285 | int |  |
| record | defend-stiff-duration-base-percent | 285 | int |  |
| record | defend-stiff-duration-value | 285 | int |  |
| record | defend-stiff-duration-value-modify | 285 | int |  |
| record | hate-base-percent | 285 | int |  |
| record | hate-power-creature-value | 285 | int |  |
| record | hate-power-modify | 285 | int |  |
| record | heal-power-base-percent | 285 | int |  |
| record | heal-power-modify | 285 | int |  |
| record | hp-regen | 285 | int |  |
| record | hp-regen-combat | 285 | int |  |
| record | job-ability-1 | 40 | int |  |
| record | job-ability-2 | 40 | int |  |
| record | max-guard-gauge | 285 | int |  |
| record | max-hp | 285 | int |  |
| record | max-sp | 285 | int |  |
| record | r-attack-abnormal-hit-modify | 285 | int |  |
| record | r-attack-aoe-pierce-modify | 285 | int |  |
| record | r-defend-abnormal-dodge-modify | 285 | int |  |
| record | r-support-power-modify | 285 | int |  |
| record | sp-regen | 285 | int |  |
| record | support-power-base-percent | 285 | int |  |
| record | support-power-value | 285 | int |  |

### duel-bot-challenge

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 1 | string |  |
| record | dispel-effect-type-1 | 1 | enum-like string ❓ | marking |
| record | dispel-effect-type-10 | 1 | string |  |
| record | dispel-effect-type-11 | 1 | string |  |
| record | dispel-effect-type-12 | 1 | string |  |
| record | dispel-effect-type-13 | 1 | string |  |
| record | dispel-effect-type-14 | 1 | string |  |
| record | dispel-effect-type-15 | 1 | string |  |
| record | dispel-effect-type-16 | 1 | string |  |
| record | dispel-effect-type-17 | 1 | enum-like string ❓ | additional-hp |
| record | dispel-effect-type-18 | 1 | string |  |
| record | dispel-effect-type-19 | 1 | string |  |
| record | dispel-effect-type-2 | 1 | string |  |
| record | dispel-effect-type-20 | 1 | string |  |
| record | dispel-effect-type-21 | 1 | string |  |
| record | dispel-effect-type-22 | 1 | string |  |
| record | dispel-effect-type-23 | 1 | string |  |
| record | dispel-effect-type-24 | 1 | string |  |
| record | dispel-effect-type-25 | 1 | string |  |
| record | dispel-effect-type-26 | 1 | string |  |
| record | dispel-effect-type-27 | 1 | string |  |
| record | dispel-effect-type-28 | 1 | string |  |
| record | dispel-effect-type-29 | 1 | string |  |
| record | dispel-effect-type-3 | 1 | string |  |
| record | dispel-effect-type-30 | 1 | string |  |
| record | dispel-effect-type-31 | 1 | string |  |
| record | dispel-effect-type-32 | 1 | string |  |
| record | dispel-effect-type-33 | 1 | string |  |
| record | dispel-effect-type-34 | 1 | string |  |
| record | dispel-effect-type-35 | 1 | enum-like string ❓ | magnetic-seal |
| record | dispel-effect-type-36 | 1 | string |  |
| record | dispel-effect-type-37 | 1 | string |  |
| record | dispel-effect-type-38 | 1 | string |  |
| record | dispel-effect-type-39 | 1 | string |  |
| record | dispel-effect-type-4 | 1 | string |  |
| record | dispel-effect-type-40 | 1 | string |  |
| record | dispel-effect-type-41 | 1 | string |  |
| record | dispel-effect-type-42 | 1 | string |  |
| record | dispel-effect-type-43 | 1 | enum-like string ❓ | attack-glide |
| record | dispel-effect-type-44 | 1 | string |  |
| record | dispel-effect-type-45 | 1 | string |  |
| record | dispel-effect-type-46 | 1 | string |  |
| record | dispel-effect-type-47 | 1 | string |  |
| record | dispel-effect-type-48 | 1 | string |  |
| record | dispel-effect-type-49 | 1 | string |  |
| record | dispel-effect-type-5 | 1 | string |  |
| record | dispel-effect-type-50 | 1 | string |  |
| record | dispel-effect-type-51 | 1 | enum-like string ❓ | dispel-soul-mask |
| record | dispel-effect-type-52 | 1 | string |  |
| record | dispel-effect-type-53 | 1 | string |  |
| record | dispel-effect-type-54 | 1 | string |  |
| record | dispel-effect-type-55 | 1 | string |  |
| record | dispel-effect-type-56 | 1 | string |  |
| record | dispel-effect-type-57 | 1 | string |  |
| record | dispel-effect-type-6 | 1 | string |  |
| record | dispel-effect-type-7 | 1 | string |  |
| record | dispel-effect-type-8 | 1 | string |  |
| record | dispel-effect-type-9 | 1 | string |  |
| record | duel-bot-spawn-delay | 1 | int |  |
| record | duel-bot-spawn-pos | 1 | string |  |
| record | duel-bot-tier-effect-2-1 | 1 | string |  |
| record | duel-bot-tier-effect-3-1 | 1 | string |  |
| record | duel-bot-tier-effect-4-1 | 1 | string |  |
| record | duel-bot-tier-effect-5-1 | 1 | string |  |
| record | duel-bot-tier-group-1 | 1 | string |  |
| record | duel-bot-tier-group-10 | 1 | string |  |
| record | duel-bot-tier-group-11 | 1 | string |  |
| record | duel-bot-tier-group-12 | 1 | string |  |
| record | duel-bot-tier-group-13 | 1 | string |  |
| record | duel-bot-tier-group-14 | 1 | string |  |
| record | duel-bot-tier-group-15 | 1 | string |  |
| record | duel-bot-tier-group-16 | 1 | string |  |
| record | duel-bot-tier-group-17 | 1 | string |  |
| record | duel-bot-tier-group-18 | 1 | string |  |
| record | duel-bot-tier-group-19 | 1 | string |  |
| record | duel-bot-tier-group-2 | 1 | string |  |
| record | duel-bot-tier-group-20 | 1 | string |  |
| record | duel-bot-tier-group-3 | 1 | string |  |
| record | duel-bot-tier-group-4 | 1 | string |  |
| record | duel-bot-tier-group-5 | 1 | string |  |
| record | duel-bot-tier-group-6 | 1 | string |  |
| record | duel-bot-tier-group-7 | 1 | string |  |
| record | duel-bot-tier-group-8 | 1 | string |  |
| record | duel-bot-tier-group-9 | 1 | string |  |
| record | finish-time-section-1 | 1 | int |  |
| record | finish-time-section-10 | 1 | int |  |
| record | finish-time-section-2 | 1 | int |  |
| record | finish-time-section-3 | 1 | int |  |
| record | finish-time-section-4 | 1 | int |  |
| record | finish-time-section-5 | 1 | int |  |
| record | finish-time-section-6 | 1 | int |  |
| record | finish-time-section-7 | 1 | int |  |
| record | finish-time-section-8 | 1 | int |  |
| record | finish-time-section-9 | 1 | int |  |
| record | finish-time-section-floor-setup-1 | 1 | int |  |
| record | finish-time-section-floor-setup-10 | 1 | int |  |
| record | finish-time-section-floor-setup-2 | 1 | int |  |
| record | finish-time-section-floor-setup-3 | 1 | int |  |
| record | finish-time-section-floor-setup-4 | 1 | int |  |
| record | finish-time-section-floor-setup-5 | 1 | int |  |
| record | finish-time-section-floor-setup-6 | 1 | int |  |
| record | finish-time-section-floor-setup-7 | 1 | int |  |
| record | finish-time-section-floor-setup-8 | 1 | int |  |
| record | finish-time-section-floor-setup-9 | 1 | int |  |
| record | floor-timeout-duration-second | 1 | int |  |
| record | group | 1 | string |  |
| record | id | 1 | int |  |
| record | leave-pc-spawn | 1 | string |  |
| record | max-floor | 1 | int |  |
| record | ready-duration | 1 | int |  |
| record | required-level | 1 | int |  |
| record | required-mastery-level | 1 | int |  |
| record | resurrection-effect-1 | 1 | string |  |
| record | resurrection-effect-2 | 1 | string |  |
| record | strategic-tool-1 | 1 | string |  |
| record | strategic-tool-10 | 1 | string |  |
| record | strategic-tool-11 | 1 | string |  |
| record | strategic-tool-12 | 1 | string |  |
| record | strategic-tool-13 | 1 | string |  |
| record | strategic-tool-2 | 1 | string |  |
| record | strategic-tool-3 | 1 | string |  |
| record | strategic-tool-4 | 1 | string |  |
| record | strategic-tool-5 | 1 | string |  |
| record | strategic-tool-6 | 1 | string |  |
| record | strategic-tool-7 | 1 | string |  |
| record | strategic-tool-8 | 1 | string |  |
| record | strategic-tool-9 | 1 | string |  |
| record | total-round | 1 | int |  |
| record | total-timeout-duration-second | 1 | int |  |
| record | type | 1 | enum-like string ❓ | round-mode |
| record | zone | 1 | int |  |

### duel-bot-challenge-floor

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 100 | string |  |
| record | floor | 100 | int |  |
| record | id | 100 | int |  |
| record | reward-exp | 100 | int |  |
| record | reward-item-1 | 100 | string |  |
| record | reward-item-2 | 30 | string |  |
| record | reward-item-count-1 | 100 | int |  |
| record | reward-item-count-2 | 30 | int |  |
| record | reward-item-count-for-system-failure-1 | 100 | int |  |
| record | reward-item-for-system-failure-1 | 100 | string |  |
| record | start-floor | 100 | int |  |

### duel-bot-challenge-strategic-tool

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 20 | string |  |
| record | duel-bot-effect-1 | 7 | string |  |
| record | duel-bot-effect-2 | 1 | string |  |
| record | fail-caster-effect-attribute-1 | 6 | string |  |
| record | fail-caster-effect-attribute-2 | 6 | string |  |
| record | fail-caster-effect-attribute-op | 6 | enum-like string ❓ | or |
| record | fail-caster-flag-1 | 6 | enum-like string ❓ | fast-freezing |
| record | fail-caster-flag-2 | 6 | string |  |
| record | fail-caster-flag-3 | 6 | string |  |
| record | fail-caster-flag-4 | 6 | string |  |
| record | fail-caster-op | 6 | enum-like string ❓ | or |
| record | immune-caster-block-skill | 6 | enum-like string ❓ | y |
| record | pc-effect-1 | 14 | string |  |
| record | pc-effect-2 | 1 | string |  |

### duel-bot-equipped-item

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 280 | string |  |
| record | hand | 280 | string |  |

### duel-bot-group

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 29 | string |  |
| record | duel-bot-1 | 29 | string |  |
| record | duel-bot-10 | 9 | string |  |
| record | duel-bot-2 | 29 | string |  |
| record | duel-bot-3 | 29 | string |  |
| record | duel-bot-4 | 29 | string |  |
| record | duel-bot-5 | 29 | string |  |
| record | duel-bot-6 | 29 | string |  |
| record | duel-bot-7 | 29 | string |  |
| record | duel-bot-8 | 29 | string |  |
| record | duel-bot-9 | 23 | string |  |

### duel-bot-trained-skill

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 9 | string |  |
| record | trained-active-skill-1 | 9 | string |  |
| record | trained-active-skill-10 | 9 | string |  |
| record | trained-active-skill-11 | 9 | string |  |
| record | trained-active-skill-12 | 9 | string |  |
| record | trained-active-skill-13 | 9 | string |  |
| record | trained-active-skill-14 | 9 | string |  |
| record | trained-active-skill-15 | 9 | string |  |
| record | trained-active-skill-16 | 9 | string |  |
| record | trained-active-skill-17 | 9 | string |  |
| record | trained-active-skill-18 | 9 | string |  |
| record | trained-active-skill-19 | 9 | string |  |
| record | trained-active-skill-2 | 9 | string |  |
| record | trained-active-skill-20 | 9 | string |  |
| record | trained-active-skill-21 | 9 | string |  |
| record | trained-active-skill-22 | 9 | string |  |
| record | trained-active-skill-23 | 9 | string |  |
| record | trained-active-skill-24 | 9 | string |  |
| record | trained-active-skill-25 | 9 | string |  |
| record | trained-active-skill-26 | 9 | string |  |
| record | trained-active-skill-27 | 9 | string |  |
| record | trained-active-skill-28 | 9 | string |  |
| record | trained-active-skill-29 | 9 | string |  |
| record | trained-active-skill-3 | 9 | string |  |
| record | trained-active-skill-30 | 9 | string |  |
| record | trained-active-skill-31 | 8 | string |  |
| record | trained-active-skill-32 | 7 | string |  |
| record | trained-active-skill-33 | 7 | string |  |
| record | trained-active-skill-34 | 7 | string |  |
| record | trained-active-skill-35 | 7 | string |  |
| record | trained-active-skill-36 | 7 | string |  |
| record | trained-active-skill-37 | 7 | string |  |
| record | trained-active-skill-38 | 7 | string |  |
| record | trained-active-skill-39 | 7 | string |  |
| record | trained-active-skill-4 | 9 | string |  |
| record | trained-active-skill-40 | 6 | string |  |
| record | trained-active-skill-41 | 6 | string |  |
| record | trained-active-skill-42 | 6 | string |  |
| record | trained-active-skill-43 | 6 | string |  |
| record | trained-active-skill-44 | 6 | string |  |
| record | trained-active-skill-45 | 3 | string |  |
| record | trained-active-skill-46 | 3 | string |  |
| record | trained-active-skill-47 | 3 | string |  |
| record | trained-active-skill-48 | 2 | string |  |
| record | trained-active-skill-49 | 1 | string |  |
| record | trained-active-skill-5 | 9 | string |  |
| record | trained-active-skill-50 | 1 | string |  |
| record | trained-active-skill-6 | 9 | string |  |
| record | trained-active-skill-7 | 9 | string |  |
| record | trained-active-skill-8 | 9 | string |  |
| record | trained-active-skill-9 | 9 | string |  |
| record | trained-passive-skill-1 | 6 | string |  |

### duel-bot-training-room

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | achievement-time-limit-second-1 | 1 | int |  |
| record | achievement-time-limit-second-2 | 1 | int |  |
| record | alias | 1 | string |  |
| record | assassin-group | 1 | string |  |
| record | blade-master-group | 1 | string |  |
| record | destroyer-group | 1 | string |  |
| record | duel-bot-spawn-pos | 1 | string |  |
| record | force-master-group | 1 | string |  |
| record | group | 1 | string |  |
| record | id | 1 | int |  |
| record | kung-fu-fighter-group | 1 | string |  |
| record | leave-pc-spawn | 1 | string |  |
| record | required-level | 1 | int |  |
| record | required-mastery-level | 1 | int |  |
| record | resurrection-effect-1 | 1 | string |  |
| record | resurrection-effect-2 | 1 | string |  |
| record | soul-fighter-group | 1 | string |  |
| record | summoner-group | 1 | string |  |
| record | sword-master-group | 1 | string |  |
| record | timeout-duration-second | 1 | int |  |
| record | warlock-group | 1 | string |  |
| record | zone | 1 | int |  |

### duel-bot-training-room-reward

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | achievement | 270 | int |  |
| record | grade | 270 | int |  |
| record | job | 270 | string |  |
| record | reward-exp | 270 | int |  |
| record | reward-item-1 | 270 | string |  |
| record | reward-item-count-1 | 270 | int |  |

### duel-bot-training-room-version

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 1 | string |  |
| record | version | 1 | int |  |

### duel-grade

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | duel-point-bonus-percent | 5 | int |  |
| record | grade | 5 | int |  |
| record | score | 5 | int |  |

### duel-zone

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 1 | string |  |
| record | id | 1 | int |  |
| record | zone-1 | 1 | int |  |
| record | zone-2 | 1 | int |  |

### dungeon

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 51 | string |  |
| record | apply-contents-ban-id | 51 | int |  |
| record | arena-entrance-zone | 43 | string |  |
| record | dungeon-quest-difficulty-type-1 | 51 | string |  |
| record | dungeon-quest-difficulty-type-2 | 11 | string |  |
| record | dungeon-quest-difficulty-type-3 | 5 | string |  |
| record | dungeon-type | 51 | enum-like string ❓ | bind, unbind |
| record | effect-npc-difficulty-type-1 | 19 | string |  |
| record | effect-npc-difficulty-type-2 | 14 | string |  |
| record | effect-pc-difficulty-type-1 | 26 | string |  |
| record | effect-pc-difficulty-type-2 | 3 | string |  |
| record | enable-heart-count | 49 | enum-like string ❓ | n, y |
| record | enable-skill-score | 41 | enum-like string ❓ | y |
| record | group | 49 | string |  |
| record | id | 51 | int |  |
| record | item-count-for-step-6 | 2 | int |  |
| record | item-for-step-6 | 2 | string |  |
| record | max-instant-heart-count-difficulty-type-1 | 49 | int |  |
| record | max-instant-heart-count-difficulty-type-2 | 49 | int |  |
| record | max-instant-heart-count-difficulty-type-3 | 49 | int |  |
| record | npc-for-step-2 | 2 | string |  |
| record | npc-for-step-6 | 2 | string |  |
| record | pc-spawn-for-step-2 | 2 | int |  |
| record | pc-spawn-for-step-6 | 2 | int |  |
| record | quest-for-step-6 | 2 | string |  |
| record | required-level | 46 | int |  |
| record | required-preceding-quest-1 | 25 | string |  |
| record | required-preceding-quest-2 | 1 | string |  |
| record | required-preceding-quest-check | 1 | enum-like string ❓ | or |
| record | skill-score-boss-npc | 45 | string |  |
| record | skill-score-boss-npc-bonus | 41 | int |  |
| record | skill-score-recommand-party-member-count-bonus-difficulty-type-1 | 41 | int |  |
| record | skill-score-recommand-party-member-count-bonus-difficulty-type-2 | 41 | int |  |
| record | skill-score-recommand-party-member-count-bonus-difficulty-type-3 | 41 | int |  |
| record | skill-score-recommend-clear-time-bonus-1 | 41 | int |  |
| record | skill-score-recommend-clear-time-bonus-2 | 41 | int |  |
| record | skill-score-recommend-clear-time-bonus-3 | 41 | int |  |
| record | skill-score-recommend-clear-time-bonus-4 | 41 | int |  |
| record | skill-score-recommend-clear-time-bonus-5 | 41 | int |  |
| record | skill-score-recommend-clear-time-step-1 | 41 | int |  |
| record | skill-score-recommend-clear-time-step-2 | 41 | int |  |
| record | skill-score-recommend-clear-time-step-3 | 41 | int |  |
| record | skill-score-recommend-clear-time-step-4 | 41 | int |  |
| record | skill-score-recommend-clear-time-step-5 | 41 | int |  |
| record | skill-score-recommend-gear-score | 41 | int |  |
| record | skill-score-recommend-hp-bonus-1 | 41 | int |  |
| record | skill-score-recommend-hp-bonus-2 | 41 | int |  |
| record | skill-score-recommend-hp-bonus-3 | 41 | int |  |
| record | skill-score-recommend-hp-bonus-4 | 41 | int |  |
| record | skill-score-recommend-hp-bonus-5 | 41 | int |  |
| record | skill-score-recommend-hp-step-1 | 41 | int |  |
| record | skill-score-recommend-hp-step-2 | 41 | int |  |
| record | skill-score-recommend-hp-step-3 | 41 | int |  |
| record | skill-score-recommend-hp-step-4 | 41 | int |  |
| record | skill-score-recommend-hp-step-5 | 41 | int |  |
| record | skill-score-recommend-use-heart-count | 41 | int |  |
| record | skill-score-recommend-use-heart-count-bonus | 41 | int |  |
| record | skill-score-recommend-use-item-bonus | 41 | int |  |
| record | skill-score-recommend-use-item-point | 41 | int |  |
| record | zone-1 | 51 | string |  |
| record | zone-2 | 34 | string |  |
| record | zone-3 | 20 | string |  |
| record | zone-4 | 9 | string |  |
| record | zone-5 | 4 | string |  |
| record | zone-6 | 2 | string |  |
| record | zone-7 | 2 | string |  |
| record | zone-8 | 2 | string |  |
| record | zone-9 | 1 | string |  |
| record | zone-mission-step-1 | 51 | int |  |
| record | zone-mission-step-2 | 35 | int |  |
| record | zone-mission-step-3 | 20 | int |  |
| record | zone-mission-step-4 | 9 | int |  |
| record | zone-mission-step-5 | 4 | int |  |
| record | zone-mission-step-6 | 2 | int |  |
| record | zone-mission-step-7 | 2 | int |  |
| record | zone-mission-step-8 | 2 | int |  |
| record | zone-mission-step-9 | 1 | int |  |
| record | zone-neutral-1 | 2 | string |  |

### effect

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | absorb-last-damage | 46 | enum-like string ❓ | y |
| record | accumulated-damage-explosion-subject-type | 12 | enum-like string ❓ | caster, target |
| record | accumulated-damage-explosion-target-slot | 12 | string |  |
| record | accumulated-damage-explosion-use-percent | 12 | int |  |
| record | accumulated-damage-explosion-use-type | 12 | enum-like string ❓ | hp-damage, hp-heal |
| record | accumulated-damage-type | 33 | enum-like string ❓ | from-anyone, from-someone, to-anyone |
| record | additional-power-diff | 3 | string |  |
| record | additional-power-percent | 22 | string |  |
| record | alias | 86147 | string |  |
| record | aoe-damage | 2407 | enum-like string ❓ | y |
| record | apply-duration-formula | 1971 | enum-like string ❓ | n, y |
| record | apply-hate | 3 | enum-like string ❓ | n |
| record | arm-effect-1 | 5 | enum-like string ❓ | none |
| record | arm-effect-2 | 14 | enum-like string ❓ | none |
| record | attack-attribute-coefficient-percent | 6985 | int |  |
| record | attribute-1 | 14287 | string |  |
| record | attribute-2 | 4908 | string |  |
| record | attribute-3 | 535 | string |  |
| record | attribute-4 | 29 | string |  |
| record | attribute-5 | 5 | string |  |
| record | basecamp | 11 | string |  |
| record | binding-skill-step-type | 398 | string |  |
| record | broadcast-type | 67572 | enum-like string ❓ | normal, self, y |
| record | buff-type | 86142 | string |  |
| record | cast-speed-percent | 50 | string |  |
| record | caster-dash | 636 | string |  |
| record | catch-caster-effect | 37 | string |  |
| record | catch-linked | 5452 | enum-like string ❓ | attach, dispel, not-attach |
| record | catch-linker | 5495 | enum-like string ❓ | attach, dispel, not-attach |
| record | catch-target-effect | 18 | string |  |
| record | change-default-stance-by-detach-timeout | 79 | enum-like string ❓ | y |
| record | class-recycle-group-id | 1013 | int |  |
| record | class-recycle-group-modify-time | 855 | string |  |
| record | class2-recycle-group-id | 177 | string |  |
| record | class2-recycle-group-modify-time | 174 | int |  |
| record | combat-mode-counting | 4 | enum-like string ❓ | n, y |
| record | content-quota-id | 2 | int |  |
| record | death-probability | 112 | int |  |
| record | desire-attack-normal-diff | 57 | int |  |
| record | detach-angle-by-dispeled | 65 | int |  |
| record | detach-angle-by-timeout | 64 | int |  |
| record | detach-by-dispel-effect-1 | 1170 | string |  |
| record | detach-by-dispel-effect-2 | 23 | string |  |
| record | detach-by-dispel-effect-3 | 1 | string |  |
| record | detach-by-rule-effect-1 | 1102 | string |  |
| record | detach-by-rule-effect-2 | 27 | string |  |
| record | detach-by-rule-effect-3 | 1 | string |  |
| record | detach-by-timeout-effect-1 | 3883 | string |  |
| record | detach-by-timeout-effect-2 | 39 | string |  |
| record | detach-by-timeout-effect-3 | 10 | string |  |
| record | detach-by-timeout-effect-4 | 4 | string |  |
| record | detach-count | 2307 | int |  |
| record | detach-effect | 2 | string |  |
| record | detach-margin-by-dispeled | 65 | int |  |
| record | detach-margin-by-timeout | 64 | int |  |
| record | detach-slot-1 | 24252 | string |  |
| record | detach-slot-2 | 22293 | string |  |
| record | detach-slot-3 | 25 | enum-like string ❓ | attack-bounce, attack-critical, attacked-hit, attacked-miss, attacked-perfect-parry |
| record | detach-slot-4 | 1 | enum-like string ❓ | attacked-critical |
| record | dispel-attribute-1 | 309 | string |  |
| record | dispel-attribute-2 | 114 | string |  |
| record | dispel-attribute-3 | 67 | string |  |
| record | dispel-attribute-4 | 43 | string |  |
| record | dispel-attribute-5 | 1 | string |  |
| record | dispel-attribute-6 | 1 | string |  |
| record | dispel-attribute-effect | 25 | string |  |
| record | dispel-effect-1 | 132 | string |  |
| record | dispel-effect-2 | 4 | string |  |
| record | dispel-level-1 | 52 | int |  |
| record | dispel-type-1 | 2118 | string |  |
| record | dispel-type-2 | 28 | string |  |
| record | dispel-type-3 | 17 | string |  |
| record | dispel-type-4 | 10 | string |  |
| record | do-not-apply-block-move | 6 | string |  |
| record | do-not-detach-by-rule-dispel | 589 | enum-like string ❓ | n, y |
| record | drain-hp-diff | 2 | int |  |
| record | drain-hp-percent | 1050 | int |  |
| record | drain-sp-diff-1 | 215 | string |  |
| record | drop-field-item | 22555 | enum-like string ❓ | n, y |
| record | effect-region-leaved-detach | 13 | enum-like string ❓ | n, y |
| record | effect-type | 78 | string |  |
| record | enable-duration | 4 | string |  |
| record | event-effect-1 | 23404 | string |  |
| record | event-effect-2 | 4350 | string |  |
| record | event-effect-3 | 1310 | string |  |
| record | event-effect-4 | 513 | string |  |
| record | event-effect-target-1 | 22617 | enum-like string ❓ | effect-eventer, effect-giver, effect-owner, skill-target |
| record | event-effect-target-2 | 4398 | enum-like string ❓ | effect-eventer, effect-giver, effect-owner, skill-target |
| record | event-effect-target-3 | 1324 | enum-like string ❓ | effect-eventer, effect-giver, effect-owner, skill-target |
| record | event-effect-target-4 | 515 | enum-like string ❓ | effect-eventer, effect-giver, effect-owner, skill-target |
| record | exhaustion | 10 | enum-like string ❓ | n, y |
| record | expiration-duration-sec | 30 | int |  |
| record | extend-duration | 144 | int |  |
| record | faction-score-max | 6 | int |  |
| record | faction-score-min | 6 | int |  |
| record | fall-through-use-percent | 29 | int |  |
| record | fall-through-use-type | 29 | enum-like string ❓ | hp-damage, hp-heal |
| record | finish | 49 | string |  |
| record | flag-1 | 8147 | string |  |
| record | flag-2 | 86 | string |  |
| record | flag-3 | 5 | string |  |
| record | force-hp-damage | 6 | enum-like string ❓ | y |
| record | fourth-slot-event-effect-1 | 1 | string |  |
| record | fourth-slot-event-effect-target-1 | 1 | enum-like string ❓ | effect-owner |
| record | function-1 | 8919 | enum-like string ❓ | block-dodge, block-hyper, block-jump, block-move, block-parry, block-rotate, block-skill, front-bounce, front-perfect-parry, immune-damage, immune-death, immune-debuff, n |
| record | function-2 | 7574 | enum-like string ❓ | back-bounce, back-perfect-parry, block-bounce, block-dodge, block-force-skill, block-jump, block-move, block-parry, block-perfect-parry, block-rotate, block-skill, front-perfect-parry, immune-damage, immune-debuff |
| record | function-3 | 5671 | enum-like string ❓ | back-perfect-parry, block-bounce, block-dodge, block-force-skill, block-hyper, block-jump, block-move, block-parry, block-perfect-parry, block-rotate, block-skill, front-perfect-parry, immune-damage, immune-debuff |
| record | function-4 | 5439 | enum-like string ❓ | back-perfect-parry, block-bounce, block-dodge, block-force-skill, block-jump, block-move, block-parry, block-perfect-parry, block-rotate, block-skill, front-perfect-parry, immune-damage, immune-debuff |
| record | function-5 | 4998 | enum-like string ❓ | back-perfect-parry, block-bounce, block-dodge, block-jump, block-move, block-parry, block-rotate, block-skill, front-perfect-parry, immune-damage, immune-debuff |
| record | function-6 | 3383 | enum-like string ❓ | block-bounce, block-dodge, block-force-skill, block-move, block-parry, block-skill, immune-damage, immune-debuff |
| record | function-7 | 73 | enum-like string ❓ | block-bounce, block-parry, block-perfect-parry, immune-damage, immune-debuff |
| record | function-8 | 24 | enum-like string ❓ | block-parry, immune-damage, immune-debuff, y |
| record | ghost-type | 7 | enum-like string ❓ | phase |
| record | gp-diff | 59 | int |  |
| record | gp-fix | 51 | int |  |
| record | guard-gauge-diff | 1 | int |  |
| record | guard-gauge-percent | 65 | int |  |
| record | hate-diff | 110 | int |  |
| record | hate-diff-value | 22 | int |  |
| record | hate-multiply-value | 161 | int |  |
| record | head-effect-1 | 1 | string |  |
| record | heal-hp-percent | 32 | int |  |
| record | heart-count | 3 | string |  |
| record | hp-diff | 5595 | string |  |
| record | hp-percent | 1247 | int |  |
| record | id | 86147 | int |  |
| record | ignore-guard-gauge | 83 | enum-like string ❓ | y |
| record | immune-attribute-1 | 2770 | string |  |
| record | immune-attribute-2 | 939 | string |  |
| record | immune-attribute-3 | 681 | string |  |
| record | immune-attribute-4 | 380 | string |  |
| record | immune-attribute-5 | 107 | string |  |
| record | immune-attribute-6 | 98 | string |  |
| record | immune-attribute-7 | 16 | string |  |
| record | immune-attribute-8 | 4 | string |  |
| record | immune-breaker-attribute | 1247 | string |  |
| record | immune-breaker-duration | 10 | int |  |
| record | increase-amount | 2032 | string |  |
| record | inhalation-linked | 5478 | enum-like string ❓ | attach, dispel, not-attach |
| record | inhalation-linker | 5521 | enum-like string ❓ | attach, dispel, not-attach |
| record | initialize-recycle-group-1 | 300 | string |  |
| record | initialize-recycle-group-2 | 30 | string |  |
| record | initialize-recycle-group-id-1 | 289 | int |  |
| record | initialize-recycle-group-id-2 | 31 | string |  |
| record | instant-heart-count | 6 | int |  |
| record | knockback-direction | 1798 | string |  |
| record | knockback-speed | 11769 | int |  |
| record | knockback-type | 1770 | string |  |
| record | land-target | 2 | enum-like string ❓ | y |
| record | leave-zone-detach | 2837 | enum-like string ❓ | n, y |
| record | leg-effect-2 | 8 | enum-like string ❓ | none |
| record | level | 82765 | int |  |
| record | link-stage | 41021 | string |  |
| record | link-type | 41021 | string |  |
| record | lock-on-target-id | 7 | enum-like string ❓ | n |
| record | max-accumulate-damage | 33 | int |  |
| record | max-hp-diff | 1 | int |  |
| record | max-hp-percent | 20 | int |  |
| record | max-sp-percent-1 | 2 | int |  |
| record | max-stack-flag | 250 | string |  |
| record | miss-probability | 28497 | int |  |
| record | modify-ability-1 | 24955 | string |  |
| record | modify-ability-2 | 14386 | string |  |
| record | modify-ability-3 | 7060 | string |  |
| record | modify-ability-4 | 3359 | string |  |
| record | modify-ability-5 | 1366 | string |  |
| record | modify-ability-6 | 895 | string |  |
| record | modify-ability-7 | 532 | enum-like string ❓ | abnormal-attack-power-value, attack-attribute-value-equip, attack-critical-damage-percent, attack-critical-damage-value, attack-critical-value, attack-damage-modify-diff, attack-power-equip-max, attack-power-equip-min, defend-critical-value-equip, defend-power-equip-value, heal-power-base-percent, pvp-attack-power-equip-max, speed |
| record | modify-ability-8 | 31 | enum-like string ❓ | attack-attribute-value-equip, attack-critical-value-equip, attack-power-equip-max, defend-damage-modify-percent, pvp-defend-power-equip-value |
| record | modify-ability-diff-1 | 23332 | int |  |
| record | modify-ability-diff-2 | 13444 | int |  |
| record | modify-ability-diff-3 | 6707 | int |  |
| record | modify-ability-diff-4 | 3102 | int |  |
| record | modify-ability-diff-5 | 1323 | int |  |
| record | modify-ability-diff-6 | 854 | int |  |
| record | modify-ability-diff-7 | 526 | int |  |
| record | modify-ability-diff-8 | 32 | int |  |
| record | modify-ability-diff-for-mastery-11 | 3 | enum-like string ❓ | attach |
| record | modify-ability-diff-for-mastery-8 | 8 | enum-like string ❓ | no-all-message |
| record | modify-ability-percent-1 | 1705 | int |  |
| record | modify-ability-percent-2 | 946 | int |  |
| record | modify-ability-percent-3 | 381 | int |  |
| record | modify-ability-percent-4 | 307 | int |  |
| record | modify-ability-percent-5 | 101 | int |  |
| record | modify-ability-percent-6 | 42 | int |  |
| record | modify-ability-percent-7 | 6 | int |  |
| record | modify-ability-percent-8 | 1 | int |  |
| record | modify-ability-percent-for-mastery-14 | 2 | enum-like string ❓ | n |
| record | modify-ability-percent-for-mastery-15 | 3 | enum-like string ❓ | n |
| record | modify-ability-percent-for-mastery-2 | 4 | enum-like string ❓ | y |
| record | modify-ability-percent-for-mastery-5 | 2 | enum-like string ❓ | y |
| record | modify-ability-percent-for-mastery-6 | 4 | enum-like string ❓ | y |
| record | mount-linked | 5478 | enum-like string ❓ | attach, dispel, not-attach |
| record | mount-linker | 5521 | enum-like string ❓ | attach, dispel, not-attach |
| record | move-speed-percent | 6 | int |  |
| record | not-dead-to-exhaustion-detach | 281 | enum-like string ❓ | n, y |
| record | passive-duration | 35737 | int |  |
| record | passive-interval | 2078 | int |  |
| record | pc-spawn | 19 | int |  |
| record | power-diff-max | 99 | int |  |
| record | power-diff-min | 100 | int |  |
| record | power-percent-max | 14999 | int |  |
| record | power-percent-min | 14999 | int |  |
| record | reattach-effect-after-changing-stack-count | 191 | string |  |
| record | save-db | 16047 | enum-like string ❓ | n, y |
| record | second-slot-event-effect-1 | 21417 | string |  |
| record | second-slot-event-effect-2 | 4074 | string |  |
| record | second-slot-event-effect-3 | 1257 | string |  |
| record | second-slot-event-effect-4 | 479 | string |  |
| record | second-slot-event-effect-target-1 | 21440 | enum-like string ❓ | effect-eventer, effect-giver, effect-owner, skill-target |
| record | second-slot-event-effect-target-2 | 4081 | enum-like string ❓ | effect-eventer, effect-giver, effect-owner, skill-target |
| record | second-slot-event-effect-target-3 | 1263 | enum-like string ❓ | effect-eventer, effect-owner, skill-target |
| record | second-slot-event-effect-target-4 | 483 | enum-like string ❓ | effect-giver, effect-owner, skill-target |
| record | shout-power-percent | 6 | int |  |
| record | skill3 | 43 | string |  |
| record | soul-mask-id | 22 | int |  |
| record | sp-diff-1 | 380 | int |  |
| record | sp-percent-1 | 3430 | int |  |
| record | speed-percent | 87 | int |  |
| record | stack-amount | 3976 | int |  |
| record | stack-count | 698 | int |  |
| record | stance | 53042 | string |  |
| record | target-combat-mode | 293 | enum-like string ❓ | n, y |
| record | target-yaw-to-caster | 51 | enum-like string ❓ | y |
| record | third-slot-event-effect-1 | 25 | string |  |
| record | third-slot-event-effect-2 | 19 | string |  |
| record | third-slot-event-effect-3 | 3 | string |  |
| record | third-slot-event-effect-target-1 | 25 | enum-like string ❓ | effect-eventer, effect-owner, skill-target |
| record | third-slot-event-effect-target-2 | 19 | enum-like string ❓ | effect-owner |
| record | third-slot-event-effect-target-3 | 3 | enum-like string ❓ | effect-eventer, effect-owner |
| record | transform-effect | 1774 | string |  |
| record | tumbling-direction | 474 | string |  |
| record | tumbling-speed | 473 | int |  |
| record | type | 86147 | string |  |
| record | value | 2 | int |  |
| record | wall-effect-1 | 178 | string |  |
| record | zone-id | 34061 | int |  |
| record | zone-score | 7 | int |  |
| record | zone-score-attraction | 7 | string |  |

### effect-group

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 39442 | string |  |
| record | caster-state-first-filter | 1267 | string |  |
| record | caster-state-second-filter | 100 | string |  |
| record | change-check-state-target | 33 | enum-like string ❓ | n |
| record | effect-1 | 38786 | string |  |
| record | effect-2 | 10516 | string |  |
| record | effect-3 | 4856 | string |  |
| record | effect-4 | 1946 | string |  |
| record | effect-5 | 819 | string |  |
| record | effect-6 | 340 | string |  |
| record | effect-7 | 197 | string |  |
| record | effect-8 | 93 | string |  |
| record | individual-probability | 1564 | enum-like string ❓ | n, y |
| record | probability-1 | 15 | int |  |
| record | probability-2 | 16 | int |  |
| record | probability-3 | 12 | int |  |
| record | probability-4 | 5 | int |  |
| record | probability-5 | 2 | int |  |
| record | probability-6 | 2 | int |  |
| record | skill-result | 39434 | string |  |
| record | target | 39436 | string |  |
| record | target-state | 39433 | string |  |
| record | target-state-first-filter | 8483 | string |  |
| record | target-state-second-filter | 1996 | string |  |

### energy-point

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | acquired-max-energy-multiple-point | 1 | int |  |
| record | acquired-max-energy-point | 1 | int |  |
| record | enable-energy-point | 1 | enum-like string ❓ | n |
| record | id | 1 | int |  |
| record | multiple-value-1 | 1 | int |  |
| record | multiple-value-2 | 1 | int |  |
| record | multiple-value-3 | 1 | int |  |
| record | ratio-of-point-to-exp | 1 | int |  |
| record | reset-hour | 1 | int |  |
| record | reward-count | 1 | int |  |

### energy-point-reward

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | exp | 5 | int |  |
| record | id | 5 | int |  |
| record | max-acquire-count | 5 | int |  |
| record | require-energy-point | 5 | int |  |
| record | reward-type | 5 | enum-like string ❓ | exp |

### equip-gem-piece

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 8013 | string |  |
| record | id | 8013 | int |  |
| record | item-ability | 8013 | enum-like string ❓ | attack-concentrate-value, attack-critical-damage-percent, attack-critical-value, attack-hit-value, attack-pierce-value, defend-critical-value, defend-dodge-value, defend-parry-value, defend-power-equip-value, max-hp |
| record | value | 8013 | int |  |

### event-skill

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | event-type | 15 | enum-like string ❓ | attack, attacked |
| record | skill-1-1 | 1 | int |  |
| record | skill-1-2 | 13 | int |  |
| record | skill-2-2 | 1 | int |  |
| record | skill-2-4 | 1 | int |  |
| record | skill-2-8 | 1 | int |  |
| record | skill-result | 15 | enum-like string ❓ | bounce, counter, critical-hit, dodge, miss, parry, perfect-parry |
| record | stance-type | 15 | string |  |
| record | state | 15 | enum-like string ❓ | none |

### expand-inventory

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | charge-type | 45 | enum-like string ❓ | item, money |
| record | desirable-expand-level | 45 | int |  |
| record | expand-size | 45 | int |  |
| record | inventory-size | 45 | int |  |
| record | inventory-type | 45 | enum-like string ❓ | depository, inventory, premium-depository, wardrobe |
| record | level | 45 | int |  |
| record | money | 34 | int |  |
| record | required-item | 11 | string |  |
| record | required-item-count | 11 | int |  |

### expand-random-store

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | random-store-number | 6 | string |  |
| record | required-item | 6 | string |  |
| record | required-item-count | 6 | int |  |
| record | required-item-for-instant-payment | 6 | string |  |
| record | slot-index | 6 | int |  |

### expand-skill-page

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | charge-type | 4 | enum-like string ❓ | item |
| record | required-item-1 | 4 | string |  |
| record | required-item-2 | 4 | string |  |
| record | required-item-count-1 | 4 | int |  |
| record | required-item-count-2 | 4 | int |  |
| record | skill-page | 5 | int |  |

### faction

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 14 | string |  |
| record | category | 14 | enum-like string ❓ | gang, main, second |
| record | hostile-faction | 14 | int |  |
| record | id | 14 | int |  |
| record | main-faction | 2 | enum-like string ❓ | y |

### faction-battle-field-zone

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 1 | string |  |
| record | group | 1 | string |  |
| record | id | 1 | int |  |
| record | required-level | 1 | int |  |
| record | revival-effect | 1 | string |  |
| record | zone | 1 | int |  |

### faction-level

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | level | 16 | int |  |
| record | max-faction-score | 16 | int |  |
| record | reputation | 16 | int |  |

### field-zone

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 5 | string |  |
| record | group | 5 | string |  |
| record | id | 5 | int |  |
| record | type | 5 | enum-like string ❓ | normal |
| record | zone-1 | 5 | int |  |
| record | zone-2 | 2 | int |  |
| record | zone-3 | 2 | int |  |
| record | zone-4 | 1 | int |  |

### fielditem

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 832 | string |  |
| record | change-activator-by-killer | 5 | enum-like string ❓ | y |
| record | consume-use-count-1 | 452 | int |  |
| record | consume-use-count-2 | 38 | int |  |
| record | consume-use-count-3 | 11 | int |  |
| record | consume-use-count-4 | 2 | int |  |
| record | equip-effect-1 | 65 | string |  |
| record | equip-effect-2 | 17 | string |  |
| record | equip-effect-3 | 1 | string |  |
| record | equip-effect-4 | 138 | string |  |
| record | explosion-time | 832 | int |  |
| record | first-effect-1 | 13 | string |  |
| record | first-effect-2 | 9 | string |  |
| record | first-effect-3 | 4 | string |  |
| record | first-effect-4 | 3 | string |  |
| record | id | 832 | int |  |
| record | init-activated | 832 | enum-like string ❓ | n, y |
| record | invoke-effect-1 | 342 | string |  |
| record | invoke-effect-2 | 214 | string |  |
| record | invoke-effect-3 | 45 | string |  |
| record | invoke-effect-4 | 36 | string |  |
| record | not-change-create-pos | 830 | enum-like string ❓ | n, y |
| record | pickup-duration | 832 | int |  |
| record | putdown-duration | 832 | int |  |
| record | remain-duration | 832 | int |  |
| record | second-effect-1 | 2 | string |  |
| record | target-first-filter-1 | 13 | int |  |
| record | target-first-filter-2 | 10 | int |  |
| record | target-first-filter-3 | 4 | int |  |
| record | target-first-filter-4 | 3 | int |  |
| record | target-radius | 243 | int |  |
| record | target-second-filter-4 | 3 | int |  |
| record | target-state | 801 | enum-like string ❓ | all, enemy, friend |
| record | throw-skill | 187 | string |  |
| record | throw-skill3 | 211 | string |  |
| record | type | 832 | enum-like string ❓ | throw, use |
| record | use-count | 669 | int |  |
| record | use-skill-1 | 433 | string |  |
| record | use-skill-2 | 38 | string |  |
| record | use-skill-3 | 10 | string |  |
| record | use-skill-4 | 2 | string |  |
| record | use-skill3-1 | 435 | string |  |
| record | use-skill3-2 | 37 | string |  |
| record | use-skill3-3 | 11 | string |  |
| record | use-skill3-4 | 2 | string |  |

### filter

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 920 | string |  |
| record | count | 96 | int |  |
| record | effect-slot | 104 | enum-like string ❓ | buff, debuff |
| record | effect-type | 104 | string |  |
| record | either | 501 | enum-like string ❓ | n, y |
| record | field | 150 | enum-like string ❓ | block-skill, casting, combat-mode, dead, ghost-type, heart-count, hp, instant-heart-count, job, level, link-stage, link-state, link-type, race, sex, sp, stance |
| record | flag | 312 | enum-like string ❓ | n, y |
| record | flagtype | 302 | string |  |
| record | id | 920 | int |  |
| record | job-1 | 96 | string |  |
| record | job-2 | 96 | string |  |
| record | job-3 | 24 | string |  |
| record | job-style-1 | 96 | string |  |
| record | job-style-2 | 96 | string |  |
| record | job-style-3 | 24 | string |  |
| record | op | 722 | enum-like string ❓ | eq, ge, gt, le, lt, neq |
| record | op-1 | 102 | enum-like string ❓ | eq, ge, gt, neq |
| record | op-2 | 17 | enum-like string ❓ | le, lt |
| record | term-op | 102 | enum-like string ❓ | and, none, or |
| record | type | 920 | enum-like string ❓ | active-faction, effect-attribute, effect-flag, effect-stack-count, job, job-style, not-npc, not-summoned, npc, party-member, pc, prop, prop-flag, prop-percent, race, sex, stance, summoned, weapon-type |
| record | value | 77 | int |  |
| record | value-1 | 813 | string |  |
| record | value-2 | 68 | string |  |
| record | value-3 | 21 | string |  |
| record | value-4 | 9 | string |  |
| record | weapon-type-1 | 623 | enum-like string ❓ | aura-bangle, dagger, gauntlet, none, pistol, rifle, staff, sword, two-handed-axe |

### fishing-field

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 13 | string |  |
| record | bonus-additional-fish-grade-selection-value | 13 | int |  |
| record | bonus-reduce-waiting-time-amount | 13 | int |  |
| record | bonus-time | 5 | string |  |
| record | dispose-fishing-paste-probability | 13 | int |  |
| record | grade-a-boundary-value | 12 | int |  |
| record | grade-a-fish-1 | 12 | string |  |
| record | grade-a-fish-2 | 7 | string |  |
| record | grade-a-fish-3 | 7 | string |  |
| record | grade-a-fish-4 | 7 | string |  |
| record | grade-a-fish-5 | 4 | string |  |
| record | grade-a-fish-prob-weight-1 | 12 | int |  |
| record | grade-a-fish-prob-weight-2 | 7 | int |  |
| record | grade-a-fish-prob-weight-3 | 7 | int |  |
| record | grade-a-fish-prob-weight-4 | 7 | int |  |
| record | grade-a-fish-prob-weight-5 | 4 | int |  |
| record | grade-b-boundary-value | 12 | int |  |
| record | grade-b-fish-1 | 12 | string |  |
| record | grade-b-fish-2 | 11 | string |  |
| record | grade-b-fish-3 | 7 | string |  |
| record | grade-b-fish-4 | 7 | string |  |
| record | grade-b-fish-5 | 7 | string |  |
| record | grade-b-fish-6 | 6 | string |  |
| record | grade-b-fish-7 | 6 | string |  |
| record | grade-b-fish-8 | 5 | string |  |
| record | grade-b-fish-9 | 3 | string |  |
| record | grade-b-fish-prob-weight-1 | 12 | int |  |
| record | grade-b-fish-prob-weight-2 | 11 | int |  |
| record | grade-b-fish-prob-weight-3 | 7 | int |  |
| record | grade-b-fish-prob-weight-4 | 7 | int |  |
| record | grade-b-fish-prob-weight-5 | 7 | int |  |
| record | grade-b-fish-prob-weight-6 | 6 | int |  |
| record | grade-b-fish-prob-weight-7 | 6 | int |  |
| record | grade-b-fish-prob-weight-8 | 5 | int |  |
| record | grade-b-fish-prob-weight-9 | 3 | int |  |
| record | grade-c-boundary-value | 13 | int |  |
| record | grade-c-fish-1 | 13 | string |  |
| record | grade-c-fish-10 | 6 | string |  |
| record | grade-c-fish-11 | 6 | string |  |
| record | grade-c-fish-12 | 6 | string |  |
| record | grade-c-fish-13 | 5 | string |  |
| record | grade-c-fish-2 | 13 | string |  |
| record | grade-c-fish-3 | 13 | string |  |
| record | grade-c-fish-4 | 12 | string |  |
| record | grade-c-fish-5 | 11 | string |  |
| record | grade-c-fish-6 | 7 | string |  |
| record | grade-c-fish-7 | 7 | string |  |
| record | grade-c-fish-8 | 6 | string |  |
| record | grade-c-fish-9 | 6 | string |  |
| record | grade-c-fish-prob-weight-1 | 13 | int |  |
| record | grade-c-fish-prob-weight-10 | 6 | int |  |
| record | grade-c-fish-prob-weight-11 | 6 | int |  |
| record | grade-c-fish-prob-weight-12 | 6 | int |  |
| record | grade-c-fish-prob-weight-13 | 5 | int |  |
| record | grade-c-fish-prob-weight-2 | 13 | int |  |
| record | grade-c-fish-prob-weight-3 | 13 | int |  |
| record | grade-c-fish-prob-weight-4 | 12 | int |  |
| record | grade-c-fish-prob-weight-5 | 11 | int |  |
| record | grade-c-fish-prob-weight-6 | 7 | int |  |
| record | grade-c-fish-prob-weight-7 | 7 | int |  |
| record | grade-c-fish-prob-weight-8 | 6 | int |  |
| record | grade-c-fish-prob-weight-9 | 6 | int |  |
| record | grade-s-boundary-value | 7 | int |  |
| record | grade-s-fish-1 | 7 | string |  |
| record | grade-s-fish-2 | 7 | string |  |
| record | grade-s-fish-3 | 2 | string |  |
| record | grade-s-fish-4 | 1 | string |  |
| record | grade-s-fish-prob-weight-1 | 7 | int |  |
| record | grade-s-fish-prob-weight-2 | 7 | int |  |
| record | grade-s-fish-prob-weight-3 | 2 | int |  |
| record | grade-s-fish-prob-weight-4 | 1 | int |  |
| record | id | 13 | int |  |
| record | waiting-time-max | 13 | int |  |
| record | waiting-time-min | 13 | int |  |

### friend-noti-location

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 331 | string |  |
| record | id | 331 | int |  |
| record | zone1-1 | 324 | int |  |
| record | zone1-10 | 21 | int |  |
| record | zone1-11 | 19 | int |  |
| record | zone1-12 | 17 | int |  |
| record | zone1-13 | 15 | int |  |
| record | zone1-14 | 14 | int |  |
| record | zone1-15 | 13 | int |  |
| record | zone1-16 | 8 | int |  |
| record | zone1-17 | 7 | int |  |
| record | zone1-18 | 7 | int |  |
| record | zone1-19 | 7 | int |  |
| record | zone1-2 | 232 | int |  |
| record | zone1-20 | 7 | int |  |
| record | zone1-21 | 7 | int |  |
| record | zone1-22 | 6 | int |  |
| record | zone1-23 | 6 | int |  |
| record | zone1-24 | 6 | int |  |
| record | zone1-25 | 6 | int |  |
| record | zone1-26 | 6 | int |  |
| record | zone1-27 | 5 | int |  |
| record | zone1-28 | 5 | int |  |
| record | zone1-29 | 5 | int |  |
| record | zone1-3 | 119 | int |  |
| record | zone1-30 | 5 | int |  |
| record | zone1-31 | 4 | int |  |
| record | zone1-32 | 4 | int |  |
| record | zone1-33 | 4 | int |  |
| record | zone1-34 | 4 | int |  |
| record | zone1-35 | 4 | int |  |
| record | zone1-36 | 4 | int |  |
| record | zone1-37 | 3 | int |  |
| record | zone1-38 | 3 | int |  |
| record | zone1-39 | 3 | int |  |
| record | zone1-4 | 92 | int |  |
| record | zone1-40 | 2 | int |  |
| record | zone1-41 | 2 | int |  |
| record | zone1-42 | 2 | int |  |
| record | zone1-43 | 2 | int |  |
| record | zone1-44 | 2 | int |  |
| record | zone1-45 | 2 | int |  |
| record | zone1-46 | 2 | int |  |
| record | zone1-47 | 2 | int |  |
| record | zone1-48 | 2 | int |  |
| record | zone1-49 | 2 | int |  |
| record | zone1-5 | 61 | int |  |
| record | zone1-50 | 2 | int |  |
| record | zone1-51 | 2 | int |  |
| record | zone1-52 | 2 | int |  |
| record | zone1-53 | 2 | int |  |
| record | zone1-54 | 2 | int |  |
| record | zone1-55 | 2 | int |  |
| record | zone1-56 | 2 | int |  |
| record | zone1-57 | 2 | int |  |
| record | zone1-58 | 2 | int |  |
| record | zone1-59 | 2 | int |  |
| record | zone1-6 | 51 | int |  |
| record | zone1-60 | 2 | int |  |
| record | zone1-61 | 2 | int |  |
| record | zone1-62 | 1 | int |  |
| record | zone1-63 | 1 | int |  |
| record | zone1-7 | 39 | int |  |
| record | zone1-8 | 34 | int |  |
| record | zone1-9 | 31 | int |  |

### guild-action

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | action | 19 | enum-like string ❓ | accumulate-guild-point, banish-guild-member, call-guild-member, change-guild-flag-bearer, change-guild-member-grade, contract-guild-craft, contract-guild-uniform, delegate-guild-leader, deposit-in-guild-bank, edit-guild-member-permission, edit-guild-notification-message, guild-join-renewal, guild-level-up, invite-guild, join-guild, leave-guild, rename-guild, warp-to-guild-meeting-place, warp-to-guild-member |
| record | exclusive-enable | 19 | string |  |
| record | open-enable | 19 | string |  |

### guild-bank-item

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | bank-item | 5 | string |  |
| record | bank-item-max-count | 5 | int |  |
| record | bank-item-round-lot | 5 | int |  |
| record | bank-item-type | 5 | enum-like string ❓ | gemstone, moonstone, tokengold, tokeniron, tokensilver |

### guild-combat

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 1 | string |  |
| record | available-matching-weekly-time | 1 | string |  |
| record | party-battle-point-bonus-rate | 1 | int |  |
| record | required-level | 1 | int |  |
| record | required-mastery-level | 1 | int |  |
| record | season-type | 1 | enum-like string ❓ | guild-combat |

### guild-craft-recipe

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 105 | string |  |
| record | brand-material-1 | 23 | string |  |
| record | brand-material-2 | 18 | string |  |
| record | brand-material-3 | 6 | string |  |
| record | brand-material-condition-type-1 | 17 | enum-like string ❓ | earring, necklace, ring, weapon |
| record | brand-material-condition-type-2 | 14 | enum-like string ❓ | earring, gauntlet, necklace, ring, weapon |
| record | brand-material-condition-type-3 | 5 | enum-like string ❓ | staff, weapon |
| record | brand-material-count-1 | 24 | int |  |
| record | brand-material-count-2 | 19 | int |  |
| record | brand-material-count-3 | 7 | int |  |
| record | brand-material-donation-point-1 | 2 | int |  |
| record | brand-material-donation-point-2 | 1 | int |  |
| record | brand-material-donation-point-3 | 1 | int |  |
| record | contract-bm-item | 15 | string |  |
| record | contract-bm-item-count | 15 | int |  |
| record | contract-item-1 | 105 | string |  |
| record | contract-item-2 | 78 | string |  |
| record | contract-item-3 | 26 | string |  |
| record | contract-item-4 | 2 | string |  |
| record | contract-item-5 | 2 | string |  |
| record | contract-item-count-1 | 105 | int |  |
| record | contract-item-count-2 | 78 | int |  |
| record | contract-item-count-3 | 26 | int |  |
| record | contract-item-count-4 | 2 | int |  |
| record | contract-item-count-5 | 2 | int |  |
| record | contract-money | 105 | int |  |
| record | craft-type | 105 | enum-like string ❓ | big, small |
| record | id | 105 | int |  |
| record | material-1 | 103 | string |  |
| record | material-2 | 81 | string |  |
| record | material-3 | 61 | string |  |
| record | material-4 | 33 | string |  |
| record | material-5 | 14 | string |  |
| record | material-6 | 11 | string |  |
| record | material-7 | 3 | string |  |
| record | material-count-1 | 103 | int |  |
| record | material-count-2 | 81 | int |  |
| record | material-count-3 | 61 | int |  |
| record | material-count-4 | 33 | int |  |
| record | material-count-5 | 14 | int |  |
| record | material-count-6 | 11 | int |  |
| record | material-count-7 | 3 | int |  |
| record | material-donation-point-1 | 11 | int |  |
| record | material-donation-point-2 | 11 | int |  |
| record | production-type-1 | 88 | string |  |
| record | production-type-2 | 80 | string |  |
| record | production-type-3 | 67 | string |  |
| record | production-type-4 | 45 | string |  |
| record | production-type-5 | 38 | string |  |
| record | production-type-6 | 35 | string |  |
| record | production-type-7 | 13 | string |  |
| record | production-type-required-exp-1 | 88 | int |  |
| record | production-type-required-exp-2 | 81 | int |  |
| record | production-type-required-exp-3 | 67 | int |  |
| record | production-type-required-exp-4 | 45 | int |  |
| record | production-type-required-exp-5 | 38 | int |  |
| record | production-type-required-exp-6 | 34 | int |  |
| record | production-type-required-exp-7 | 14 | int |  |
| record | production-type-required-pc-count-1 | 88 | int |  |
| record | production-type-required-pc-count-2 | 81 | int |  |
| record | production-type-required-pc-count-3 | 67 | int |  |
| record | production-type-required-pc-count-4 | 45 | int |  |
| record | production-type-required-pc-count-5 | 38 | int |  |
| record | production-type-required-pc-count-6 | 35 | int |  |
| record | production-type-required-pc-count-7 | 13 | int |  |
| record | rent-bm-item | 67 | string |  |
| record | rent-bm-item-count | 67 | int |  |
| record | required-guild-level | 105 | int |  |
| record | result | 105 | string |  |
| record | result-bonus-count | 39 | int |  |
| record | result-bonus-guildlevel | 39 | int |  |
| record | result-count | 105 | int |  |
| record | result-receive-type | 105 | enum-like string ❓ | authorized, specified |
| record | time-cost | 105 | int |  |
| record | use-state | 105 | enum-like string ❓ | in-use, no-more-use |

### guild-discount

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | accessory-tranform | 2 | int |  |
| record | alias | 6 | string |  |
| record | item-awakening | 2 | int |  |
| record | item-growth | 5 | int |  |
| record | post-charge-discount-max | 3 | int |  |
| record | post-charge-discount-rate | 3 | int |  |
| record | weapon-tranform | 2 | int |  |

### guild-member-grade-authority

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | authority | 36 | string |  |
| record | editable-1 | 36 | string |  |
| record | editable-2 | 36 | string |  |
| record | editable-3 | 36 | string |  |
| record | editable-4 | 36 | string |  |
| record | editable-5 | 36 | string |  |
| record | guild-type | 36 | int |  |
| record | initial-value-1 | 36 | string |  |
| record | initial-value-2 | 36 | string |  |
| record | initial-value-3 | 36 | string |  |
| record | initial-value-4 | 36 | string |  |
| record | initial-value-5 | 36 | string |  |

### guildlevel

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | guild-discount | 16 | string |  |
| record | guild-type | 16 | int |  |
| record | guildcraft-big-rentable-count | 15 | int |  |
| record | guildcraft-big-usable-count | 15 | int |  |
| record | guildcraft-small-rentable-count | 15 | int |  |
| record | guildcraft-small-usable-count | 15 | int |  |
| record | level | 16 | int |  |
| record | max-member-count | 16 | int |  |
| record | min-enable-member-count | 16 | int |  |
| record | min-member-count | 16 | int |  |
| record | money | 16 | int |  |
| record | required-item-1 | 14 | string |  |
| record | required-item-2 | 13 | string |  |
| record | required-item-3 | 13 | string |  |
| record | required-item-4 | 13 | string |  |
| record | required-item-5 | 13 | string |  |
| record | required-item-count-1 | 14 | int |  |
| record | required-item-count-2 | 13 | int |  |
| record | required-item-count-3 | 13 | int |  |
| record | required-item-count-4 | 13 | int |  |
| record | required-item-count-5 | 13 | int |  |

### item

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | ability-feeding-required-item-1 | 1407 | string |  |
| record | ability-feeding-required-item-count-1 | 1407 | int |  |
| record | ability-feeding-required-money | 1407 | int |  |
| record | ability-feeding-target-1 | 22 | string |  |
| record | ability-feeding-target-2 | 11 | string |  |
| record | abnormal-attack-power-value-equip | 375 | int |  |
| record | accessory-type | 32466 | string |  |
| record | account-used | 120064 | enum-like string ❓ | n, y |
| record | acquire-used | 144012 | enum-like string ❓ | n, y |
| record | add-weapon-gem-slot-level-five-prob-weight | 2 | int |  |
| record | add-weapon-gem-slot-level-four-prob-weight | 2 | int |  |
| record | add-weapon-gem-slot-level-seven-prob-weight | 3 | int |  |
| record | add-weapon-gem-slot-level-six-prob-weight | 3 | int |  |
| record | add-weapon-gem-slot-level-three-prob-weight | 2 | int |  |
| record | additional-awakening-probability | 2 | int |  |
| record | additional-exp-percent-by-kill | 1 | int |  |
| record | additional-mastery-stat-point | 4 | int |  |
| record | alias | 144241 | string |  |
| record | attach-equip-gem-piece-1 | 604 | string |  |
| record | attach-equip-gem-piece-10 | 600 | string |  |
| record | attach-equip-gem-piece-2 | 603 | string |  |
| record | attach-equip-gem-piece-3 | 603 | string |  |
| record | attach-equip-gem-piece-4 | 603 | string |  |
| record | attach-equip-gem-piece-5 | 601 | string |  |
| record | attach-equip-gem-piece-6 | 601 | string |  |
| record | attach-equip-gem-piece-7 | 601 | string |  |
| record | attach-equip-gem-piece-8 | 601 | string |  |
| record | attach-equip-gem-piece-9 | 601 | string |  |
| record | attach-equip-gem-piece-consumable-item-1 | 20166 | string |  |
| record | attach-equip-gem-piece-consumable-item-2 | 10877 | string |  |
| record | attach-equip-gem-piece-consumable-item-count-1 | 20166 | int |  |
| record | attach-equip-gem-piece-consumable-item-count-2 | 10877 | int |  |
| record | attach-equip-gem-piece-money-cost | 17893 | int |  |
| record | attach-equip-gem-piece-primary-gem-equip-level-max | 20167 | int |  |
| record | attach-equip-gem-piece-primary-gem-equip-level-min | 20167 | int |  |
| record | attach-equip-gem-piece-prop-weight-1 | 604 | int |  |
| record | attach-equip-gem-piece-prop-weight-10 | 600 | int |  |
| record | attach-equip-gem-piece-prop-weight-2 | 603 | int |  |
| record | attach-equip-gem-piece-prop-weight-3 | 603 | int |  |
| record | attach-equip-gem-piece-prop-weight-4 | 603 | int |  |
| record | attach-equip-gem-piece-prop-weight-5 | 601 | int |  |
| record | attach-equip-gem-piece-prop-weight-6 | 601 | int |  |
| record | attach-equip-gem-piece-prop-weight-7 | 601 | int |  |
| record | attach-equip-gem-piece-prop-weight-8 | 601 | int |  |
| record | attach-equip-gem-piece-prop-weight-9 | 601 | int |  |
| record | attack-attribute-value-equip | 7252 | int |  |
| record | attack-critical-damage-percent | 28 | int |  |
| record | attack-critical-damage-value-equip | 17593 | int |  |
| record | attack-critical-value-equip | 61696 | int |  |
| record | attack-damage-modify-diff | 14138 | int |  |
| record | attack-effect-1 | 477 | string |  |
| record | attack-effect-2 | 81 | string |  |
| record | attack-effect-probability | 464 | int |  |
| record | attack-effect-skill-result | 481 | enum-like string ❓ | critical-hit, hit-critical-hit, parry-dodge |
| record | attack-effect-target-effect-flag-1 | 178 | enum-like string ❓ | down, stun |
| record | attack-effect-target-effect-flag-2 | 178 | enum-like string ❓ | down, stun |
| record | attack-effect-target-effect-flag-3 | 45 | enum-like string ❓ | kneel |
| record | attack-hit-value-equip | 19844 | int |  |
| record | attack-pierce-value-equip | 34550 | int |  |
| record | attack-power-equip-max | 71583 | int |  |
| record | attack-power-equip-min | 71583 | int |  |
| record | attack-stiff-duration-base-percent | 376 | int |  |
| record | auctionable | 144217 | enum-like string ❓ | n, y |
| record | awakening-probability | 2 | int |  |
| record | awakening-recipe-cost | 3325 | int |  |
| record | awakening-recipe-material-1 | 3325 | string |  |
| record | awakening-recipe-material-2 | 3080 | string |  |
| record | awakening-recipe-material-3 | 2250 | string |  |
| record | awakening-recipe-material-4 | 1433 | string |  |
| record | awakening-recipe-material-5 | 1328 | string |  |
| record | awakening-recipe-material-6 | 1258 | string |  |
| record | awakening-recipe-material-count-1 | 3325 | int |  |
| record | awakening-recipe-material-count-2 | 3080 | int |  |
| record | awakening-recipe-material-count-3 | 2246 | int |  |
| record | awakening-recipe-material-count-4 | 1425 | int |  |
| record | awakening-recipe-material-count-5 | 1319 | int |  |
| record | awakening-recipe-material-count-6 | 936 | int |  |
| record | badge-gear-score | 747 | int |  |
| record | badge-synthesis-score | 747 | int |  |
| record | base-fee | 144082 | int |  |
| record | bonus-account-exp | 6 | int |  |
| record | bonus-exp | 3 | int |  |
| record | bonus-mastery-exp | 16 | int |  |
| record | brand | 89144 | string |  |
| record | cannot-depot | 143994 | enum-like string ❓ | n, y |
| record | cannot-dispose | 144008 | enum-like string ❓ | n, y |
| record | cannot-equip-identical-pairs | 13 | enum-like string ❓ | earring, necklace, ring, soul |
| record | cannot-sell | 144003 | enum-like string ❓ | n, y |
| record | cannot-trade | 144009 | enum-like string ❓ | n, y |
| record | cannot-transform | 460 | enum-like string ❓ | y |
| record | closet-group-id | 3227 | int |  |
| record | consume-durability | 143978 | enum-like string ❓ | n, y |
| record | custom-dress-design-state | 352 | enum-like string ❓ | activated, disabled |
| record | decompose-by-item2-1 | 1481 | string |  |
| record | decompose-by-item2-2 | 1329 | string |  |
| record | decompose-by-item2-3 | 474 | string |  |
| record | decompose-by-item2-4 | 30 | string |  |
| record | decompose-by-item2-5 | 9 | string |  |
| record | decompose-by-item2-6 | 9 | string |  |
| record | decompose-by-item2-stack-count-1 | 1481 | int |  |
| record | decompose-by-item2-stack-count-2 | 1329 | int |  |
| record | decompose-by-item2-stack-count-3 | 474 | int |  |
| record | decompose-by-item2-stack-count-4 | 30 | int |  |
| record | decompose-by-item2-stack-count-5 | 9 | int |  |
| record | decompose-by-item2-stack-count-6 | 9 | int |  |
| record | decompose-job-reward-archer | 1107 | string |  |
| record | decompose-job-reward-assassin | 1337 | string |  |
| record | decompose-job-reward-blade-master | 1337 | string |  |
| record | decompose-job-reward-destroyer | 1337 | string |  |
| record | decompose-job-reward-force-master | 1337 | string |  |
| record | decompose-job-reward-kung-fu-fighter | 1337 | string |  |
| record | decompose-job-reward-shooter | 1107 | string |  |
| record | decompose-job-reward-soul-fighter | 1121 | string |  |
| record | decompose-job-reward-summoner | 1261 | string |  |
| record | decompose-job-reward-sword-master | 1261 | string |  |
| record | decompose-job-reward-thunderer | 1 | string |  |
| record | decompose-job-reward-warlock | 1166 | string |  |
| record | decompose-job-reward-warrior | 1107 | string |  |
| record | decompose-max | 92796 | int |  |
| record | decompose-money-cost | 38 | int |  |
| record | decompose-reward-1 | 92487 | string |  |
| record | decompose-reward-2 | 200 | string |  |
| record | decompose-reward-3 | 86 | string |  |
| record | decompose-reward-4 | 11 | string |  |
| record | decompose-reward-5 | 9 | string |  |
| record | decompose-reward-6 | 9 | string |  |
| record | decompose-reward-by-consume-index | 212 | enum-like string ❓ | y |
| record | default-gem-level | 52218 | int |  |
| record | defend-critical-value-equip | 5180 | int |  |
| record | defend-damage-modify-diff | 2948 | int |  |
| record | defend-dodge-value-equip | 28499 | int |  |
| record | defend-parry-value-equip | 17160 | int |  |
| record | defend-power-equip-value | 13667 | int |  |
| record | defend-stiff-duration-base-percent | 42 | int |  |
| record | detach-gem-money-cost | 1300 | int |  |
| record | dispose-awakening-feed-item | 20 | string |  |
| record | dispose-awakening-material | 20 | string |  |
| record | duel-skill3 | 72 | string |  |
| record | effect-equip-1 | 62732 | string |  |
| record | effect-equip-2 | 16341 | string |  |
| record | effect-equip-3 | 88 | string |  |
| record | enchant-gem-slot-usable-1 | 2272 | enum-like string ❓ | y |
| record | enchant-gem-slot-usable-2 | 2 | enum-like string ❓ | y |
| record | equip-faction | 14 | string |  |
| record | equip-faction-level | 14 | int |  |
| record | equip-job-check-1 | 90574 | string |  |
| record | equip-job-check-2 | 1807 | string |  |
| record | equip-job-check-3 | 391 | string |  |
| record | equip-job-check-4 | 296 | string |  |
| record | equip-level | 144011 | int |  |
| record | equip-level-max | 1 | int |  |
| record | equip-mastery-level | 2 | int |  |
| record | equip-race | 323 | enum-like string ❓ | gon, jin, kun, lyn, summoned-cat |
| record | equip-sex | 251 | enum-like string ❓ | female, male |
| record | equip-type | 144178 | string |  |
| record | equip-used | 143972 | enum-like string ❓ | n, y |
| record | event-info | 1594 | string |  |
| record | exceptional-usable-attraction | 92 | string |  |
| record | expand-weapon-gem-slot-level-1 | 12 | int |  |
| record | expand-weapon-gem-slot-level-2 | 5 | int |  |
| record | faction | 21 | string |  |
| record | feed-ability-random-1 | 23 | string |  |
| record | feed-ability-random-2 | 10 | string |  |
| record | feed-ability-random-3 | 10 | string |  |
| record | feed-ability-random-4 | 4 | string |  |
| record | feed-ability-random-count | 23 | int |  |
| record | fixed-open-gem-slot | 52122 | int |  |
| record | gear-score | 108264 | int |  |
| record | gem-slot-by-probability | 52987 | enum-like string ❓ | n, y |
| record | gem-slot-expandable | 53687 | enum-like string ❓ | n, y |
| record | gem-slot-type-1 | 22996 | enum-like string ❓ | plus-one, plus-zero |
| record | gem-slot-type-2 | 20301 | enum-like string ❓ | plus-one, plus-zero |
| record | gem-slot-type-3 | 15381 | enum-like string ❓ | plus-one, plus-zero |
| record | gem-slot-type-4 | 1985 | enum-like string ❓ | plus-zero |
| record | gem-type | 21820 | enum-like string ❓ | equip-gem, feed-gem, pet-gem, random-stat-equip-gem, weapon-enchant-gem, weapon-gem |
| record | grocery-type | 34565 | enum-like string ❓ | badge, base-camp-warp, cave-escape, fishing-paste, key, messenger, other, pet-food, random-box, repair, scroll, sealed, skill-book, weapon-gem-slot-adder, weapon-gem-slot-expander |
| record | growth-efficient-level | 76403 | int |  |
| record | growth-exp-scale | 16530 | int |  |
| record | growth-id | 47970 | string |  |
| record | growth-level-max | 47957 | int |  |
| record | growth-recipe-cost | 55126 | int |  |
| record | growth-recipe-exp-1 | 155 | int |  |
| record | growth-recipe-exp-2 | 155 | int |  |
| record | growth-recipe-exp-3 | 155 | int |  |
| record | growth-recipe-exp-4 | 155 | int |  |
| record | growth-recipe-exp-max | 55029 | int |  |
| record | growth-recipe-exp-min | 55024 | int |  |
| record | growth-recipe-exp-prob-weight-1 | 155 | int |  |
| record | growth-recipe-exp-prob-weight-2 | 109 | int |  |
| record | growth-recipe-exp-prob-weight-3 | 109 | int |  |
| record | growth-recipe-exp-prob-weight-4 | 109 | int |  |
| record | growth-recipe-material-1 | 29654 | string |  |
| record | growth-recipe-material-count-1 | 29654 | int |  |
| record | growth-step | 47957 | int |  |
| record | guild-used | 4 | enum-like string ❓ | y |
| record | hate-power-equip-value | 435 | int |  |
| record | heal-power-value | 1021 | int |  |
| record | hidden-power-attach | 5643 | int |  |
| record | hp-regen-combat-equip | 1473 | int |  |
| record | hp-regen-equip | 714 | int |  |
| record | id | 144241 | int |  |
| record | identify-enable | 6631 | enum-like string ❓ | n, y |
| record | identify-grocery | 5070 | string |  |
| record | init-durability | 55098 | int |  |
| record | item-grade | 143576 | int |  |
| record | item-skill-1 | 15030 | string |  |
| record | item-skill-10 | 15002 | string |  |
| record | item-skill-11 | 14982 | string |  |
| record | item-skill-12 | 14982 | string |  |
| record | item-skill-13 | 14982 | string |  |
| record | item-skill-14 | 14982 | string |  |
| record | item-skill-15 | 14982 | string |  |
| record | item-skill-16 | 14982 | string |  |
| record | item-skill-2 | 15026 | string |  |
| record | item-skill-3 | 15026 | string |  |
| record | item-skill-4 | 15026 | string |  |
| record | item-skill-5 | 15006 | string |  |
| record | item-skill-6 | 15006 | string |  |
| record | item-skill-7 | 15002 | string |  |
| record | item-skill-8 | 15002 | string |  |
| record | item-skill-9 | 15002 | string |  |
| record | item-skill-second-1 | 1872 | string |  |
| record | item-skill-second-10 | 1872 | string |  |
| record | item-skill-second-11 | 1872 | string |  |
| record | item-skill-second-12 | 1872 | string |  |
| record | item-skill-second-13 | 1872 | string |  |
| record | item-skill-second-14 | 1872 | string |  |
| record | item-skill-second-15 | 1872 | string |  |
| record | item-skill-second-16 | 1872 | string |  |
| record | item-skill-second-2 | 1872 | string |  |
| record | item-skill-second-3 | 1872 | string |  |
| record | item-skill-second-4 | 1872 | string |  |
| record | item-skill-second-5 | 1872 | string |  |
| record | item-skill-second-6 | 1872 | string |  |
| record | item-skill-second-7 | 1872 | string |  |
| record | item-skill-second-8 | 1872 | string |  |
| record | item-skill-second-9 | 1872 | string |  |
| record | item-usable-group | 34474 | string |  |
| record | job-ability-1 | 68609 | int |  |
| record | job-decompose-by-item2-1 | 268 | string |  |
| record | job-decompose-by-item2-2 | 87 | string |  |
| record | job-decompose-by-item2-stack-count-1 | 268 | int |  |
| record | job-decompose-by-item2-stack-count-2 | 87 | int |  |
| record | level | 144241 | int |  |
| record | main-ability-fixed | 1384 | string |  |
| record | market-category-1 | 142109 | string |  |
| record | market-category-2 | 142109 | string |  |
| record | market-category-3 | 142109 | string |  |
| record | max-durability | 55098 | int |  |
| record | max-gem-slot-count | 51914 | int |  |
| record | max-gem-slot-level | 52430 | int |  |
| record | max-guard-gauge-equip | 1111 | int |  |
| record | max-hp-equip | 32961 | int |  |
| record | message-range | 6 | enum-like string ❓ | all, district |
| record | necessary-campfire | 49 | enum-like string ❓ | n, y |
| record | necessary-costume | 64 | string |  |
| record | necessary-costume-brand | 42 | string |  |
| record | one-gem-slot-prob-weight | 3717 | int |  |
| record | open-gem-slot-grocery-1 | 50604 | string |  |
| record | open-gem-slot-grocery-2 | 50604 | string |  |
| record | open-gem-slot-grocery-3 | 50604 | string |  |
| record | open-gem-slot-grocery-4 | 50604 | string |  |
| record | open-gem-slot-grocery-5 | 12982 | string |  |
| record | open-gem-slot-grocery-6 | 12982 | string |  |
| record | open-gem-slot-grocery-7 | 12957 | string |  |
| record | open-gem-slot-grocery-8 | 12957 | string |  |
| record | open-gem-slot-grocery-count-1 | 50520 | int |  |
| record | open-gem-slot-grocery-count-2 | 50520 | int |  |
| record | open-gem-slot-grocery-count-3 | 50520 | int |  |
| record | open-gem-slot-grocery-count-4 | 50520 | int |  |
| record | open-gem-slot-grocery-count-5 | 12982 | int |  |
| record | open-gem-slot-grocery-count-6 | 12982 | int |  |
| record | open-gem-slot-grocery-count-7 | 12957 | int |  |
| record | open-gem-slot-grocery-count-8 | 12957 | int |  |
| record | open-gem-slot-percent-1 | 50606 | int |  |
| record | open-gem-slot-percent-2 | 50606 | int |  |
| record | open-gem-slot-percent-3 | 50606 | int |  |
| record | open-gem-slot-percent-4 | 50606 | int |  |
| record | open-gem-slot-percent-5 | 12982 | int |  |
| record | open-gem-slot-percent-6 | 12982 | int |  |
| record | open-gem-slot-percent-7 | 12957 | int |  |
| record | open-gem-slot-percent-8 | 12957 | int |  |
| record | open-gem-slot-price-1 | 50520 | int |  |
| record | open-gem-slot-price-2 | 50520 | int |  |
| record | open-gem-slot-price-3 | 50520 | int |  |
| record | open-gem-slot-price-4 | 50520 | int |  |
| record | open-gem-slot-price-5 | 12982 | int |  |
| record | open-gem-slot-price-6 | 12982 | int |  |
| record | open-gem-slot-price-7 | 12957 | int |  |
| record | open-gem-slot-price-8 | 12957 | int |  |
| record | pccafe-ncsoft-korea-only | 209 | enum-like string ❓ | y |
| record | pet-food-recovery | 2 | string |  |
| record | potential-attack-critical-damage-percent | 19625 | int |  |
| record | potential-attack-critical-value | 19631 | int |  |
| record | potential-attack-hit-value | 19625 | int |  |
| record | potential-attack-pierce-value | 19625 | int |  |
| record | potential-defend-critical-value | 19625 | int |  |
| record | potential-defend-dodge-value | 19625 | int |  |
| record | potential-defend-parry-value | 19625 | int |  |
| record | potential-defend-power-equip-value | 19625 | int |  |
| record | potential-max-hp | 19541 | int |  |
| record | pouch-appearance | 142402 | string |  |
| record | pouch-appearance-priority | 142402 | int |  |
| record | price | 144084 | int |  |
| record | probability-block-gem-slot-2 | 11176 | int |  |
| record | probability-block-gem-slot-3 | 19369 | int |  |
| record | probability-plus-one-gem-slot-1 | 28836 | int |  |
| record | probability-plus-one-gem-slot-2 | 28906 | int |  |
| record | probability-plus-one-gem-slot-3 | 19394 | int |  |
| record | probability-plus-one-gem-slot-4 | 18 | int |  |
| record | probability-plus-zero-gem-slot-1 | 10765 | int |  |
| record | probability-plus-zero-gem-slot-2 | 29675 | int |  |
| record | probability-plus-zero-gem-slot-3 | 19570 | int |  |
| record | probability-plus-zero-gem-slot-4 | 40 | int |  |
| record | pve-boss-level-npc-attack-power-equip-max | 3543 | int |  |
| record | pve-boss-level-npc-attack-power-equip-min | 3542 | int |  |
| record | pve-boss-level-npc-defend-power-equip-value | 76 | int |  |
| record | pvp-attack-power-equip-max | 1051 | int |  |
| record | pvp-attack-power-equip-min | 1051 | int |  |
| record | pvp-defend-power-equip-value | 6 | int |  |
| record | recharge-heart-type | 12 | enum-like string ❓ | instant-heart |
| record | repair-duration | 42 | int |  |
| record | repair-item-count | 52969 | int |  |
| record | repair-level | 54673 | int |  |
| record | repair-level-max | 42 | int |  |
| record | repair-level-min | 42 | int |  |
| record | repair-money-cost | 52984 | int |  |
| record | repairable | 1047 | enum-like string ❓ | n |
| record | required-awakening-bm-item-count | 2 | int |  |
| record | required-completion-quest | 21 | string |  |
| record | required-feed-item-brand-for-awakening | 3325 | string |  |
| record | required-feed-item-brand-for-growth | 10 | string |  |
| record | required-feed-item-condition-type | 3325 | string |  |
| record | required-feed-item-for-growth | 4337 | string |  |
| record | required-feed-item-level-min | 3308 | int |  |
| record | required-seed-item-condition-type | 55199 | string |  |
| record | required-seed-item-equip-level-max | 55192 | int |  |
| record | required-seed-item-equip-level-min | 55192 | int |  |
| record | required-seed-item-growth-course | 55199 | enum-like string ❓ | coin, normal |
| record | resurrect-effect | 55 | string |  |
| record | seal-acquire-item | 27211 | string |  |
| record | seal-consume-item-1 | 27211 | string |  |
| record | seal-consume-item-count-1 | 27211 | int |  |
| record | second-attack-effect-1 | 34 | string |  |
| record | second-attack-effect-2 | 6 | string |  |
| record | second-attack-effect-probability | 31 | int |  |
| record | second-attack-effect-skill-result | 34 | enum-like string ❓ | critical-hit, hit-critical-hit, parry-dodge |
| record | second-attack-effect-target-effect-flag-1 | 9 | enum-like string ❓ | down, stun |
| record | second-attack-effect-target-effect-flag-2 | 9 | enum-like string ❓ | down, stun |
| record | set-item | 25130 | string |  |
| record | skill-build-up-level-1 | 300 | int |  |
| record | skill-build-up-level-2 | 264 | int |  |
| record | skill-build-up-level-3 | 2 | int |  |
| record | skill-build-up-parent-skill3-id-1 | 300 | int |  |
| record | skill-build-up-parent-skill3-id-2 | 264 | int |  |
| record | skill-build-up-parent-skill3-id-3 | 2 | int |  |
| record | skill-build-up-point | 3 | int |  |
| record | skill-by-equipment | 1 | string |  |
| record | skill-limit-level | 554 | int |  |
| record | skill-limit-level-max | 63 | int |  |
| record | skill-limit-mastery-level | 38 | int |  |
| record | skill-limit-mastery-level-max | 39 | int |  |
| record | skill-modify-info-group-1 | 10890 | string |  |
| record | skill-modify-info-group-2 | 10890 | string |  |
| record | skill-modify-info-group-3 | 8973 | string |  |
| record | skill-modify-info-group-6 | 10890 | string |  |
| record | skill-modify-info-group-7 | 10890 | string |  |
| record | skill-modify-info-group-8 | 8973 | string |  |
| record | skill-skin | 13 | string |  |
| record | skill-skin-acquire | 9 | string |  |
| record | skill3 | 712 | string |  |
| record | slate-scroll | 18 | string |  |
| record | stack-count | 34565 | int |  |
| record | sub-ability-fixed | 288 | string |  |
| record | sub-ability-random-1 | 1384 | string |  |
| record | sub-ability-random-2 | 1384 | string |  |
| record | sub-ability-random-3 | 1384 | string |  |
| record | sub-ability-random-4 | 1096 | string |  |
| record | sub-ability-random-count | 1384 | int |  |
| record | time-limit-type | 692 | enum-like string ❓ | duration |
| record | type | 144241 | enum-like string ❓ | accessory, costume, gem, grocery, weapon |
| record | ui-icon-state | 31335 | string |  |
| record | unseal-acquire-item-1 | 18103 | string |  |
| record | unseal-acquire-item-10 | 429 | string |  |
| record | unseal-acquire-item-11 | 429 | string |  |
| record | unseal-acquire-item-12 | 429 | string |  |
| record | unseal-acquire-item-13 | 420 | string |  |
| record | unseal-acquire-item-14 | 420 | string |  |
| record | unseal-acquire-item-15 | 420 | string |  |
| record | unseal-acquire-item-16 | 420 | string |  |
| record | unseal-acquire-item-17 | 420 | string |  |
| record | unseal-acquire-item-18 | 420 | string |  |
| record | unseal-acquire-item-19 | 60 | string |  |
| record | unseal-acquire-item-2 | 1411 | string |  |
| record | unseal-acquire-item-20 | 60 | string |  |
| record | unseal-acquire-item-3 | 1393 | string |  |
| record | unseal-acquire-item-4 | 1393 | string |  |
| record | unseal-acquire-item-5 | 1387 | string |  |
| record | unseal-acquire-item-6 | 485 | string |  |
| record | unseal-acquire-item-7 | 485 | string |  |
| record | unseal-acquire-item-8 | 485 | string |  |
| record | unseal-acquire-item-9 | 485 | string |  |
| record | unseal-acquire-item-prob-weight-1 | 18103 | int |  |
| record | unseal-acquire-item-prob-weight-10 | 429 | int |  |
| record | unseal-acquire-item-prob-weight-11 | 429 | int |  |
| record | unseal-acquire-item-prob-weight-12 | 453 | int |  |
| record | unseal-acquire-item-prob-weight-13 | 444 | int |  |
| record | unseal-acquire-item-prob-weight-14 | 444 | int |  |
| record | unseal-acquire-item-prob-weight-15 | 444 | int |  |
| record | unseal-acquire-item-prob-weight-16 | 436 | int |  |
| record | unseal-acquire-item-prob-weight-17 | 436 | int |  |
| record | unseal-acquire-item-prob-weight-18 | 436 | int |  |
| record | unseal-acquire-item-prob-weight-19 | 60 | int |  |
| record | unseal-acquire-item-prob-weight-2 | 1416 | int |  |
| record | unseal-acquire-item-prob-weight-20 | 60 | int |  |
| record | unseal-acquire-item-prob-weight-3 | 1398 | int |  |
| record | unseal-acquire-item-prob-weight-4 | 1398 | int |  |
| record | unseal-acquire-item-prob-weight-5 | 1392 | int |  |
| record | unseal-acquire-item-prob-weight-6 | 485 | int |  |
| record | unseal-acquire-item-prob-weight-7 | 485 | int |  |
| record | unseal-acquire-item-prob-weight-8 | 485 | int |  |
| record | unseal-acquire-item-prob-weight-9 | 485 | int |  |
| record | unseal-consume-item2-1 | 18103 | string |  |
| record | unseal-consume-item2-2 | 18029 | string |  |
| record | unseal-consume-item2-stack-count-1 | 18103 | int |  |
| record | unseal-consume-item2-stack-count-2 | 18029 | int |  |
| record | usable-duration | 692 | int |  |
| record | usable-in-cave-and-dungeon-only | 12 | enum-like string ❓ | y |
| record | usable-in-duel | 472 | enum-like string ❓ | n |
| record | use-global-recycle-group | 121 | int |  |
| record | use-global-recycle-group-duration | 121 | int |  |
| record | use-recycle-group | 121 | string |  |
| record | use-recycle-group-duration | 121 | int |  |
| record | use-recycle-group-id | 121 | int |  |
| record | valid-attraction-1 | 157 | string |  |
| record | valid-attraction-2 | 84 | string |  |
| record | valid-attraction-3 | 43 | string |  |
| record | valid-attraction-4 | 10 | string |  |
| record | weapon-appearance-change-type | 53067 | enum-like string ❓ | both, used-only-as-applying-weapon, used-only-as-target-weapon |
| record | weapon-enchant-gem-slot-type | 341 | enum-like string ❓ | first, second |
| record | weapon-gem-ability-1 | 1280 | int |  |
| record | weapon-gem-ability-2 | 336 | int |  |
| record | weapon-gem-level | 1309 | int |  |
| record | weapon-gem-type | 1311 | string |  |
| record | weapon-type | 55762 | string |  |
| record | zero-gem-slot-prob-weight | 3717 | int |  |

### item-brand

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 3188 | string |  |
| record | id | 3188 | int |  |

### item-brand-tooltip

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |

### item-buy-price

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 3550 | string |  |
| record | check-content-quota | 19 | string |  |
| record | check-solo-duel-grade | 14 | int |  |
| record | check-team-duel-grade | 14 | int |  |
| record | faction-level | 146 | int |  |
| record | money | 987 | int |  |
| record | required-achievement-id | 212 | int |  |
| record | required-achievement-score | 1 | int |  |
| record | required-achievement-step-min | 212 | int |  |
| record | required-duel-point | 71 | int |  |
| record | required-faction-score | 38 | int |  |
| record | required-field-play-point | 168 | int |  |
| record | required-item-1 | 2810 | string |  |
| record | required-item-2 | 607 | string |  |
| record | required-item-3 | 166 | string |  |
| record | required-item-4 | 24 | string |  |
| record | required-item-count-1 | 2810 | int |  |
| record | required-item-count-2 | 607 | int |  |
| record | required-item-count-3 | 166 | int |  |
| record | required-item-count-4 | 24 | int |  |
| record | required-itembrand | 129 | string |  |
| record | required-itembrand-condition-type | 129 | enum-like string ❓ | all, assassin-dagger, aura-bangle, axe, blade-master-sword, gauntlet, kung-fu-fighter-gauntlet, lyn-sword, soul, staff, warlock-dagger |
| record | required-life-contents-point | 42 | int |  |
| record | required-party-battle-point | 56 | int |  |

### item-event

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 221 | string |  |
| record | event-expiration-time | 221 | string |  |

### item-random-ability-section

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 1877 | string |  |
| record | distribution-type | 1877 | string |  |
| record | variation-value-max | 1877 | int |  |
| record | variation-value-min | 1877 | int |  |
| record | variation-value-with-special-item-max | 1877 | int |  |
| record | variation-value-with-special-item-min | 1877 | int |  |

### item-random-ability-slot

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | ability | 5649 | enum-like string ❓ | attack-critical-value, attack-hit-value, attack-pierce-value, defend-critical-value, defend-dodge-value, defend-parry-value, defend-power-equip-value, max-hp |
| record | alias | 5649 | string |  |
| record | id | 5649 | int |  |
| record | initial-value-max | 5440 | int |  |
| record | item-ability-section-1 | 5649 | string |  |
| record | item-ability-section-percent-1 | 5649 | int |  |
| record | value-max | 5440 | int |  |
| record | value-min | 5440 | int |  |

### item-transform-retry-cost

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 1 | string |  |
| record | mileage-range-max-1 | 1 | int |  |
| record | mileage-range-max-2 | 1 | int |  |
| record | mileage-range-max-3 | 1 | int |  |
| record | mileage-range-max-4 | 1 | int |  |
| record | mileage-range-max-5 | 1 | int |  |
| record | required-item-for-instant-payment-1 | 1 | string |  |
| record | required-item-for-instant-payment-2 | 1 | string |  |
| record | required-item-for-instant-payment-3 | 1 | string |  |
| record | required-item-for-instant-payment-4 | 1 | string |  |
| record | required-item-for-instant-payment-5 | 1 | string |  |

### item-usable-group

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 15 | string |  |
| record | default_item_usable | 15 | enum-like string ❓ | n, y |

### jackpot-boss-zone

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 1 | string |  |
| record | boss-max-faction-score | 1 | int |  |
| record | boss-npc-faction-1 | 1 | string |  |
| record | boss-npc-faction-2 | 1 | string |  |
| record | despawn-force-faction-1 | 1 | enum-like string ❓ | y |
| record | despawn-force-faction-2 | 1 | enum-like string ❓ | y |
| record | despawn-npc-group-faction-1 | 1 | string |  |
| record | despawn-npc-group-faction-2 | 1 | string |  |
| record | group | 1 | string |  |
| record | id | 1 | int |  |
| record | respawn-after-despawn-faction-1 | 1 | enum-like string ❓ | n |
| record | respawn-after-despawn-faction-2 | 1 | enum-like string ❓ | n |
| record | spawn-boss-faction-score | 1 | int |  |
| record | spawn-delay-faction-1 | 1 | int |  |
| record | spawn-delay-faction-2 | 1 | int |  |
| record | spawn-npc-group-faction-1 | 1 | string |  |
| record | spawn-npc-group-faction-2 | 1 | string |  |
| record | transfer-jackpot-percent | 1 | int |  |
| record | zone | 1 | int |  |

### job

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | guard-gauge-effect-1 | 1 | string |  |
| record | job | 14 | string |  |

### job-quest

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |

### job-style

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | default-stance | 64 | string |  |
| record | job | 64 | string |  |
| record | job-style | 64 | string |  |
| record | max-sp2 | 64 | int |  |
| record | sp2-regen | 64 | int |  |

### level

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | default-faction-score-acquire-level | 60 | int |  |
| record | duel-death-match-1vs1-lose-draw-duel-point | 60 | int |  |
| record | duel-death-match-1vs1-lose-draw-exp | 60 | int |  |
| record | duel-death-match-1vs1-win-duel-point | 60 | int |  |
| record | duel-death-match-1vs1-win-exp | 60 | int |  |
| record | duel-tag-match-3vs3-lose-draw-duel-point | 60 | int |  |
| record | duel-tag-match-3vs3-lose-draw-exp | 60 | int |  |
| record | duel-tag-match-3vs3-win-duel-point | 60 | int |  |
| record | duel-tag-match-3vs3-win-exp | 60 | int |  |
| record | exp | 60 | int |  |
| record | exp-boost-1 | 60 | int |  |
| record | exp-boost-2 | 60 | int |  |
| record | exp-boost-3 | 60 | int |  |
| record | exp-boost-recharge-time-1 | 60 | int |  |
| record | exp-boost-recharge-time-2 | 60 | int |  |
| record | exp-boost-recharge-time-3 | 60 | int |  |
| record | express-cost | 60 | int |  |
| record | half-faction-score-acquire-level | 60 | int |  |
| record | level | 60 | int |  |
| record | party-battle-capture-the-flag-lose-draw-exp | 60 | int |  |
| record | party-battle-capture-the-flag-lose-draw-party-battle-point | 60 | int |  |
| record | party-battle-capture-the-flag-win-exp | 60 | int |  |
| record | party-battle-capture-the-flag-win-party-battle-point | 60 | int |  |
| record | party-battle-lead-the-ball-lose-draw-exp | 60 | int |  |
| record | party-battle-lead-the-ball-lose-draw-party-battle-point | 60 | int |  |
| record | party-battle-lead-the-ball-win-exp | 60 | int |  |
| record | party-battle-lead-the-ball-win-party-battle-point | 60 | int |  |
| record | party-battle-occupation-war-lose-draw-exp | 60 | int |  |
| record | party-battle-occupation-war-lose-draw-party-battle-point | 60 | int |  |
| record | party-battle-occupation-war-win-exp | 60 | int |  |
| record | party-battle-occupation-war-win-party-battle-point | 60 | int |  |

### market-register-amount-tax-rate

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | max-amount | 10 | int |  |
| record | min-amount | 10 | int |  |
| record | tax-rate | 10 | int |  |

### mastery-ability

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | aoe-defend-base-percent | 1200 | int |  |
| record | aoe-defend-power-modify | 1200 | int |  |
| record | aoe-defend-power-value | 1200 | int |  |
| record | attack-concentrate-value | 1200 | int |  |
| record | attack-counter-damage-modify | 1200 | int |  |
| record | attack-critical-base-percent | 1200 | int |  |
| record | attack-critical-damage-modify | 1200 | int |  |
| record | attack-critical-damage-percent | 1200 | int |  |
| record | attack-critical-value | 1200 | int |  |
| record | attack-critical-value-modify | 1200 | int |  |
| record | attack-damage-modify-diff | 1200 | int |  |
| record | attack-damage-modify-percent | 1200 | int |  |
| record | attack-hit-base-percent | 1200 | int |  |
| record | attack-hit-value | 1200 | int |  |
| record | attack-hit-value-modify | 1200 | int |  |
| record | attack-parry-pierce-modify | 1200 | int |  |
| record | attack-perfect-parry-damage-modify | 1200 | int |  |
| record | attack-pierce-modify | 1200 | int |  |
| record | attack-pierce-value | 1200 | int |  |
| record | attack-power-creature-max | 1200 | int |  |
| record | attack-power-creature-min | 1200 | int |  |
| record | attack-power-equip-max | 1200 | int |  |
| record | attack-power-equip-min | 1200 | int |  |
| record | attack-stiff-duration-base-percent | 1200 | int |  |
| record | attack-stiff-duration-value | 1200 | int |  |
| record | attack-stiff-duration-value-modify | 1200 | int |  |
| record | cast-duration-base-percent | 1200 | int |  |
| record | cast-duration-value | 1200 | int |  |
| record | cast-duration-value-modify | 1200 | int |  |
| record | defend-bounce-percent | 1200 | int |  |
| record | defend-counter-reduce-modify | 1200 | int |  |
| record | defend-counter-reduce-percent | 1200 | int |  |
| record | defend-critical-base-percent | 1200 | int |  |
| record | defend-critical-damage-modify | 1200 | int |  |
| record | defend-critical-value | 1200 | int |  |
| record | defend-critical-value-modify | 1200 | int |  |
| record | defend-damage-modify-diff | 1200 | int |  |
| record | defend-damage-modify-percent | 1200 | int |  |
| record | defend-dodge-base-percent | 1200 | int |  |
| record | defend-dodge-value | 1200 | int |  |
| record | defend-dodge-value-modify | 1200 | int |  |
| record | defend-force-damage-reduce-percent | 1200 | int |  |
| record | defend-force-value-modify | 1200 | int |  |
| record | defend-immune-base-percent | 1200 | int |  |
| record | defend-miss-base-percent | 1200 | int |  |
| record | defend-parry-base-percent | 1176 | int |  |
| record | defend-parry-reduce-diff | 1200 | int |  |
| record | defend-parry-reduce-modify | 1200 | int |  |
| record | defend-parry-reduce-percent | 1200 | int |  |
| record | defend-parry-value | 1200 | int |  |
| record | defend-parry-value-modify | 1200 | int |  |
| record | defend-perfect-parry-base-percent | 1200 | int |  |
| record | defend-perfect-parry-reduce-modify | 1200 | int |  |
| record | defend-perfect-parry-reduce-percent | 1200 | int |  |
| record | defend-physical-damage-reduce-percent | 1200 | int |  |
| record | defend-physical-value-modify | 1200 | int |  |
| record | defend-power-creature-value | 1200 | int |  |
| record | defend-power-equip-value | 1200 | int |  |
| record | defend-resist-power-creature-value | 1200 | int |  |
| record | defend-resist-power-equip-value | 1200 | int |  |
| record | defend-stiff-duration-base-percent | 1200 | int |  |
| record | defend-stiff-duration-value | 1200 | int |  |
| record | defend-stiff-duration-value-modify | 1200 | int |  |
| record | heal-power-base-percent | 1200 | int |  |
| record | heal-power-modify | 1200 | int |  |
| record | hp-regen | 1200 | int |  |
| record | hp-regen-combat | 1200 | int |  |
| record | job | 1200 | string |  |
| record | job-ability-1 | 1200 | int |  |
| record | job-ability-2 | 1200 | int |  |
| record | mastery-level | 1200 | int |  |
| record | max-guard-gauge | 1200 | int |  |
| record | max-hp | 1200 | int |  |
| record | max-sp | 1200 | int |  |
| record | modify-cast-speed-percent | 1200 | int |  |
| record | race | 1200 | string |  |
| record | speed | 1200 | int |  |

### mastery-level

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | additional-stat-point-max | 50 | int |  |
| record | cannot-mastery-exp-up-in-min-exp | 16 | enum-like string ❓ | y |
| record | current-stat-point | 50 | int |  |
| record | mastery-exp | 50 | int |  |
| record | mastery-level | 50 | int |  |
| record | skill-build-up-point | 50 | int |  |
| record | stat-point-attacker-max | 50 | int |  |
| record | stat-point-defender-max | 50 | int |  |

### mastery-stat-point

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | event-effect-1 | 482 | string |  |
| record | event-effect-2 | 231 | string |  |
| record | event-effect-3 | 116 | string |  |
| record | stat-effect | 500 | string |  |
| record | stat-point | 500 | int |  |
| record | stat-point-attribute | 500 | enum-like string ❓ | attacker, defender |

### mastery-stat-point-pick

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | effect | 325 | string |  |
| record | required-stat-point | 390 | int |  |
| record | stat-point-job | 390 | string |  |
| record | stat-point-pick-attribute | 390 | string |  |
| record | stat-point-tier | 390 | int |  |

### mysterious-store

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 2 | string |  |
| record | default-item-group | 2 | string |  |
| record | group-prob-1 | 2 | int |  |
| record | group-prob-10 | 2 | int |  |
| record | group-prob-11 | 2 | int |  |
| record | group-prob-12 | 2 | int |  |
| record | group-prob-13 | 2 | int |  |
| record | group-prob-14 | 2 | int |  |
| record | group-prob-2 | 2 | int |  |
| record | group-prob-3 | 2 | int |  |
| record | group-prob-4 | 2 | int |  |
| record | group-prob-5 | 2 | int |  |
| record | group-prob-6 | 2 | int |  |
| record | group-prob-7 | 2 | int |  |
| record | group-prob-8 | 2 | int |  |
| record | group-prob-9 | 2 | int |  |
| record | group-redice-count-1 | 2 | int |  |
| record | group-redice-count-10 | 2 | int |  |
| record | group-redice-count-11 | 2 | int |  |
| record | group-redice-count-12 | 2 | int |  |
| record | group-redice-count-13 | 2 | int |  |
| record | group-redice-count-14 | 2 | int |  |
| record | group-redice-count-2 | 2 | int |  |
| record | group-redice-count-3 | 2 | int |  |
| record | group-redice-count-4 | 2 | int |  |
| record | group-redice-count-5 | 2 | int |  |
| record | group-redice-count-6 | 2 | int |  |
| record | group-redice-count-7 | 2 | int |  |
| record | group-redice-count-8 | 2 | int |  |
| record | group-redice-count-9 | 2 | int |  |
| record | id | 2 | int |  |
| record | item-group-1 | 2 | string |  |
| record | item-group-10 | 2 | string |  |
| record | item-group-11 | 2 | string |  |
| record | item-group-12 | 2 | string |  |
| record | item-group-13 | 2 | string |  |
| record | item-group-14 | 2 | string |  |
| record | item-group-2 | 2 | string |  |
| record | item-group-3 | 2 | string |  |
| record | item-group-4 | 2 | string |  |
| record | item-group-5 | 2 | string |  |
| record | item-group-6 | 2 | string |  |
| record | item-group-7 | 2 | string |  |
| record | item-group-8 | 2 | string |  |
| record | item-group-9 | 2 | string |  |

### mysterious-store-item-group

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 28 | string |  |
| record | buy-price-1 | 28 | string |  |
| record | buy-price-2 | 28 | string |  |
| record | buy-price-3 | 28 | string |  |
| record | buy-price-4 | 28 | string |  |
| record | buy-price-5 | 28 | string |  |
| record | buy-price-6 | 28 | string |  |
| record | buy-price-7 | 28 | string |  |
| record | buy-price-8 | 28 | string |  |
| record | id | 28 | int |  |
| record | item-1 | 28 | string |  |
| record | item-2 | 28 | string |  |
| record | item-3 | 28 | string |  |
| record | item-4 | 28 | string |  |
| record | item-5 | 28 | string |  |
| record | item-6 | 28 | string |  |
| record | item-7 | 28 | string |  |
| record | item-8 | 28 | string |  |
| record | item-count-1 | 28 | int |  |
| record | item-count-2 | 28 | int |  |
| record | item-count-3 | 28 | int |  |
| record | item-count-4 | 28 | int |  |
| record | item-count-5 | 28 | int |  |
| record | item-count-6 | 28 | int |  |
| record | item-count-7 | 28 | int |  |
| record | item-count-8 | 28 | int |  |

### newbie-care

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | achievement-register | 6 | string |  |
| record | alias | 6 | string |  |
| record | id | 6 | int |  |

### npc

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | abnormal-attack-base-percent | 34434 | int |  |
| record | abnormal-attack-power-modify | 34434 | int |  |
| record | abnormal-attack-power-value | 34434 | int |  |
| record | abnormal-defend-base-percent | 34434 | int |  |
| record | abnormal-defend-power-modify | 34434 | int |  |
| record | abnormal-defend-power-value | 34434 | int |  |
| record | alias | 34434 | string |  |
| record | aoe-defend-base-percent | 34434 | int |  |
| record | aoe-defend-power-modify | 34434 | int |  |
| record | aoe-defend-power-value | 34434 | int |  |
| record | apply-pouch-pickup-rights-for-team | 26189 | enum-like string ❓ | n |
| record | attack-abnormal-hit-base-percent | 34434 | int |  |
| record | attack-abnormal-hit-value | 34434 | int |  |
| record | attack-aoe-pierce-value | 34434 | int |  |
| record | attack-attribute-modify | 34434 | int |  |
| record | attack-attribute-value | 34434 | int |  |
| record | attack-concentrate-value | 34434 | int |  |
| record | attack-counter-damage-modify | 34434 | int |  |
| record | attack-critical-base-percent | 34434 | int |  |
| record | attack-critical-damage-modify | 34434 | int |  |
| record | attack-critical-damage-percent | 34434 | int |  |
| record | attack-critical-damage-value | 34434 | int |  |
| record | attack-critical-value | 34434 | int |  |
| record | attack-critical-value-modify | 34434 | int |  |
| record | attack-damage-modify-diff | 34434 | int |  |
| record | attack-damage-modify-percent | 34434 | int |  |
| record | attack-hit-base-percent | 34434 | int |  |
| record | attack-hit-value | 34434 | int |  |
| record | attack-hit-value-modify | 34434 | int |  |
| record | attack-parry-pierce-modify | 34434 | int |  |
| record | attack-parry-pierce-percent | 34434 | int |  |
| record | attack-perfect-parry-damage-modify | 34434 | int |  |
| record | attack-pierce-base-percent | 34434 | int |  |
| record | attack-pierce-modify | 34434 | int |  |
| record | attack-pierce-value | 34434 | int |  |
| record | attack-power-creature-max | 34434 | int |  |
| record | attack-power-creature-min | 34434 | int |  |
| record | attack-power-equip-max | 34434 | int |  |
| record | attack-power-equip-min | 34434 | int |  |
| record | attack-stiff-duration-base-percent | 34434 | int |  |
| record | attack-stiff-duration-value | 34434 | int |  |
| record | attack-stiff-duration-value-modify | 34434 | int |  |
| record | attackable | 34434 | enum-like string ❓ | n, y |
| record | boss-challenge-attraction-score | 158 | int |  |
| record | boss-npc | 2717 | string |  |
| record | brain | 34433 | string |  |
| record | brain-parameters | 34434 | string |  |
| record | cast-duration-base-percent | 34434 | int |  |
| record | cast-duration-value | 34434 | int |  |
| record | cast-duration-value-modify | 34434 | int |  |
| record | chase-distance | 34434 | int |  |
| record | chase-radius | 34434 | int |  |
| record | contribution-day-reward-fri | 62 | string |  |
| record | contribution-day-reward-mon | 62 | string |  |
| record | contribution-day-reward-sat | 62 | string |  |
| record | contribution-day-reward-sun | 62 | string |  |
| record | contribution-day-reward-thu | 62 | string |  |
| record | contribution-day-reward-tue | 62 | string |  |
| record | contribution-day-reward-wed | 62 | string |  |
| record | contribution-peaktime-reward-fri | 2 | string |  |
| record | contribution-peaktime-reward-fri-end-hour | 26490 | int |  |
| record | contribution-peaktime-reward-fri-start-hour | 26490 | int |  |
| record | contribution-peaktime-reward-mon | 2 | string |  |
| record | contribution-peaktime-reward-mon-end-hour | 26490 | int |  |
| record | contribution-peaktime-reward-mon-start-hour | 26490 | int |  |
| record | contribution-peaktime-reward-sat | 7 | string |  |
| record | contribution-peaktime-reward-sat-end-hour | 26490 | int |  |
| record | contribution-peaktime-reward-sat-start-hour | 26490 | int |  |
| record | contribution-peaktime-reward-sun | 7 | string |  |
| record | contribution-peaktime-reward-sun-end-hour | 26490 | int |  |
| record | contribution-peaktime-reward-sun-start-hour | 26490 | int |  |
| record | contribution-peaktime-reward-thu | 2 | string |  |
| record | contribution-peaktime-reward-thu-end-hour | 26490 | int |  |
| record | contribution-peaktime-reward-thu-start-hour | 26490 | int |  |
| record | contribution-peaktime-reward-tue | 2 | string |  |
| record | contribution-peaktime-reward-tue-end-hour | 26490 | int |  |
| record | contribution-peaktime-reward-tue-start-hour | 26490 | int |  |
| record | contribution-peaktime-reward-wed | 2 | string |  |
| record | contribution-peaktime-reward-wed-end-hour | 26490 | int |  |
| record | contribution-peaktime-reward-wed-start-hour | 26490 | int |  |
| record | contribution-reward | 221 | string |  |
| record | deadbody-fielditemdrop | 8 | string |  |
| record | deadbody-hide-duration | 9474 | int |  |
| record | deadbody-pickup-duration | 8 | int |  |
| record | defend-abnormal-dodge-base-percent | 34434 | int |  |
| record | defend-abnormal-dodge-value | 34434 | int |  |
| record | defend-bounce-percent | 34434 | int |  |
| record | defend-counter-reduce-modify | 34434 | int |  |
| record | defend-counter-reduce-percent | 34434 | int |  |
| record | defend-critical-base-percent | 34434 | int |  |
| record | defend-critical-damage-modify | 34434 | int |  |
| record | defend-critical-damage-percent | 34434 | int |  |
| record | defend-critical-value | 34434 | int |  |
| record | defend-critical-value-modify | 34434 | int |  |
| record | defend-damage-modify-diff | 34434 | int |  |
| record | defend-damage-modify-percent | 34434 | int |  |
| record | defend-dodge-base-percent | 34434 | int |  |
| record | defend-dodge-value | 34434 | int |  |
| record | defend-dodge-value-modify | 34434 | int |  |
| record | defend-force-damage-reduce-percent | 34434 | int |  |
| record | defend-force-value-modify | 34434 | int |  |
| record | defend-immune-base-percent | 34434 | int |  |
| record | defend-miss-base-percent | 34434 | int |  |
| record | defend-parry-base-percent | 34434 | int |  |
| record | defend-parry-reduce-diff | 34434 | int |  |
| record | defend-parry-reduce-modify | 34434 | int |  |
| record | defend-parry-reduce-percent | 34434 | int |  |
| record | defend-parry-value | 34434 | int |  |
| record | defend-parry-value-modify | 34434 | int |  |
| record | defend-perfect-parry-base-percent | 34434 | int |  |
| record | defend-perfect-parry-reduce-modify | 34434 | int |  |
| record | defend-perfect-parry-reduce-percent | 34434 | int |  |
| record | defend-physical-damage-reduce-percent | 34434 | int |  |
| record | defend-physical-value-modify | 34434 | int |  |
| record | defend-power-creature-value | 34434 | int |  |
| record | defend-power-equip-value | 34434 | int |  |
| record | defend-resist-power-creature-value | 34434 | int |  |
| record | defend-resist-power-equip-value | 34434 | int |  |
| record | defend-stiff-duration-base-percent | 34434 | int |  |
| record | defend-stiff-duration-value | 34434 | int |  |
| record | defend-stiff-duration-value-modify | 34434 | int |  |
| record | depot | 102 | enum-like string ❓ | y |
| record | detect-fixed-range | 6819 | enum-like string ❓ | y |
| record | detect-height-bottom | 151 | int |  |
| record | detect-height-top | 369 | int |  |
| record | detect-hiding | 4646 | string |  |
| record | detect-range | 34434 | int |  |
| record | detect-signal | 5238 | string |  |
| record | distribution-pouch | 34434 | enum-like string ❓ | n, y |
| record | enable-co-ownership-pouch | 3934 | enum-like string ❓ | n, y |
| record | equip_hand | 14672 | string |  |
| record | event-reward | 3219 | string |  |
| record | exchange-faction-score | 34434 | enum-like string ❓ | n, y |
| record | exchange-faction-score-max-faction-level | 34431 | int |  |
| record | exchange-faction-score-multiply-value | 2 | int |  |
| record | faction | 1599 | int |  |
| record | fielditemdrop | 3456 | string |  |
| record | formal-radius | 34434 | int |  |
| record | ghost-type | 34434 | string |  |
| record | grade2 | 34434 | string |  |
| record | guild | 18 | enum-like string ❓ | y |
| record | hate-base-percent | 34434 | int |  |
| record | hate-on | 34434 | enum-like string ❓ | n, y |
| record | hate-power-creature-value | 34434 | int |  |
| record | hate-power-modify | 34434 | int |  |
| record | heal-power-base-percent | 34434 | int |  |
| record | heal-power-modify | 34434 | int |  |
| record | hp-regen | 34434 | int |  |
| record | hp-regen-combat | 8863 | int |  |
| record | immune-breaker-attribute-1 | 2222 | string |  |
| record | immune-breaker-attribute-2 | 2215 | string |  |
| record | immune-breaker-attribute-3 | 2215 | string |  |
| record | immune-breaker-count | 2226 | int |  |
| record | immune-breaker-init-duration | 2226 | int |  |
| record | immune-effect-attribute-1 | 34434 | string |  |
| record | immune-effect-attribute-10 | 8764 | string |  |
| record | immune-effect-attribute-11 | 7136 | string |  |
| record | immune-effect-attribute-12 | 7133 | string |  |
| record | immune-effect-attribute-13 | 5314 | string |  |
| record | immune-effect-attribute-14 | 4892 | string |  |
| record | immune-effect-attribute-15 | 4892 | string |  |
| record | immune-effect-attribute-2 | 9753 | string |  |
| record | immune-effect-attribute-3 | 8884 | string |  |
| record | immune-effect-attribute-4 | 8822 | string |  |
| record | immune-effect-attribute-5 | 8817 | string |  |
| record | immune-effect-attribute-6 | 8792 | string |  |
| record | immune-effect-attribute-7 | 8792 | string |  |
| record | immune-effect-attribute-8 | 8792 | string |  |
| record | immune-effect-attribute-9 | 8768 | string |  |
| record | immune-skill-attribute-1 | 34434 | string |  |
| record | immune-skill-attribute-2 | 10338 | string |  |
| record | immune-skill-attribute-3 | 7956 | string |  |
| record | immune-skill-attribute-4 | 7021 | string |  |
| record | immune-skill-attribute-5 | 6778 | string |  |
| record | is-mysterious-store | 2 | enum-like string ❓ | y |
| record | join-leave-faction | 34433 | enum-like string ❓ | n, y |
| record | level | 34434 | int |  |
| record | manipulate-duration | 639 | int |  |
| record | manipulate-transit-zone | 3728 | int |  |
| record | market | 95 | enum-like string ❓ | y |
| record | mastery-level | 34434 | int |  |
| record | max-faction-score | 26443 | int |  |
| record | max-hp | 34434 | int |  |
| record | max-manipulate-transit-zone-count | 3728 | int |  |
| record | max-sp | 8699 | int |  |
| record | mysterious-store | 2 | string |  |
| record | neutral-hostile | 34434 | enum-like string ❓ | fightback, neutral, offensive |
| record | notify-boast | 6 | enum-like string ❓ | y |
| record | peaktime-reward-fri-end-hour | 26490 | int |  |
| record | peaktime-reward-fri-start-hour | 26490 | int |  |
| record | peaktime-reward-mon-end-hour | 26490 | int |  |
| record | peaktime-reward-mon-start-hour | 26490 | int |  |
| record | peaktime-reward-sat-end-hour | 26490 | int |  |
| record | peaktime-reward-sat-start-hour | 26490 | int |  |
| record | peaktime-reward-sun-end-hour | 26490 | int |  |
| record | peaktime-reward-sun-start-hour | 26490 | int |  |
| record | peaktime-reward-thu-end-hour | 26490 | int |  |
| record | peaktime-reward-thu-start-hour | 26490 | int |  |
| record | peaktime-reward-tue-end-hour | 26490 | int |  |
| record | peaktime-reward-tue-start-hour | 26490 | int |  |
| record | peaktime-reward-wed-end-hour | 26490 | int |  |
| record | peaktime-reward-wed-start-hour | 26490 | int |  |
| record | post-office | 93 | enum-like string ❓ | y |
| record | production | 62 | string |  |
| record | r-attack-abnormal-hit-modify | 34434 | int |  |
| record | r-attack-aoe-pierce-modify | 34434 | int |  |
| record | r-defend-abnormal-dodge-modify | 34434 | int |  |
| record | r-support-power-modify | 34434 | int |  |
| record | radius | 34434 | int |  |
| record | repairer | 124 | enum-like string ❓ | y |
| record | reward-default | 4327 | string |  |
| record | reward-difficulty-type-1 | 98 | string |  |
| record | reward-difficulty-type-2 | 158 | string |  |
| record | reward-difficulty-type-3 | 158 | string |  |
| record | reward-exp | 34434 | int |  |
| record | reward-faction-score | 324 | int |  |
| record | run-speed | 34434 | int |  |
| record | side | 10967 | string |  |
| record | soul-npc | 34434 | enum-like string ❓ | n, y |
| record | soul-npc-skill-level | 34434 | int |  |
| record | stance | 34434 | string |  |
| record | store2-1 | 525 | string |  |
| record | store2-2 | 149 | string |  |
| record | store2-3 | 44 | string |  |
| record | store2-4 | 29 | string |  |
| record | store2-5 | 4 | string |  |
| record | store2-6 | 1 | string |  |
| record | summoned-beauty-shop | 8 | string |  |
| record | support-power-base-percent | 34434 | int |  |
| record | support-power-value | 34434 | int |  |
| record | teleport | 1 | int |  |
| record | transfer-faction | 34434 | enum-like string ❓ | n, y |
| record | walk-speed | 34434 | int |  |
| record | write-combat-log | 34434 | string |  |

### npc-brain-parameters

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 34446 | string |  |
| record | bleeding-1 | 2176 | int |  |
| record | bleeding-2 | 1161 | int |  |
| record | bleeding-3 | 707 | int |  |
| record | combat-sequence-1 | 34446 | string |  |
| record | combat-sequence-10 | 218 | string |  |
| record | combat-sequence-2 | 1487 | string |  |
| record | combat-sequence-3 | 770 | string |  |
| record | combat-sequence-4 | 620 | string |  |
| record | combat-sequence-5 | 428 | string |  |
| record | combat-sequence-6 | 358 | string |  |
| record | combat-sequence-7 | 326 | string |  |
| record | combat-sequence-8 | 280 | string |  |
| record | combat-sequence-9 | 248 | string |  |
| record | despawn-social | 9257 | string |  |
| record | event-invoke-effect-attribute-1 | 57 | string |  |
| record | event-invoke-effect-attribute-2 | 2 | string |  |
| record | event-invoked-effect-attribute-1 | 4295 | string |  |
| record | event-invoked-effect-attribute-2 | 1151 | string |  |
| record | event-invoked-effect-attribute-3 | 757 | string |  |
| record | event-invoked-effect-attribute-4 | 419 | string |  |
| record | hate-share-social | 58 | string |  |
| record | hide-off-social | 622 | string |  |
| record | hide-on-init | 4405 | string |  |
| record | hide-on-return-start | 2597 | string |  |
| record | hide-on-social | 627 | string |  |
| record | hide-type | 4516 | string |  |
| record | max-attack-distance-1 | 34446 | int |  |
| record | max-attack-distance-2 | 34446 | int |  |
| record | max-attack-distance-3 | 34446 | int |  |
| record | max-attack-distance-4 | 34446 | int |  |
| record | max-attack-distance-5 | 34446 | int |  |
| record | max-attack-distance-6 | 34446 | int |  |
| record | max-attack-distance-7 | 34446 | int |  |
| record | max-attack-height-1 | 34446 | int |  |
| record | max-attack-height-2 | 34446 | int |  |
| record | max-attack-height-3 | 34446 | int |  |
| record | max-attack-height-4 | 34446 | int |  |
| record | max-attack-height-5 | 34446 | int |  |
| record | max-attack-height-6 | 34446 | int |  |
| record | max-attack-height-7 | 34446 | int |  |
| record | max-close-in-distance-1 | 34446 | int |  |
| record | max-close-in-distance-2 | 34446 | int |  |
| record | max-close-in-distance-3 | 34446 | int |  |
| record | max-close-in-distance-4 | 34446 | int |  |
| record | max-close-in-distance-5 | 34446 | int |  |
| record | max-close-in-distance-6 | 34446 | int |  |
| record | max-close-in-distance-7 | 34446 | int |  |
| record | min-attack-distance-1 | 34446 | int |  |
| record | min-attack-distance-2 | 34446 | int |  |
| record | min-attack-distance-3 | 34446 | int |  |
| record | min-attack-distance-4 | 34446 | int |  |
| record | min-attack-distance-5 | 34446 | int |  |
| record | min-attack-distance-6 | 34446 | int |  |
| record | min-attack-distance-7 | 34446 | int |  |
| record | min-close-in-distance-1 | 34446 | int |  |
| record | min-close-in-distance-2 | 34446 | int |  |
| record | min-close-in-distance-3 | 34446 | int |  |
| record | min-close-in-distance-4 | 34446 | int |  |
| record | min-close-in-distance-5 | 34446 | int |  |
| record | min-close-in-distance-6 | 34446 | int |  |
| record | min-close-in-distance-7 | 34446 | int |  |
| record | monster-talkable | 161 | string |  |
| record | period-bind-cur-target-sec | 33394 | int |  |
| record | reach-1 | 34446 | int |  |
| record | reach-2 | 34446 | int |  |
| record | reach-3 | 34446 | int |  |
| record | return-end-social | 2 | string |  |
| record | return-start-social | 2 | string |  |
| record | script | 34446 | string |  |
| record | setup-dir-1 | 4675 | enum-like string ❓ | target, y |
| record | setup-dir-2 | 19 | string |  |
| record | setup-position-1 | 6449 | string |  |
| record | setup-position-2 | 13 | enum-like string ❓ | target, y |
| record | skill-1 | 6004 | string |  |
| record | skill-10 | 632 | string |  |
| record | skill-11 | 498 | string |  |
| record | skill-12 | 344 | string |  |
| record | skill-14 | 1971 | string |  |
| record | skill-15 | 1548 | string |  |
| record | skill-16 | 1348 | string |  |
| record | skill-17 | 924 | string |  |
| record | skill-18 | 39 | string |  |
| record | skill-19 | 31 | string |  |
| record | skill-2 | 5640 | string |  |
| record | skill-3 | 4291 | string |  |
| record | skill-4 | 2795 | string |  |
| record | skill-5 | 1041 | string |  |
| record | skill-6 | 35 | string |  |
| record | skill-7 | 17 | string |  |
| record | skill-8 | 3872 | string |  |
| record | skill-9 | 1019 | string |  |
| record | skill3-1 | 6004 | string |  |
| record | skill3-10 | 632 | string |  |
| record | skill3-11 | 498 | string |  |
| record | skill3-12 | 344 | string |  |
| record | skill3-14 | 1971 | string |  |
| record | skill3-15 | 1548 | string |  |
| record | skill3-16 | 1348 | string |  |
| record | skill3-17 | 924 | string |  |
| record | skill3-18 | 39 | string |  |
| record | skill3-19 | 31 | string |  |
| record | skill3-2 | 5640 | string |  |
| record | skill3-3 | 4291 | string |  |
| record | skill3-4 | 2795 | string |  |
| record | skill3-5 | 1041 | string |  |
| record | skill3-6 | 35 | string |  |
| record | skill3-7 | 17 | string |  |
| record | skill3-8 | 3872 | string |  |
| record | skill3-9 | 1019 | string |  |
| record | social-1 | 2710 | string |  |
| record | social-10 | 3958 | string |  |
| record | social-11 | 3958 | string |  |
| record | social-12 | 3958 | string |  |
| record | social-2 | 1229 | string |  |
| record | social-3 | 621 | string |  |
| record | social-4 | 169 | string |  |
| record | social-5 | 4746 | string |  |
| record | social-6 | 4746 | string |  |
| record | social-7 | 4748 | string |  |
| record | social-8 | 4746 | string |  |
| record | social-9 | 3959 | string |  |
| record | soul-npc-skill-category-1 | 33394 | string |  |
| record | soul-npc-skill-category-2 | 33394 | string |  |
| record | soul-npc-skill-category-3 | 33394 | string |  |
| record | soul-npc-skill-category-4 | 33394 | string |  |
| record | spawn-skill3 | 468 | string |  |
| record | spawn-social | 8301 | string |  |
| record | stance-1 | 34446 | string |  |
| record | stance-2 | 14087 | string |  |
| record | stance-3 | 15 | string |  |
| record | stance-effect-1 | 7 | string |  |
| record | stance-effect-2 | 7 | string |  |
| record | stance-effect-3 | 7 | string |  |
| record | targeting-type | 33394 | enum-like string ❓ | hate-top, hate-tour |
| record | timer-1 | 51 | int |  |
| record | timer-2 | 43 | int |  |
| record | timer-3 | 41 | int |  |
| record | timer-4 | 36 | int |  |
| record | timer-5 | 10 | int |  |
| record | timer-6 | 5 | int |  |
| record | timer-7 | 3 | int |  |
| record | timer-8 | 3 | int |  |
| record | transit-type | 2592 | enum-like string ❓ | by-event, by-timer |
| record | weapon-1 | 14676 | string |  |
| record | weapon-2 | 270 | string |  |
| record | weapon-3 | 7 | string |  |

### npc-reset-stage

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 227 | string |  |
| record | clear-npc-reg | 22 | enum-like string ❓ | n, y |
| record | clear-party-reg | 17 | enum-like string ❓ | n, y |
| record | despawn-force-1 | 38 | enum-like string ❓ | n, y |
| record | despawn-force-10 | 2 | enum-like string ❓ | y |
| record | despawn-force-2 | 29 | enum-like string ❓ | n, y |
| record | despawn-force-3 | 24 | enum-like string ❓ | n, y |
| record | despawn-force-4 | 18 | enum-like string ❓ | n, y |
| record | despawn-force-5 | 13 | enum-like string ❓ | n, y |
| record | despawn-force-6 | 8 | enum-like string ❓ | y |
| record | despawn-force-7 | 8 | enum-like string ❓ | y |
| record | despawn-force-8 | 6 | enum-like string ❓ | y |
| record | despawn-force-9 | 2 | enum-like string ❓ | y |
| record | despawn-my-party | 27 | enum-like string ❓ | n, y |
| record | despawn-npc-group-1 | 89 | string |  |
| record | despawn-npc-group-10 | 2 | string |  |
| record | despawn-npc-group-2 | 74 | string |  |
| record | despawn-npc-group-3 | 67 | string |  |
| record | despawn-npc-group-4 | 60 | string |  |
| record | despawn-npc-group-5 | 42 | string |  |
| record | despawn-npc-group-6 | 8 | string |  |
| record | despawn-npc-group-7 | 8 | string |  |
| record | despawn-npc-group-8 | 6 | string |  |
| record | despawn-npc-group-9 | 2 | string |  |
| record | despawn-npc-max-id | 2 | int |  |
| record | despawn-npc-min-id | 2 | int |  |
| record | respawn-after-despawn-1 | 37 | enum-like string ❓ | n |
| record | respawn-after-despawn-10 | 2 | enum-like string ❓ | n |
| record | respawn-after-despawn-2 | 28 | enum-like string ❓ | n |
| record | respawn-after-despawn-3 | 24 | enum-like string ❓ | n |
| record | respawn-after-despawn-4 | 18 | enum-like string ❓ | n |
| record | respawn-after-despawn-5 | 13 | enum-like string ❓ | n |
| record | respawn-after-despawn-6 | 8 | enum-like string ❓ | n |
| record | respawn-after-despawn-7 | 8 | enum-like string ❓ | n |
| record | respawn-after-despawn-8 | 6 | enum-like string ❓ | n |
| record | respawn-after-despawn-9 | 2 | enum-like string ❓ | n |
| record | set-env-operation-1 | 82 | enum-like string ❓ | close, disable, enable, open |
| record | set-env-operation-10 | 18 | enum-like string ❓ | close, disable, enable, open |
| record | set-env-operation-11 | 14 | enum-like string ❓ | close, disable, enable, open |
| record | set-env-operation-12 | 14 | enum-like string ❓ | close, disable, enable, open |
| record | set-env-operation-13 | 14 | enum-like string ❓ | disable, enable |
| record | set-env-operation-14 | 12 | enum-like string ❓ | disable, enable |
| record | set-env-operation-15 | 10 | enum-like string ❓ | disable, enable |
| record | set-env-operation-16 | 6 | enum-like string ❓ | disable, enable |
| record | set-env-operation-17 | 2 | enum-like string ❓ | disable, enable |
| record | set-env-operation-18 | 2 | enum-like string ❓ | disable, enable |
| record | set-env-operation-19 | 2 | enum-like string ❓ | disable, enable |
| record | set-env-operation-2 | 69 | enum-like string ❓ | close, disable, enable |
| record | set-env-operation-20 | 2 | enum-like string ❓ | disable, enable |
| record | set-env-operation-21 | 2 | enum-like string ❓ | disable, enable |
| record | set-env-operation-22 | 2 | enum-like string ❓ | disable, enable |
| record | set-env-operation-23 | 2 | enum-like string ❓ | close, open |
| record | set-env-operation-24 | 2 | enum-like string ❓ | close, open |
| record | set-env-operation-25 | 2 | enum-like string ❓ | close, open |
| record | set-env-operation-26 | 2 | enum-like string ❓ | close, open |
| record | set-env-operation-27 | 2 | enum-like string ❓ | disable, enable |
| record | set-env-operation-28 | 2 | enum-like string ❓ | disable, enable |
| record | set-env-operation-29 | 2 | enum-like string ❓ | disable, enable |
| record | set-env-operation-3 | 61 | enum-like string ❓ | close, disable, enable, open |
| record | set-env-operation-30 | 2 | enum-like string ❓ | disable, enable |
| record | set-env-operation-31 | 2 | enum-like string ❓ | disable, enable |
| record | set-env-operation-32 | 2 | enum-like string ❓ | disable, enable |
| record | set-env-operation-33 | 2 | enum-like string ❓ | disable, enable |
| record | set-env-operation-34 | 2 | enum-like string ❓ | disable, enable |
| record | set-env-operation-35 | 2 | enum-like string ❓ | disable, enable |
| record | set-env-operation-36 | 2 | enum-like string ❓ | disable, enable |
| record | set-env-operation-37 | 2 | enum-like string ❓ | disable, enable |
| record | set-env-operation-38 | 2 | enum-like string ❓ | disable, enable |
| record | set-env-operation-4 | 50 | enum-like string ❓ | close, disable, enable |
| record | set-env-operation-5 | 46 | enum-like string ❓ | close, disable, enable |
| record | set-env-operation-6 | 46 | enum-like string ❓ | close, disable, enable |
| record | set-env-operation-7 | 18 | enum-like string ❓ | close, disable, enable |
| record | set-env-operation-8 | 18 | enum-like string ❓ | close, disable, enable, open |
| record | set-env-operation-9 | 18 | enum-like string ❓ | close, disable, enable, open |
| record | set-env2-target-1 | 82 | string |  |
| record | set-env2-target-10 | 18 | string |  |
| record | set-env2-target-11 | 14 | string |  |
| record | set-env2-target-12 | 14 | string |  |
| record | set-env2-target-13 | 14 | string |  |
| record | set-env2-target-14 | 12 | string |  |
| record | set-env2-target-15 | 10 | string |  |
| record | set-env2-target-16 | 6 | string |  |
| record | set-env2-target-17 | 2 | string |  |
| record | set-env2-target-18 | 2 | string |  |
| record | set-env2-target-19 | 2 | string |  |
| record | set-env2-target-2 | 69 | string |  |
| record | set-env2-target-20 | 2 | string |  |
| record | set-env2-target-21 | 2 | string |  |
| record | set-env2-target-22 | 2 | string |  |
| record | set-env2-target-23 | 2 | string |  |
| record | set-env2-target-24 | 2 | string |  |
| record | set-env2-target-25 | 2 | string |  |
| record | set-env2-target-26 | 2 | string |  |
| record | set-env2-target-27 | 2 | string |  |
| record | set-env2-target-28 | 2 | string |  |
| record | set-env2-target-29 | 2 | string |  |
| record | set-env2-target-3 | 61 | string |  |
| record | set-env2-target-30 | 2 | string |  |
| record | set-env2-target-31 | 2 | string |  |
| record | set-env2-target-32 | 2 | string |  |
| record | set-env2-target-33 | 2 | string |  |
| record | set-env2-target-34 | 2 | string |  |
| record | set-env2-target-35 | 2 | string |  |
| record | set-env2-target-36 | 2 | string |  |
| record | set-env2-target-37 | 2 | string |  |
| record | set-env2-target-38 | 2 | string |  |
| record | set-env2-target-4 | 50 | string |  |
| record | set-env2-target-5 | 46 | string |  |
| record | set-env2-target-6 | 46 | string |  |
| record | set-env2-target-7 | 18 | string |  |
| record | set-env2-target-8 | 18 | string |  |
| record | set-env2-target-9 | 18 | string |  |
| record | spawn-delay-1 | 134 | int |  |
| record | spawn-delay-2 | 32 | int |  |
| record | spawn-delay-3 | 23 | int |  |
| record | spawn-delay-4 | 17 | int |  |
| record | spawn-delay-5 | 10 | int |  |
| record | spawn-my-party | 27 | enum-like string ❓ | n, y |
| record | spawn-npc-group-1 | 145 | string |  |
| record | spawn-npc-group-2 | 38 | string |  |
| record | spawn-npc-group-3 | 29 | string |  |
| record | spawn-npc-group-4 | 20 | string |  |
| record | spawn-npc-group-5 | 12 | string |  |
| record | spawn-npc-max-id | 2 | int |  |
| record | spawn-npc-min-id | 2 | int |  |

### party-battle-field-zone

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | add-point-lose | 3 | int |  |
| record | add-point-rate-lose | 3 | int |  |
| record | add-point-rate-win | 3 | int |  |
| record | add-point-win | 3 | int |  |
| record | alias | 3 | string |  |
| record | alpha-control-point-1 | 1 | string |  |
| record | alpha-control-point-2 | 1 | string |  |
| record | alpha-control-point-3 | 1 | string |  |
| record | alpha-goal-post | 1 | string |  |
| record | alpha-side-respawn | 3 | string |  |
| record | arrow-control-point-1 | 1 | string |  |
| record | arrow-control-point-2 | 1 | string |  |
| record | arrow-control-point-3 | 1 | string |  |
| record | arrow-control-point-4 | 1 | string |  |
| record | arrow-control-point-5 | 1 | string |  |
| record | arrow-control-point-6 | 1 | string |  |
| record | arrow-control-point-enable-delay-second-1 | 1 | int |  |
| record | arrow-control-point-enable-delay-second-2 | 1 | int |  |
| record | arrow-control-point-enable-delay-second-3 | 1 | int |  |
| record | arrow-control-point-enable-delay-second-4 | 1 | int |  |
| record | arrow-control-point-enable-delay-second-5 | 1 | int |  |
| record | arrow-control-point-enable-delay-second-6 | 1 | int |  |
| record | arrow-control-point-init-state-1 | 1 | string |  |
| record | arrow-control-point-init-state-4 | 1 | string |  |
| record | available-random-matching-weekly-time | 3 | string |  |
| record | ball-npc-1 | 1 | string |  |
| record | ball-npc-10 | 1 | string |  |
| record | ball-npc-11 | 1 | string |  |
| record | ball-npc-12 | 1 | string |  |
| record | ball-npc-13 | 1 | string |  |
| record | ball-npc-14 | 1 | string |  |
| record | ball-npc-15 | 1 | string |  |
| record | ball-npc-16 | 1 | string |  |
| record | ball-npc-17 | 1 | string |  |
| record | ball-npc-18 | 1 | string |  |
| record | ball-npc-19 | 1 | string |  |
| record | ball-npc-2 | 1 | string |  |
| record | ball-npc-20 | 1 | string |  |
| record | ball-npc-3 | 1 | string |  |
| record | ball-npc-4 | 1 | string |  |
| record | ball-npc-5 | 1 | string |  |
| record | ball-npc-6 | 1 | string |  |
| record | ball-npc-7 | 1 | string |  |
| record | ball-npc-8 | 1 | string |  |
| record | ball-npc-9 | 1 | string |  |
| record | ball-spawn-interval-second-1 | 1 | int |  |
| record | ball-spawn-interval-second-2 | 1 | int |  |
| record | ball-spawn-interval-second-3 | 1 | int |  |
| record | ball-spawn-max-count | 1 | int |  |
| record | ball-speed-effect-2 | 1 | string |  |
| record | ball-speed-effect-3 | 1 | string |  |
| record | beta-control-point-1 | 1 | string |  |
| record | beta-control-point-2 | 1 | string |  |
| record | beta-control-point-3 | 1 | string |  |
| record | beta-goal-post | 1 | string |  |
| record | beta-side-respawn | 3 | string |  |
| record | bonus-exp-percent-1 | 3 | int |  |
| record | bonus-exp-percent-2 | 3 | int |  |
| record | bonus-point-percent-1 | 3 | int |  |
| record | bonus-point-percent-2 | 3 | int |  |
| record | consecutive-goal-bonus-1 | 1 | int |  |
| record | consecutive-goal-bonus-2 | 1 | int |  |
| record | consecutive-goal-bonus-3 | 1 | int |  |
| record | consecutive-goal-bonus-duration-second | 1 | int |  |
| record | control-point-1 | 1 | string |  |
| record | control-point-2 | 1 | string |  |
| record | control-point-3 | 1 | string |  |
| record | double-occupation-bonus-score | 1 | int |  |
| record | enable-infinite-hyper-energy | 3 | enum-like string ❓ | y |
| record | enable-no-game-decision-by-idleness | 3 | enum-like string ❓ | y |
| record | enter-alpha-side-pc-spawn | 3 | string |  |
| record | enter-beta-side-pc-spawn | 3 | string |  |
| record | flag-spawn-env | 1 | string |  |
| record | flag-spawn-env-respawn-duration-second-1 | 1 | int |  |
| record | flag-spawn-env-respawn-duration-second-2 | 1 | int |  |
| record | flag-spawn-env-respawn-duration-second-3 | 1 | int |  |
| record | flag-spawn-env-respawn-duration-second-4 | 1 | int |  |
| record | flag-spawn-env-respawn-duration-second-5 | 1 | int |  |
| record | goal-in-score-1 | 1 | int |  |
| record | goal-in-score-2 | 1 | int |  |
| record | goal-in-score-3 | 1 | int |  |
| record | goal-post-enable-delay-second | 1 | int |  |
| record | goal-score | 3 | int |  |
| record | group | 3 | string |  |
| record | id | 3 | int |  |
| record | init-ball-spawn-delay-second | 1 | int |  |
| record | kill-score | 3 | int |  |
| record | no-game-decision-duration-second | 3 | int |  |
| record | no-game-decision-min-damage | 3 | int |  |
| record | occupation-effect-1 | 1 | string |  |
| record | occupation-effect-2 | 1 | string |  |
| record | opening-tip-off-control-point | 1 | string |  |
| record | play-duration-second | 3 | int |  |
| record | play-section-duration-second-1 | 1 | int |  |
| record | play-section-duration-second-2 | 1 | int |  |
| record | ready-duration-second | 3 | int |  |
| record | required-level | 3 | int |  |
| record | required-mastery-level | 3 | int |  |
| record | required-member-count-team-matching | 3 | int |  |
| record | respawn-delay | 3 | int |  |
| record | set-env-operation-1 | 3 | enum-like string ❓ | open |
| record | set-env-operation-10 | 1 | enum-like string ❓ | enable |
| record | set-env-operation-2 | 3 | enum-like string ❓ | open |
| record | set-env-operation-3 | 3 | enum-like string ❓ | enable, open |
| record | set-env-operation-4 | 3 | enum-like string ❓ | enable, open |
| record | set-env-operation-5 | 1 | enum-like string ❓ | open |
| record | set-env-operation-6 | 1 | enum-like string ❓ | open |
| record | set-env-operation-7 | 1 | enum-like string ❓ | open |
| record | set-env-operation-8 | 1 | enum-like string ❓ | open |
| record | set-env-operation-9 | 1 | enum-like string ❓ | enable |
| record | set-env-time-1 | 3 | int |  |
| record | set-env-time-10 | 1 | int |  |
| record | set-env-time-2 | 3 | int |  |
| record | set-env-time-3 | 3 | int |  |
| record | set-env-time-4 | 3 | int |  |
| record | set-env-time-5 | 1 | int |  |
| record | set-env-time-6 | 1 | int |  |
| record | set-env-time-7 | 1 | int |  |
| record | set-env-time-8 | 1 | int |  |
| record | set-env-time-9 | 1 | int |  |
| record | set-env2-target-1 | 3 | string |  |
| record | set-env2-target-10 | 1 | string |  |
| record | set-env2-target-2 | 3 | string |  |
| record | set-env2-target-3 | 3 | string |  |
| record | set-env2-target-4 | 3 | string |  |
| record | set-env2-target-5 | 1 | string |  |
| record | set-env2-target-6 | 1 | string |  |
| record | set-env2-target-7 | 1 | string |  |
| record | set-env2-target-8 | 1 | string |  |
| record | set-env2-target-9 | 1 | string |  |
| record | start-alpha-side-pc-spawn | 3 | string |  |
| record | start-beta-side-pc-spawn | 3 | string |  |
| record | steal-flag-effect-1 | 1 | string |  |
| record | steal-flag-effect-2 | 1 | string |  |
| record | type | 3 | enum-like string ❓ | capture-the-flag, lead-the-ball, occupation-war |
| record | vote-delay-second | 1 | int |  |
| record | vote-duration-second | 3 | int |  |
| record | weekly-time-table-for-added-reward-1 | 3 | string |  |
| record | weekly-time-table-for-added-reward-2 | 3 | string |  |
| record | whole-occupation-bonus-score | 1 | int |  |
| record | zone | 3 | int |  |

### partymatch

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 93 | string |  |
| record | faction | 2 | string |  |

### pc

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | ability-rate-max-guard-gauge-equip | 10200 | int |  |
| record | abnormal-attack-base-percent | 10200 | int |  |
| record | abnormal-attack-power-modify | 10200 | int |  |
| record | abnormal-defend-power-modify | 10200 | int |  |
| record | aoe-defend-base-percent | 10200 | int |  |
| record | aoe-defend-power-modify | 10200 | int |  |
| record | aoe-defend-power-value | 10200 | int |  |
| record | attack-abnormal-hit-base-percent | 10200 | int |  |
| record | attack-abnormal-hit-value | 10200 | int |  |
| record | attack-aoe-pierce-value | 10200 | int |  |
| record | attack-attribute-base-percent | 10200 | int |  |
| record | attack-attribute-modify | 10200 | int |  |
| record | attack-concentrate-value | 10200 | int |  |
| record | attack-counter-damage-modify | 10200 | int |  |
| record | attack-critical-base-percent | 10200 | int |  |
| record | attack-critical-damage-modify | 10200 | int |  |
| record | attack-critical-damage-percent | 10200 | int |  |
| record | attack-critical-damage-value | 8160 | int |  |
| record | attack-critical-value | 10200 | int |  |
| record | attack-critical-value-modify | 10200 | int |  |
| record | attack-damage-modify-diff | 10200 | int |  |
| record | attack-damage-modify-percent | 10200 | int |  |
| record | attack-hit-base-percent | 10200 | int |  |
| record | attack-hit-value | 10200 | int |  |
| record | attack-hit-value-modify | 10200 | int |  |
| record | attack-parry-pierce-modify | 10200 | int |  |
| record | attack-perfect-parry-damage-modify | 10200 | int |  |
| record | attack-pierce-base-percent | 10200 | int |  |
| record | attack-pierce-modify | 10200 | int |  |
| record | attack-pierce-value | 10200 | int |  |
| record | attack-power-creature-max | 10200 | int |  |
| record | attack-power-creature-min | 10200 | int |  |
| record | attack-power-equip-max | 10200 | int |  |
| record | attack-power-equip-min | 10200 | int |  |
| record | attack-precise-creature-value | 10200 | int |  |
| record | attack-stiff-duration-base-percent | 10200 | int |  |
| record | attack-stiff-duration-value | 10200 | int |  |
| record | attack-stiff-duration-value-modify | 10200 | int |  |
| record | cast-duration-base-percent | 10200 | int |  |
| record | cast-duration-value | 10200 | int |  |
| record | cast-duration-value-modify | 10200 | int |  |
| record | contents-type | 10200 | string |  |
| record | defend-abnormal-dodge-base-percent | 10200 | int |  |
| record | defend-abnormal-dodge-value | 10200 | int |  |
| record | defend-bounce-percent | 10200 | int |  |
| record | defend-counter-reduce-modify | 10200 | int |  |
| record | defend-counter-reduce-percent | 10200 | int |  |
| record | defend-critical-base-percent | 10200 | int |  |
| record | defend-critical-damage-modify | 10200 | int |  |
| record | defend-critical-damage-percent | 10200 | int |  |
| record | defend-critical-value | 10200 | int |  |
| record | defend-critical-value-modify | 10200 | int |  |
| record | defend-damage-modify-diff | 10200 | int |  |
| record | defend-damage-modify-percent | 10200 | int |  |
| record | defend-dodge-base-percent | 10200 | int |  |
| record | defend-dodge-value | 10200 | int |  |
| record | defend-dodge-value-modify | 10200 | int |  |
| record | defend-force-damage-reduce-percent | 10200 | int |  |
| record | defend-force-value-modify | 10200 | int |  |
| record | defend-immune-base-percent | 10200 | int |  |
| record | defend-miss-base-percent | 10200 | int |  |
| record | defend-parry-base-percent | 10200 | int |  |
| record | defend-parry-reduce-diff | 10200 | int |  |
| record | defend-parry-reduce-modify | 10200 | int |  |
| record | defend-parry-reduce-percent | 10200 | int |  |
| record | defend-parry-value | 10200 | int |  |
| record | defend-parry-value-modify | 10200 | int |  |
| record | defend-perfect-parry-base-percent | 10200 | int |  |
| record | defend-perfect-parry-reduce-modify | 10200 | int |  |
| record | defend-perfect-parry-reduce-percent | 10200 | int |  |
| record | defend-physical-damage-reduce-percent | 10200 | int |  |
| record | defend-physical-value-modify | 10200 | int |  |
| record | defend-power-creature-value | 10200 | int |  |
| record | defend-power-equip-value | 10200 | int |  |
| record | defend-resist-power-creature-value | 10200 | int |  |
| record | defend-resist-power-equip-value | 10200 | int |  |
| record | defend-stiff-duration-base-percent | 10200 | int |  |
| record | defend-stiff-duration-value | 10200 | int |  |
| record | defend-stiff-duration-value-modify | 10200 | int |  |
| record | defend-strength-creature-value | 10200 | int |  |
| record | hate-base-percent | 10200 | int |  |
| record | hate-power-creature-value | 10200 | int |  |
| record | hate-power-modify | 10200 | int |  |
| record | heal-power-base-percent | 10200 | int |  |
| record | heal-power-modify | 10200 | int |  |
| record | hp-regen | 10200 | int |  |
| record | hp-regen-combat | 10200 | int |  |
| record | id | 10200 | int |  |
| record | job | 10200 | string |  |
| record | job-ability-1 | 8360 | int |  |
| record | job-ability-2 | 8360 | int |  |
| record | level | 10200 | int |  |
| record | max-guard-gauge | 10200 | int |  |
| record | max-hp | 10200 | int |  |
| record | max-sp | 10200 | int |  |
| record | pc-job-style-skill3-1 | 660 | string |  |
| record | pc-job-style-skill3-2 | 660 | string |  |
| record | pc-job-style-skill3-6 | 663 | string |  |
| record | pc-job-style-skill3-7 | 663 | string |  |
| record | pc-job-style-skill3-8 | 203 | string |  |
| record | r-attack-abnormal-hit-modify | 10200 | int |  |
| record | r-attack-aoe-pierce-modify | 10200 | int |  |
| record | r-defend-abnormal-dodge-modify | 10200 | int |  |
| record | r-support-power-modify | 10200 | int |  |
| record | race | 10200 | string |  |
| record | radius | 10200 | int |  |
| record | sex | 10200 | string |  |
| record | skill-build-up-level-max | 2040 | int |  |
| record | skill-build-up-point | 2040 | int |  |
| record | sp-regen | 10200 | int |  |
| record | speed | 10200 | int |  |
| record | support-power-base-percent | 10200 | int |  |
| record | support-power-value | 10200 | int |  |

### pc-race-sex-job

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | job | 46 | string |  |
| record | race | 46 | string |  |
| record | sex | 46 | string |  |

### pet-food-recovery

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 3 | string |  |
| record | item-grade-1 | 3 | int |  |
| record | item-grade-2 | 3 | int |  |
| record | item-grade-3 | 3 | int |  |
| record | recovery-amount-1 | 3 | int |  |
| record | recovery-amount-2 | 3 | int |  |
| record | recovery-amount-3 | 3 | int |  |

### public-raid

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 7 | string |  |
| record | arena-entrance-zone | 1 | int |  |
| record | bottom | 5 | int |  |
| record | center-x | 5 | int |  |
| record | center-y | 5 | int |  |
| record | center-z | 5 | int |  |
| record | change-state-env-by-start | 5 | string |  |
| record | enable-public-raid-event | 5 | enum-like string ❓ | y |
| record | enable-reset-combat-mode | 5 | enum-like string ❓ | y |
| record | enter-pc-spawn | 7 | string |  |
| record | group | 6 | string |  |
| record | id | 7 | int |  |
| record | max-party-count | 1 | int |  |
| record | max-pc-count | 6 | int |  |
| record | radius | 5 | int |  |
| record | required-available-weekly-time | 7 | string |  |
| record | required-level | 7 | int |  |
| record | reset-combat-mode-recycle-duration | 5 | int |  |
| record | round-id | 7 | int |  |
| record | stand-by-duration-second | 5 | int |  |
| record | start-weekly-time | 5 | string |  |
| record | target-npc-to-finish-state | 5 | string |  |
| record | top | 5 | int |  |
| record | type | 7 | string |  |
| record | zone | 7 | int |  |

### public-raid-event

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | add-score-amount | 2 | int |  |
| record | add-score-recycle-duration | 2 | int |  |
| record | alias | 2 | string |  |
| record | event-duration | 2 | int |  |
| record | final-score-1 | 2 | int |  |
| record | final-score-1-target-npc-1 | 2 | string |  |
| record | final-score-1-target-npc-effect-1 | 2 | string |  |
| record | final-score-1-target-pc-effect-1 | 2 | string |  |
| record | final-score-1-target-summoned-effect-1 | 2 | string |  |
| record | final-score-2 | 2 | int |  |
| record | final-score-2-target-npc-1 | 2 | string |  |
| record | final-score-2-target-npc-2 | 2 | string |  |
| record | final-score-2-target-npc-effect-1 | 2 | string |  |
| record | final-score-2-target-npc-effect-2 | 2 | string |  |
| record | final-score-3 | 2 | int |  |
| record | final-score-3-target-npc-1 | 2 | string |  |
| record | final-score-3-target-npc-2 | 2 | string |  |
| record | final-score-3-target-npc-effect-1 | 2 | string |  |
| record | final-score-3-target-npc-effect-2 | 2 | string |  |
| record | final-score-3-target-pc-effect-1 | 2 | string |  |
| record | final-score-3-target-pc-effect-2 | 2 | string |  |
| record | final-score-4 | 2 | int |  |
| record | final-score-4-target-npc-1 | 2 | string |  |
| record | final-score-4-target-npc-2 | 2 | string |  |
| record | final-score-4-target-npc-effect-1 | 2 | string |  |
| record | final-score-4-target-npc-effect-2 | 2 | string |  |
| record | final-score-4-target-pc-effect-1 | 2 | string |  |
| record | final-score-4-target-pc-effect-2 | 2 | string |  |
| record | final-score-5 | 2 | int |  |
| record | final-score-5-target-npc-1 | 2 | string |  |
| record | final-score-5-target-npc-2 | 2 | string |  |
| record | final-score-5-target-npc-effect-1 | 2 | string |  |
| record | final-score-5-target-npc-effect-2 | 2 | string |  |
| record | final-score-5-target-pc-effect-1 | 2 | string |  |
| record | final-score-5-target-pc-effect-2 | 2 | string |  |
| record | final-score-6 | 2 | int |  |
| record | id | 2 | int |  |
| record | reduce-score-amount | 1 | int |  |
| record | start-score-amount | 2 | int |  |
| record | type | 2 | enum-like string ❓ | score-event |

### quest

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| quest | activated-faction | 92 | string |  |
| quest | alias | 2610 | string |  |
| quest | attraction-1 | 216 | string |  |
| quest | attraction-2 | 52 | string |  |
| quest | attraction-3 | 27 | string |  |
| quest | attraction-4 | 18 | string |  |
| quest | broadcast-category | 1285 | enum-like string ❓ | always, field, sextet, solo-quartet |
| quest | category | 2610 | enum-like string ❓ | attraction, dungeon, epic, job, normal, tendency-simple, tendency-tendency |
| quest | completed-list | 341 | enum-like string ❓ | n, y |
| quest | desc | 2 | string |  |
| quest | district-1 | 891 | enum-like string ❓ | deasamak, jeryoungrim, startzone, suwal |
| quest | district-2 | 6 | enum-like string ❓ | deasamak, suwal |
| quest | dungeon | 117 | string |  |
| quest | extra-quest-complete-achievement-1 | 3 | string |  |
| quest | extra-quest-complete-achievement-2 | 2 | string |  |
| quest | extra-quest-complete-achievement-3 | 2 | string |  |
| quest | grade | 2 | int |  |
| quest | group | 1 | string |  |
| quest | group2 | 2 | string |  |
| quest | id | 2610 | int |  |
| quest | invoke-fx-msg | 1 | enum-like string ❓ | y |
| quest | main-faction | 103 | string |  |
| quest | map-group-1-1 | 2 | enum-like string ❓ | anywhere, jeryoungrim |
| quest | max-repeat | 2605 | int |  |
| quest | name | 1 | string |  |
| quest | name2 | 2 | string |  |
| quest | production | 8 | string |  |
| quest | progress-difficulty-type-1 | 63 | enum-like string ❓ | n, y |
| quest | progress-difficulty-type-2 | 80 | enum-like string ❓ | n, y |
| quest | progress-difficulty-type-3 | 111 | enum-like string ❓ | n, y |
| quest | reset-by-acquire-time | 51 | enum-like string ❓ | daily, none |
| quest | reset-enable | 11 | enum-like string ❓ | n, y |
| quest | reset-item-1 | 5 | string |  |
| quest | reset-item-count-1 | 6 | int |  |
| quest | reset-type | 561 | enum-like string ❓ | daily, none, weekly |
| quest | retired | 632 | enum-like string ❓ | n, y |
| quest | save-type | 2569 | enum-like string ❓ | all, except-completion, except-completion-and-logout-save, nothing |
| quest | show-tutorial-tag | 2 | enum-like string ❓ | y |
| quest | tutorial | 328 | enum-like string ❓ | n, y |
| quest/acquisition | check-tencent-vitality | 228 | enum-like string ❓ | n |
| quest/acquisition | faction | 82 | string |  |
| quest/acquisition | faction-level | 16 | int |  |
| quest/acquisition | faction-reputation | 6 | int |  |
| quest/acquisition | job-1 | 113 | string |  |
| quest/acquisition | job-10 | 1 | string |  |
| quest/acquisition | job-11 | 1 | string |  |
| quest/acquisition | job-2 | 2 | string |  |
| quest/acquisition | job-3 | 2 | string |  |
| quest/acquisition | job-4 | 2 | string |  |
| quest/acquisition | job-5 | 2 | string |  |
| quest/acquisition | job-6 | 2 | string |  |
| quest/acquisition | job-7 | 2 | string |  |
| quest/acquisition | job-8 | 2 | string |  |
| quest/acquisition | job-9 | 2 | string |  |
| quest/acquisition | level | 1747 | int |  |
| quest/acquisition | mastery-level | 23 | int |  |
| quest/acquisition | preceding-quest-1 | 975 | string |  |
| quest/acquisition | preceding-quest-10 | 1 | string |  |
| quest/acquisition | preceding-quest-11 | 1 | string |  |
| quest/acquisition | preceding-quest-12 | 1 | string |  |
| quest/acquisition | preceding-quest-2 | 275 | string |  |
| quest/acquisition | preceding-quest-3 | 18 | string |  |
| quest/acquisition | preceding-quest-4 | 3 | string |  |
| quest/acquisition | preceding-quest-5 | 1 | string |  |
| quest/acquisition | preceding-quest-6 | 1 | string |  |
| quest/acquisition | preceding-quest-7 | 1 | string |  |
| quest/acquisition | preceding-quest-8 | 1 | string |  |
| quest/acquisition | preceding-quest-9 | 1 | string |  |
| quest/acquisition | preceding-quest-check | 1239 | enum-like string ❓ | and, or |
| quest/acquisition | preceding-quest-count-1 | 4 | int |  |
| quest/acquisition | preceding-quest-mission-step-1 | 186 | int |  |
| quest/acquisition | preceding-quest-mission-step-2 | 10 | int |  |
| quest/acquisition | preceding-quest-mission-step-3 | 1 | int |  |
| quest/acquisition | preceding-quest-retired-1 | 16 | string |  |
| quest/acquisition | production-exp | 104 | int |  |
| quest/acquisition | production-id | 104 | string |  |
| quest/acquisition | recommended-level | 2 | int |  |
| quest/acquisition | reward-1 | 92 | string |  |
| quest/acquisition | reward-2 | 1 | string |  |
| quest/acquisition | valid-dayofweek-start-day-3 | 1 | enum-like string ❓ | none |
| quest/acquisition | valid-dayofweek-start-day-4 | 1 | enum-like string ❓ | none |
| quest/acquisition/acquisition-loss | item-1 | 45 | string |  |
| quest/acquisition/acquisition-loss | item-count-1 | 41 | int |  |
| quest/acquisition/case | acquire-quest | 1903 | enum-like string ❓ | n, y |
| quest/acquisition/case | completion-count | 619 | int |  |
| quest/acquisition/case | completion-count-op | 280 | enum-like string ❓ | ge, lt |
| quest/acquisition/case | drop-gadget | 16 | enum-like string ❓ | n, y |
| quest/acquisition/case | fielditem | 5 | string |  |
| quest/acquisition/case | gadget-required | 13 | enum-like string ❓ | dont-care, empty |
| quest/acquisition/case | grocery | 41 | string |  |
| quest/acquisition/case | item | 88 | string |  |
| quest/acquisition/case | loot-item | 200 | string |  |
| quest/acquisition/case | npc-response | 1 | string |  |
| quest/acquisition/case | object | 1375 | string |  |
| quest/acquisition/case | object2 | 496 | string |  |
| quest/acquisition/case | party-broadcast | 1840 | enum-like string ❓ | n |
| quest/acquisition/case | prob | 10 | int |  |
| quest/acquisition/case | progress-mission | 267 | enum-like string ❓ | n, reaction-only, y |
| quest/acquisition/case | quest-decision | 34 | string |  |
| quest/acquisition/case | quest-symbol-drop-prob | 123 | int |  |
| quest/acquisition/case | remove-grocery | 7 | enum-like string ❓ | y |
| quest/acquisition/case | required-item-1 | 39 | string |  |
| quest/acquisition/case | required-item-depot | 42 | enum-like string ❓ | bag, none |
| quest/acquisition/case | required-item-loss | 39 | enum-like string ❓ | y |
| quest/acquisition/case | team-broadcast | 2 | enum-like string ❓ | n |
| quest/acquisition/case | type | 2100 | enum-like string ❓ | approach, enter-portal, enter-zone, loot, manipulate, npc-manipulate, pick-up-fielditem, talk, talk-to-item, talk-to-self |
| quest/acquisition/case | valid-zone-1 | 3 | string |  |
| quest/acquisition/case | visible-object | 22 | enum-like string ❓ | n, y |
| quest/acquisition/case/filter-set | name | 12 | string |  |
| quest/acquisition/case/filter-set/filter | field | 17 | enum-like string ❓ | faction, level |
| quest/acquisition/case/filter-set/filter | op | 17 | enum-like string ❓ | eq, ge, le |
| quest/acquisition/case/filter-set/filter | subject | 11 | string |  |
| quest/acquisition/case/filter-set/filter | subject2 | 164 | string |  |
| quest/acquisition/case/filter-set/filter | type | 164 | enum-like string ❓ | active-faction, faction, prop |
| quest/acquisition/case/filter-set/filter | value | 164 | string |  |
| quest/acquisition/fixed-reward | item-count-2 | 2 | int |  |
| quest/acquisition/fixed-reward | item-count-3 | 2 | int |  |
| quest/acquisition/fixed-reward | item-count-4 | 2 | int |  |
| quest/acquisition/fixed-reward | race-3 | 1 | enum-like string ❓ | race-none |
| quest/acquisition/fixed-reward | sex-3 | 2 | enum-like string ❓ | sex-none |
| quest/acquisition/fixed-reward | slot-1 | 13 | string |  |
| quest/completion/next-quest | faction | 8 | string |  |
| quest/completion/next-quest | job-1 | 50 | string |  |
| quest/completion/next-quest | quest | 371 | string |  |
| quest/giveup-loss | item-1 | 80 | string |  |
| quest/giveup-loss | item-2 | 20 | string |  |
| quest/giveup-loss | item-3 | 11 | string |  |
| quest/giveup-loss | item-4 | 8 | string |  |
| quest/giveup-loss | item-count-1 | 24 | int |  |
| quest/giveup-loss | item-count-2 | 11 | int |  |
| quest/giveup-loss | item-count-3 | 9 | int |  |
| quest/giveup-loss | item-count-4 | 7 | int |  |
| quest/giveup-loss | job-1 | 14 | string |  |
| quest/mission-step | completion-type | 453 | enum-like string ❓ | and, or |
| quest/mission-step | giveup-warp-to-pcspawn | 183 | string |  |
| quest/mission-step | giveup-zone-1 | 183 | string |  |
| quest/mission-step | giveup-zone-2 | 66 | string |  |
| quest/mission-step | giveup-zone-3 | 17 | string |  |
| quest/mission-step | guide-message | 1 | string |  |
| quest/mission-step | guide-message-zone-1 | 1 | string |  |
| quest/mission-step | hide | 16 | enum-like string ❓ | n, y |
| quest/mission-step | id | 6706 | int |  |
| quest/mission-step | quest-decision | 1065 | string |  |
| quest/mission-step | retired | 45 | enum-like string ❓ | n, y |
| quest/mission-step | rollback | 1063 | enum-like string ❓ | leave-world, leave-zone, none, remove-fielditem |
| quest/mission-step | skip-dest-mission-step | 2 | int |  |
| quest/mission-step | skip-quest-decision | 2 | string |  |
| quest/mission-step | skip-quest-decision-zone | 2 | string |  |
| quest/mission-step | time-limit | 16 | int |  |
| quest/mission-step | time-limit-type | 29 | enum-like string ❓ | none, until |
| quest/mission-step | zone-1 | 1186 | string |  |
| quest/mission-step | zone-2 | 3 | string |  |
| quest/mission-step/mission | check-tencent-vitality | 11 | enum-like string ❓ | n |
| quest/mission-step/mission | id | 7028 | int |  |
| quest/mission-step/mission | name | 1 | string |  |
| quest/mission-step/mission | name2 | 6 | string |  |
| quest/mission-step/mission | required-attraction | 497 | string |  |
| quest/mission-step/mission | required-register-value | 6530 | int |  |
| quest/mission-step/mission | reset-teleport-recycle-time | 178 | enum-like string ❓ | n, y |
| quest/mission-step/mission | reward-1 | 1532 | string |  |
| quest/mission-step/mission | reward-2 | 408 | string |  |
| quest/mission-step/mission | simple-quest-play-section | 167 | enum-like string ❓ | enemy, our |
| quest/mission-step/mission | tendency-id | 497 | int |  |
| quest/mission-step/mission | variation-required-condition-type | 371 | enum-like string ❓ | faction-score, field-play-point |
| quest/mission-step/mission | variation-required-condition-value-1 | 497 | int |  |
| quest/mission-step/mission | variation-required-condition-value-2 | 218 | int |  |
| quest/mission-step/mission | variation-required-condition-value-3 | 218 | int |  |
| quest/mission-step/mission | variation-required-condition-value-4 | 76 | int |  |
| quest/mission-step/mission | variation-required-condition-value-5 | 76 | int |  |
| quest/mission-step/mission | variation-required-condition-value-6 | 76 | int |  |
| quest/mission-step/mission | variation-required-condition-value-7 | 76 | int |  |
| quest/mission-step/mission | variation-required-condition-value-8 | 76 | int |  |
| quest/mission-step/mission | variation-required-field-play-point-1 | 497 | int |  |
| quest/mission-step/mission | variation-required-field-play-point-2 | 76 | int |  |
| quest/mission-step/mission | variation-required-field-play-point-3 | 76 | int |  |
| quest/mission-step/mission | variation-required-field-play-point-4 | 76 | int |  |
| quest/mission-step/mission | variation-required-field-play-point-5 | 76 | int |  |
| quest/mission-step/mission | variation-required-field-play-point-6 | 76 | int |  |
| quest/mission-step/mission | variation-required-field-play-point-7 | 76 | int |  |
| quest/mission-step/mission | variation-required-field-play-point-8 | 76 | int |  |
| quest/mission-step/mission | variation-required-register-value-1 | 497 | int |  |
| quest/mission-step/mission | variation-required-register-value-2 | 218 | int |  |
| quest/mission-step/mission | variation-required-register-value-3 | 218 | int |  |
| quest/mission-step/mission | variation-required-register-value-4 | 76 | int |  |
| quest/mission-step/mission | variation-required-register-value-5 | 76 | int |  |
| quest/mission-step/mission | variation-required-register-value-6 | 76 | int |  |
| quest/mission-step/mission | variation-required-register-value-7 | 76 | int |  |
| quest/mission-step/mission | variation-required-register-value-8 | 76 | int |  |
| quest/mission-step/mission | variation-reward-faction-score-1 | 295 | int |  |
| quest/mission-step/mission | variation-reward-faction-score-2 | 142 | int |  |
| quest/mission-step/mission | variation-reward-faction-score-3 | 142 | int |  |
| quest/mission-step/mission | variation-reward-field-play-point-1 | 497 | int |  |
| quest/mission-step/mission | variation-reward-field-play-point-2 | 218 | int |  |
| quest/mission-step/mission | variation-reward-field-play-point-3 | 218 | int |  |
| quest/mission-step/mission | variation-reward-field-play-point-4 | 76 | int |  |
| quest/mission-step/mission | variation-reward-field-play-point-5 | 76 | int |  |
| quest/mission-step/mission | variation-reward-field-play-point-6 | 76 | int |  |
| quest/mission-step/mission | variation-reward-field-play-point-7 | 76 | int |  |
| quest/mission-step/mission | variation-reward-field-play-point-8 | 76 | int |  |
| quest/mission-step/mission | variation-reward-tendency-score-1 | 497 | int |  |
| quest/mission-step/mission | variation-reward-tendency-score-2 | 218 | int |  |
| quest/mission-step/mission | variation-reward-tendency-score-3 | 218 | int |  |
| quest/mission-step/mission | variation-reward-tendency-score-4 | 76 | int |  |
| quest/mission-step/mission | variation-reward-tendency-score-5 | 76 | int |  |
| quest/mission-step/mission | variation-reward-tendency-score-6 | 76 | int |  |
| quest/mission-step/mission | variation-reward-tendency-score-7 | 76 | int |  |
| quest/mission-step/mission | variation-reward-tendency-score-8 | 76 | int |  |
| quest/mission-step/mission-step-fail | dispose-quest | 19 | enum-like string ❓ | n, y |
| quest/mission-step/mission-step-fail | quest-decision | 74 | string |  |
| quest/mission-step/mission-step-fail | rollback-step-id | 780 | int |  |
| quest/mission-step/mission-step-fail | zone-1 | 686 | string |  |
| quest/mission-step/mission-step-fail | zone-2 | 4 | string |  |
| quest/mission-step/mission-step-fail/case | convoy | 39 | string |  |
| quest/mission-step/mission-step-fail/case | object | 39 | string |  |
| quest/mission-step/mission-step-fail/case | party-broadcast | 36 | enum-like string ❓ | n |
| quest/mission-step/mission-step-fail/case | progress-mission | 31 | enum-like string ❓ | reaction-only |
| quest/mission-step/mission-step-fail/case | quest-decision | 1 | string |  |
| quest/mission-step/mission-step-fail/case | type | 39 | enum-like string ❓ | convoy-failed |
| quest/mission-step/mission/case | acquire-quest | 69 | enum-like string ❓ | n, y |
| quest/mission-step/mission/case | approach-social | 9 | string |  |
| quest/mission-step/mission/case | arena-matching-rule-detail | 9 | enum-like string ❓ | none, normal |
| quest/mission-step/mission/case | battle-royal-field | 2 | string |  |
| quest/mission-step/mission/case | completion-count | 618 | int |  |
| quest/mission-step/mission/case | completion-count-op | 289 | enum-like string ❓ | ge, le, lt |
| quest/mission-step/mission/case | convoy | 76 | string |  |
| quest/mission-step/mission/case | convoy-member-1 | 1 | int |  |
| quest/mission-step/mission/case | convoy-member-2 | 1 | int |  |
| quest/mission-step/mission/case | drop-gadget | 344 | enum-like string ❓ | n, y |
| quest/mission-step/mission/case | duel-result | 7 | enum-like string ❓ | win |
| quest/mission-step/mission/case | duel-type | 13 | string |  |
| quest/mission-step/mission/case | faction | 16 | string |  |
| quest/mission-step/mission/case | fail-quest-decision | 11 | string |  |
| quest/mission-step/mission/case | fee | 263 | enum-like string ❓ | n, y |
| quest/mission-step/mission/case | gadget | 290 | string |  |
| quest/mission-step/mission/case | gadget-required | 906 | enum-like string ❓ | carry, carry-and-remove, dont-care, empty, not-a |
| quest/mission-step/mission/case | grocery | 169 | string |  |
| quest/mission-step/mission/case | grocery-count | 14 | int |  |
| quest/mission-step/mission/case | idx | 183 | int |  |
| quest/mission-step/mission/case | item | 71 | string |  |
| quest/mission-step/mission/case | join-faction2 | 3 | string |  |
| quest/mission-step/mission/case | killed-difficulty-type | 15 | enum-like string ❓ | hard, none |
| quest/mission-step/mission/case | loot-item | 85 | string |  |
| quest/mission-step/mission/case | multi-object-1 | 521 | string |  |
| quest/mission-step/mission/case | multi-object-10 | 16 | string |  |
| quest/mission-step/mission/case | multi-object-11 | 16 | string |  |
| quest/mission-step/mission/case | multi-object-12 | 15 | string |  |
| quest/mission-step/mission/case | multi-object-13 | 13 | string |  |
| quest/mission-step/mission/case | multi-object-14 | 13 | string |  |
| quest/mission-step/mission/case | multi-object-15 | 12 | string |  |
| quest/mission-step/mission/case | multi-object-16 | 12 | string |  |
| quest/mission-step/mission/case | multi-object-2 | 252 | string |  |
| quest/mission-step/mission/case | multi-object-3 | 146 | string |  |
| quest/mission-step/mission/case | multi-object-4 | 97 | string |  |
| quest/mission-step/mission/case | multi-object-5 | 53 | string |  |
| quest/mission-step/mission/case | multi-object-6 | 43 | string |  |
| quest/mission-step/mission/case | multi-object-7 | 27 | string |  |
| quest/mission-step/mission/case | multi-object-8 | 20 | string |  |
| quest/mission-step/mission/case | multi-object-9 | 18 | string |  |
| quest/mission-step/mission/case | object | 11518 | string |  |
| quest/mission-step/mission/case | object-1 | 4 | string |  |
| quest/mission-step/mission/case | object-2 | 2 | string |  |
| quest/mission-step/mission/case | object-3 | 2 | string |  |
| quest/mission-step/mission/case | object2 | 5668 | string |  |
| quest/mission-step/mission/case | object2-1 | 30 | string |  |
| quest/mission-step/mission/case | object2-2 | 15 | string |  |
| quest/mission-step/mission/case | object2-3 | 11 | string |  |
| quest/mission-step/mission/case | object2-4 | 9 | string |  |
| quest/mission-step/mission/case | object2-5 | 9 | string |  |
| quest/mission-step/mission/case | object2-6 | 6 | string |  |
| quest/mission-step/mission/case | object2-7 | 3 | string |  |
| quest/mission-step/mission/case | object2-8 | 2 | string |  |
| quest/mission-step/mission/case | object2-9 | 1 | string |  |
| quest/mission-step/mission/case | party-battle-result | 6 | enum-like string ❓ | win |
| quest/mission-step/mission/case | party-battle-type | 14 | enum-like string ❓ | capture-the-flag, lead-the-ball, occupation-war |
| quest/mission-step/mission/case | party-broadcast | 16767 | enum-like string ❓ | n, y |
| quest/mission-step/mission/case | prob | 44 | int |  |
| quest/mission-step/mission/case | progress-mission | 14283 | enum-like string ❓ | n, reaction-only, y |
| quest/mission-step/mission/case | progress-value | 15 | int |  |
| quest/mission-step/mission/case | quest-decision | 4077 | string |  |
| quest/mission-step/mission/case | quest-symbol-drop-prob | 242 | int |  |
| quest/mission-step/mission/case | range-max | 20 | int |  |
| quest/mission-step/mission/case | range-min | 17 | int |  |
| quest/mission-step/mission/case | remove-grocery | 67 | enum-like string ❓ | n, y |
| quest/mission-step/mission/case | required-item-1 | 145 | string |  |
| quest/mission-step/mission/case | required-item-2 | 5 | string |  |
| quest/mission-step/mission/case | required-item-3 | 2 | string |  |
| quest/mission-step/mission/case | required-item-count-1 | 2 | int |  |
| quest/mission-step/mission/case | required-item-depot | 155 | enum-like string ❓ | bag, none |
| quest/mission-step/mission/case | required-item-loss | 154 | enum-like string ❓ | n, y |
| quest/mission-step/mission/case | required-money | 1 | int |  |
| quest/mission-step/mission/case | skill | 14 | string |  |
| quest/mission-step/mission/case | skill3 | 29 | string |  |
| quest/mission-step/mission/case | specify-object-type | 6 | enum-like string ❓ | duelbot, specific-id |
| quest/mission-step/mission/case | summoned-preset | 15 | string |  |
| quest/mission-step/mission/case | team-broadcast | 127 | enum-like string ❓ | n, y |
| quest/mission-step/mission/case | type | 17134 | enum-like string ❓ | acquire-summoned, approach, battle-royal, convoy-arrived, duel-finish, enter-zone, env-entered, join-faction, killed, loot, manipulate, npc-bleeding-occured, npc-manipulate, party-battle, pc-social, skill, talk, talk-to-item, talk-to-self |
| quest/mission-step/mission/case | valid-zone-1 | 24 | string |  |
| quest/mission-step/mission/case | valid-zone-2 | 1 | string |  |
| quest/mission-step/mission/case | visible-object | 185 | enum-like string ❓ | n, y |
| quest/mission-step/mission/case/case | object | 1 | string |  |
| quest/mission-step/mission/case/case | party-broadcast | 1 | enum-like string ❓ | n |
| quest/mission-step/mission/case/case | progress-mission | 1 | enum-like string ❓ | reaction-only |
| quest/mission-step/mission/case/case | quest-decision | 1 | string |  |
| quest/mission-step/mission/case/case | type | 1 | enum-like string ❓ | enter-zone |
| quest/mission-step/mission/case/filter-set | name | 37 | string |  |
| quest/mission-step/mission/case/filter-set/filter | count-op | 1 | enum-like string ❓ | lt |
| quest/mission-step/mission/case/filter-set/filter | field | 119 | enum-like string ❓ | equip-body, equip-hand, equip-head, faction, level |
| quest/mission-step/mission/case/filter-set/filter | item | 60 | string |  |
| quest/mission-step/mission/case/filter-set/filter | op | 124 | enum-like string ❓ | eq, ge, le |
| quest/mission-step/mission/case/filter-set/filter | quest | 3 | string |  |
| quest/mission-step/mission/case/filter-set/filter | spawn | 10 | string |  |
| quest/mission-step/mission/case/filter-set/filter | subject | 267 | string |  |
| quest/mission-step/mission/case/filter-set/filter | subject2 | 593 | string |  |
| quest/mission-step/mission/case/filter-set/filter | type | 593 | enum-like string ❓ | active-faction, faction, faction-level, inventory, job, npc-spawn, number, prop, quest-complete |
| quest/mission-step/mission/case/filter-set/filter | value | 376 | string |  |
| quest/mission-step/mission/case/filter-set/filter | value-1 | 144 | string |  |
| quest/mission-step/mission/case/filter-set/filter | value-2 | 24 | string |  |
| quest/mission-step/mission/completion-loss | item-1 | 66 | string |  |
| quest/mission-step/mission/completion-loss | item-2 | 9 | string |  |
| quest/mission-step/mission/completion-loss | item-3 | 1 | string |  |
| quest/mission-step/mission/completion-loss | item-count-1 | 34 | int |  |
| quest/mission-step/mission/completion-loss | job-1 | 35 | string |  |
| quest/mission-step/mission/completion-loss | money | 2 | int |  |
| quest/mission-step/mission/fixed-reward | item-count-1 | 3 | int |  |
| quest/mission-step/mission/fixed-reward | job-1 | 54 | string |  |
| quest/mission-step/mission/fixed-reward | race-3 | 1 | enum-like string ❓ | race-none |
| quest/mission-step/mission/fixed-reward | slot-1 | 106 | string |  |
| quest/mission-step/mission/fixed-reward | slot-2 | 3 | string |  |
| quest/mission-step/mission/fixed-reward | slot-3 | 1 | string |  |
| quest/mission-step/mission/tutorial-case | object | 3 | string |  |
| quest/mission-step/mission/tutorial-case | object2 | 3 | string |  |
| quest/mission-step/mission/tutorial-case | progress-mission | 532 | enum-like string ❓ | reaction-only, y |
| quest/mission-step/mission/tutorial-case | quest-decision | 265 | string |  |
| quest/mission-step/mission/tutorial-case | range-max | 312 | int |  |
| quest/mission-step/mission/tutorial-case | range-min | 312 | int |  |
| quest/mission-step/mission/tutorial-case | type | 532 | string |  |
| quest/mission-step/mission/tutorial-case | zone | 428 | string |  |
| quest/mission-step/mission/tutorial-case/filter-set/filter | field | 1 | enum-like string ❓ | level |
| quest/mission-step/mission/tutorial-case/filter-set/filter | op | 1 | enum-like string ❓ | ge |
| quest/mission-step/mission/tutorial-case/filter-set/filter | subject2 | 1 | string |  |
| quest/mission-step/mission/tutorial-case/filter-set/filter | type | 1 | enum-like string ❓ | prop |
| quest/mission-step/mission/tutorial-case/filter-set/filter | value | 1 | int |  |
| quest/transit | id | 841 | int |  |
| quest/transit | zone | 841 | string |  |
| quest/transit/complete | zone-index | 14 | int |  |
| quest/transit/destination | mission-step-id | 3669 | int |  |
| quest/transit/destination | zone-index | 3669 | int |  |
| quest/transit/not-acquire | zone-index | 1 | int |  |

### raid-dungeon

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 15 | string |  |
| record | boss-npc-1 | 15 | string |  |
| record | boss-npc-2 | 7 | string |  |
| record | boss-npc-3 | 5 | string |  |
| record | boss-npc-4 | 2 | string |  |
| record | dungeon-type | 15 | enum-like string ❓ | bind |
| record | enable-heart-count | 15 | enum-like string ❓ | n |
| record | group | 15 | string |  |
| record | id | 15 | int |  |
| record | max-leave-count-after-combat | 4 | int |  |
| record | required-level | 15 | int |  |
| record | required-preceding-quest-1 | 15 | string |  |
| record | required-preceding-quest-2 | 2 | string |  |
| record | required-preceding-quest-check | 2 | enum-like string ❓ | or |
| record | step-count | 15 | int |  |
| record | zone-1 | 15 | string |  |
| record | zone-2 | 7 | string |  |
| record | zone-3 | 5 | string |  |
| record | zone-4 | 2 | string |  |
| record | zone-index-1 | 15 | int |  |
| record | zone-index-2 | 7 | int |  |
| record | zone-index-3 | 5 | int |  |
| record | zone-index-4 | 2 | int |  |

### random-distribution

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 8 | string |  |
| record | weight-1 | 8 | int |  |
| record | weight-10 | 8 | int |  |
| record | weight-100 | 8 | int |  |
| record | weight-101 | 8 | int |  |
| record | weight-11 | 8 | int |  |
| record | weight-12 | 8 | int |  |
| record | weight-13 | 8 | int |  |
| record | weight-14 | 8 | int |  |
| record | weight-15 | 8 | int |  |
| record | weight-16 | 8 | int |  |
| record | weight-17 | 8 | int |  |
| record | weight-18 | 8 | int |  |
| record | weight-19 | 8 | int |  |
| record | weight-2 | 8 | int |  |
| record | weight-20 | 8 | int |  |
| record | weight-21 | 8 | int |  |
| record | weight-22 | 8 | int |  |
| record | weight-23 | 8 | int |  |
| record | weight-24 | 8 | int |  |
| record | weight-25 | 8 | int |  |
| record | weight-26 | 8 | int |  |
| record | weight-27 | 8 | int |  |
| record | weight-28 | 8 | int |  |
| record | weight-29 | 8 | int |  |
| record | weight-3 | 8 | int |  |
| record | weight-30 | 8 | int |  |
| record | weight-31 | 8 | int |  |
| record | weight-32 | 8 | int |  |
| record | weight-33 | 8 | int |  |
| record | weight-34 | 8 | int |  |
| record | weight-35 | 8 | int |  |
| record | weight-36 | 8 | int |  |
| record | weight-37 | 8 | int |  |
| record | weight-38 | 8 | int |  |
| record | weight-39 | 8 | int |  |
| record | weight-4 | 8 | int |  |
| record | weight-40 | 8 | int |  |
| record | weight-41 | 8 | int |  |
| record | weight-42 | 8 | int |  |
| record | weight-43 | 8 | int |  |
| record | weight-44 | 8 | int |  |
| record | weight-45 | 8 | int |  |
| record | weight-46 | 8 | int |  |
| record | weight-47 | 8 | int |  |
| record | weight-48 | 8 | int |  |
| record | weight-49 | 8 | int |  |
| record | weight-5 | 8 | int |  |
| record | weight-50 | 8 | int |  |
| record | weight-51 | 8 | int |  |
| record | weight-52 | 8 | int |  |
| record | weight-53 | 8 | int |  |
| record | weight-54 | 8 | int |  |
| record | weight-55 | 8 | int |  |
| record | weight-56 | 8 | int |  |
| record | weight-57 | 8 | int |  |
| record | weight-58 | 8 | int |  |
| record | weight-59 | 8 | int |  |
| record | weight-6 | 8 | int |  |
| record | weight-60 | 8 | int |  |
| record | weight-61 | 8 | int |  |
| record | weight-62 | 8 | int |  |
| record | weight-63 | 8 | int |  |
| record | weight-64 | 8 | int |  |
| record | weight-65 | 8 | int |  |
| record | weight-66 | 8 | int |  |
| record | weight-67 | 8 | int |  |
| record | weight-68 | 8 | int |  |
| record | weight-69 | 8 | int |  |
| record | weight-7 | 8 | int |  |
| record | weight-70 | 8 | int |  |
| record | weight-71 | 8 | int |  |
| record | weight-72 | 8 | int |  |
| record | weight-73 | 8 | int |  |
| record | weight-74 | 8 | int |  |
| record | weight-75 | 8 | int |  |
| record | weight-76 | 8 | int |  |
| record | weight-77 | 8 | int |  |
| record | weight-78 | 8 | int |  |
| record | weight-79 | 8 | int |  |
| record | weight-8 | 8 | int |  |
| record | weight-80 | 8 | int |  |
| record | weight-81 | 8 | int |  |
| record | weight-82 | 8 | int |  |
| record | weight-83 | 8 | int |  |
| record | weight-84 | 8 | int |  |
| record | weight-85 | 8 | int |  |
| record | weight-86 | 8 | int |  |
| record | weight-87 | 8 | int |  |
| record | weight-88 | 8 | int |  |
| record | weight-89 | 8 | int |  |
| record | weight-9 | 8 | int |  |
| record | weight-90 | 8 | int |  |
| record | weight-91 | 8 | int |  |
| record | weight-92 | 8 | int |  |
| record | weight-93 | 8 | int |  |
| record | weight-94 | 8 | int |  |
| record | weight-95 | 8 | int |  |
| record | weight-96 | 8 | int |  |
| record | weight-97 | 8 | int |  |
| record | weight-98 | 8 | int |  |
| record | weight-99 | 8 | int |  |

### random-store-item

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 976 | string |  |
| record | id | 976 | int |  |
| record | item | 976 | string |  |
| record | item-count | 976 | int |  |
| record | item-price-item | 10 | string |  |
| record | item-price-item-amount | 10 | int |  |
| record | item-price-money | 976 | int |  |

### random-store-item-group

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 122 | string |  |
| record | id | 122 | int |  |
| record | random-store-item-1 | 122 | string |  |
| record | random-store-item-2 | 122 | string |  |
| record | random-store-item-3 | 122 | string |  |
| record | random-store-item-4 | 122 | string |  |
| record | random-store-item-5 | 122 | string |  |
| record | random-store-item-6 | 122 | string |  |
| record | random-store-item-7 | 122 | string |  |
| record | random-store-item-8 | 122 | string |  |
| record | ui-grade | 122 | int |  |

### random-store-prob-group

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 1 | string |  |
| record | id | 1 | int |  |
| record | normal-group-free-prob-weight-1 | 1 | int |  |
| record | normal-group-free-prob-weight-10 | 1 | int |  |
| record | normal-group-free-prob-weight-11 | 1 | int |  |
| record | normal-group-free-prob-weight-12 | 1 | int |  |
| record | normal-group-free-prob-weight-13 | 1 | int |  |
| record | normal-group-free-prob-weight-14 | 1 | int |  |
| record | normal-group-free-prob-weight-15 | 1 | int |  |
| record | normal-group-free-prob-weight-16 | 1 | int |  |
| record | normal-group-free-prob-weight-17 | 1 | int |  |
| record | normal-group-free-prob-weight-18 | 1 | int |  |
| record | normal-group-free-prob-weight-19 | 1 | int |  |
| record | normal-group-free-prob-weight-2 | 1 | int |  |
| record | normal-group-free-prob-weight-20 | 1 | int |  |
| record | normal-group-free-prob-weight-21 | 1 | int |  |
| record | normal-group-free-prob-weight-22 | 1 | int |  |
| record | normal-group-free-prob-weight-23 | 1 | int |  |
| record | normal-group-free-prob-weight-24 | 1 | int |  |
| record | normal-group-free-prob-weight-25 | 1 | int |  |
| record | normal-group-free-prob-weight-26 | 1 | int |  |
| record | normal-group-free-prob-weight-27 | 1 | int |  |
| record | normal-group-free-prob-weight-28 | 1 | int |  |
| record | normal-group-free-prob-weight-29 | 1 | int |  |
| record | normal-group-free-prob-weight-3 | 1 | int |  |
| record | normal-group-free-prob-weight-30 | 1 | int |  |
| record | normal-group-free-prob-weight-31 | 1 | int |  |
| record | normal-group-free-prob-weight-32 | 1 | int |  |
| record | normal-group-free-prob-weight-33 | 1 | int |  |
| record | normal-group-free-prob-weight-34 | 1 | int |  |
| record | normal-group-free-prob-weight-35 | 1 | int |  |
| record | normal-group-free-prob-weight-36 | 1 | int |  |
| record | normal-group-free-prob-weight-37 | 1 | int |  |
| record | normal-group-free-prob-weight-38 | 1 | int |  |
| record | normal-group-free-prob-weight-39 | 1 | int |  |
| record | normal-group-free-prob-weight-4 | 1 | int |  |
| record | normal-group-free-prob-weight-40 | 1 | int |  |
| record | normal-group-free-prob-weight-41 | 1 | int |  |
| record | normal-group-free-prob-weight-42 | 1 | int |  |
| record | normal-group-free-prob-weight-43 | 1 | int |  |
| record | normal-group-free-prob-weight-44 | 1 | int |  |
| record | normal-group-free-prob-weight-45 | 1 | int |  |
| record | normal-group-free-prob-weight-46 | 1 | int |  |
| record | normal-group-free-prob-weight-47 | 1 | int |  |
| record | normal-group-free-prob-weight-48 | 1 | int |  |
| record | normal-group-free-prob-weight-49 | 1 | int |  |
| record | normal-group-free-prob-weight-5 | 1 | int |  |
| record | normal-group-free-prob-weight-50 | 1 | int |  |
| record | normal-group-free-prob-weight-51 | 1 | int |  |
| record | normal-group-free-prob-weight-52 | 1 | int |  |
| record | normal-group-free-prob-weight-53 | 1 | int |  |
| record | normal-group-free-prob-weight-54 | 1 | int |  |
| record | normal-group-free-prob-weight-55 | 1 | int |  |
| record | normal-group-free-prob-weight-56 | 1 | int |  |
| record | normal-group-free-prob-weight-57 | 1 | int |  |
| record | normal-group-free-prob-weight-58 | 1 | int |  |
| record | normal-group-free-prob-weight-59 | 1 | int |  |
| record | normal-group-free-prob-weight-6 | 1 | int |  |
| record | normal-group-free-prob-weight-60 | 1 | int |  |
| record | normal-group-free-prob-weight-61 | 1 | int |  |
| record | normal-group-free-prob-weight-62 | 1 | int |  |
| record | normal-group-free-prob-weight-63 | 1 | int |  |
| record | normal-group-free-prob-weight-64 | 1 | int |  |
| record | normal-group-free-prob-weight-65 | 1 | int |  |
| record | normal-group-free-prob-weight-66 | 1 | int |  |
| record | normal-group-free-prob-weight-67 | 1 | int |  |
| record | normal-group-free-prob-weight-68 | 1 | int |  |
| record | normal-group-free-prob-weight-69 | 1 | int |  |
| record | normal-group-free-prob-weight-7 | 1 | int |  |
| record | normal-group-free-prob-weight-70 | 1 | int |  |
| record | normal-group-free-prob-weight-71 | 1 | int |  |
| record | normal-group-free-prob-weight-72 | 1 | int |  |
| record | normal-group-free-prob-weight-73 | 1 | int |  |
| record | normal-group-free-prob-weight-74 | 1 | int |  |
| record | normal-group-free-prob-weight-75 | 1 | int |  |
| record | normal-group-free-prob-weight-76 | 1 | int |  |
| record | normal-group-free-prob-weight-77 | 1 | int |  |
| record | normal-group-free-prob-weight-78 | 1 | int |  |
| record | normal-group-free-prob-weight-79 | 1 | int |  |
| record | normal-group-free-prob-weight-8 | 1 | int |  |
| record | normal-group-free-prob-weight-80 | 1 | int |  |
| record | normal-group-free-prob-weight-81 | 1 | int |  |
| record | normal-group-free-prob-weight-82 | 1 | int |  |
| record | normal-group-free-prob-weight-83 | 1 | int |  |
| record | normal-group-free-prob-weight-84 | 1 | int |  |
| record | normal-group-free-prob-weight-85 | 1 | int |  |
| record | normal-group-free-prob-weight-86 | 1 | int |  |
| record | normal-group-free-prob-weight-87 | 1 | int |  |
| record | normal-group-free-prob-weight-88 | 1 | int |  |
| record | normal-group-free-prob-weight-89 | 1 | int |  |
| record | normal-group-free-prob-weight-9 | 1 | int |  |
| record | normal-group-free-prob-weight-90 | 1 | int |  |
| record | normal-group-premium-prob-weight-1 | 1 | int |  |
| record | normal-group-premium-prob-weight-10 | 1 | int |  |
| record | normal-group-premium-prob-weight-11 | 1 | int |  |
| record | normal-group-premium-prob-weight-12 | 1 | int |  |
| record | normal-group-premium-prob-weight-13 | 1 | int |  |
| record | normal-group-premium-prob-weight-14 | 1 | int |  |
| record | normal-group-premium-prob-weight-15 | 1 | int |  |
| record | normal-group-premium-prob-weight-16 | 1 | int |  |
| record | normal-group-premium-prob-weight-17 | 1 | int |  |
| record | normal-group-premium-prob-weight-18 | 1 | int |  |
| record | normal-group-premium-prob-weight-19 | 1 | int |  |
| record | normal-group-premium-prob-weight-2 | 1 | int |  |
| record | normal-group-premium-prob-weight-20 | 1 | int |  |
| record | normal-group-premium-prob-weight-21 | 1 | int |  |
| record | normal-group-premium-prob-weight-22 | 1 | int |  |
| record | normal-group-premium-prob-weight-23 | 1 | int |  |
| record | normal-group-premium-prob-weight-24 | 1 | int |  |
| record | normal-group-premium-prob-weight-25 | 1 | int |  |
| record | normal-group-premium-prob-weight-26 | 1 | int |  |
| record | normal-group-premium-prob-weight-27 | 1 | int |  |
| record | normal-group-premium-prob-weight-28 | 1 | int |  |
| record | normal-group-premium-prob-weight-29 | 1 | int |  |
| record | normal-group-premium-prob-weight-3 | 1 | int |  |
| record | normal-group-premium-prob-weight-30 | 1 | int |  |
| record | normal-group-premium-prob-weight-31 | 1 | int |  |
| record | normal-group-premium-prob-weight-32 | 1 | int |  |
| record | normal-group-premium-prob-weight-33 | 1 | int |  |
| record | normal-group-premium-prob-weight-34 | 1 | int |  |
| record | normal-group-premium-prob-weight-35 | 1 | int |  |
| record | normal-group-premium-prob-weight-36 | 1 | int |  |
| record | normal-group-premium-prob-weight-37 | 1 | int |  |
| record | normal-group-premium-prob-weight-38 | 1 | int |  |
| record | normal-group-premium-prob-weight-39 | 1 | int |  |
| record | normal-group-premium-prob-weight-4 | 1 | int |  |
| record | normal-group-premium-prob-weight-40 | 1 | int |  |
| record | normal-group-premium-prob-weight-41 | 1 | int |  |
| record | normal-group-premium-prob-weight-42 | 1 | int |  |
| record | normal-group-premium-prob-weight-43 | 1 | int |  |
| record | normal-group-premium-prob-weight-44 | 1 | int |  |
| record | normal-group-premium-prob-weight-45 | 1 | int |  |
| record | normal-group-premium-prob-weight-46 | 1 | int |  |
| record | normal-group-premium-prob-weight-47 | 1 | int |  |
| record | normal-group-premium-prob-weight-48 | 1 | int |  |
| record | normal-group-premium-prob-weight-49 | 1 | int |  |
| record | normal-group-premium-prob-weight-5 | 1 | int |  |
| record | normal-group-premium-prob-weight-50 | 1 | int |  |
| record | normal-group-premium-prob-weight-51 | 1 | int |  |
| record | normal-group-premium-prob-weight-52 | 1 | int |  |
| record | normal-group-premium-prob-weight-53 | 1 | int |  |
| record | normal-group-premium-prob-weight-54 | 1 | int |  |
| record | normal-group-premium-prob-weight-55 | 1 | int |  |
| record | normal-group-premium-prob-weight-56 | 1 | int |  |
| record | normal-group-premium-prob-weight-57 | 1 | int |  |
| record | normal-group-premium-prob-weight-58 | 1 | int |  |
| record | normal-group-premium-prob-weight-59 | 1 | int |  |
| record | normal-group-premium-prob-weight-6 | 1 | int |  |
| record | normal-group-premium-prob-weight-60 | 1 | int |  |
| record | normal-group-premium-prob-weight-61 | 1 | int |  |
| record | normal-group-premium-prob-weight-62 | 1 | int |  |
| record | normal-group-premium-prob-weight-63 | 1 | int |  |
| record | normal-group-premium-prob-weight-64 | 1 | int |  |
| record | normal-group-premium-prob-weight-65 | 1 | int |  |
| record | normal-group-premium-prob-weight-66 | 1 | int |  |
| record | normal-group-premium-prob-weight-67 | 1 | int |  |
| record | normal-group-premium-prob-weight-68 | 1 | int |  |
| record | normal-group-premium-prob-weight-69 | 1 | int |  |
| record | normal-group-premium-prob-weight-7 | 1 | int |  |
| record | normal-group-premium-prob-weight-70 | 1 | int |  |
| record | normal-group-premium-prob-weight-71 | 1 | int |  |
| record | normal-group-premium-prob-weight-72 | 1 | int |  |
| record | normal-group-premium-prob-weight-73 | 1 | int |  |
| record | normal-group-premium-prob-weight-74 | 1 | int |  |
| record | normal-group-premium-prob-weight-75 | 1 | int |  |
| record | normal-group-premium-prob-weight-76 | 1 | int |  |
| record | normal-group-premium-prob-weight-77 | 1 | int |  |
| record | normal-group-premium-prob-weight-78 | 1 | int |  |
| record | normal-group-premium-prob-weight-79 | 1 | int |  |
| record | normal-group-premium-prob-weight-8 | 1 | int |  |
| record | normal-group-premium-prob-weight-80 | 1 | int |  |
| record | normal-group-premium-prob-weight-81 | 1 | int |  |
| record | normal-group-premium-prob-weight-82 | 1 | int |  |
| record | normal-group-premium-prob-weight-83 | 1 | int |  |
| record | normal-group-premium-prob-weight-84 | 1 | int |  |
| record | normal-group-premium-prob-weight-85 | 1 | int |  |
| record | normal-group-premium-prob-weight-86 | 1 | int |  |
| record | normal-group-premium-prob-weight-87 | 1 | int |  |
| record | normal-group-premium-prob-weight-88 | 1 | int |  |
| record | normal-group-premium-prob-weight-89 | 1 | int |  |
| record | normal-group-premium-prob-weight-9 | 1 | int |  |
| record | normal-group-premium-prob-weight-90 | 1 | int |  |
| record | normal-item-group-1 | 1 | string |  |
| record | normal-item-group-10 | 1 | string |  |
| record | normal-item-group-11 | 1 | string |  |
| record | normal-item-group-12 | 1 | string |  |
| record | normal-item-group-13 | 1 | string |  |
| record | normal-item-group-14 | 1 | string |  |
| record | normal-item-group-15 | 1 | string |  |
| record | normal-item-group-16 | 1 | string |  |
| record | normal-item-group-17 | 1 | string |  |
| record | normal-item-group-18 | 1 | string |  |
| record | normal-item-group-19 | 1 | string |  |
| record | normal-item-group-2 | 1 | string |  |
| record | normal-item-group-20 | 1 | string |  |
| record | normal-item-group-21 | 1 | string |  |
| record | normal-item-group-22 | 1 | string |  |
| record | normal-item-group-23 | 1 | string |  |
| record | normal-item-group-24 | 1 | string |  |
| record | normal-item-group-25 | 1 | string |  |
| record | normal-item-group-26 | 1 | string |  |
| record | normal-item-group-27 | 1 | string |  |
| record | normal-item-group-28 | 1 | string |  |
| record | normal-item-group-29 | 1 | string |  |
| record | normal-item-group-3 | 1 | string |  |
| record | normal-item-group-30 | 1 | string |  |
| record | normal-item-group-31 | 1 | string |  |
| record | normal-item-group-32 | 1 | string |  |
| record | normal-item-group-33 | 1 | string |  |
| record | normal-item-group-34 | 1 | string |  |
| record | normal-item-group-35 | 1 | string |  |
| record | normal-item-group-36 | 1 | string |  |
| record | normal-item-group-37 | 1 | string |  |
| record | normal-item-group-38 | 1 | string |  |
| record | normal-item-group-39 | 1 | string |  |
| record | normal-item-group-4 | 1 | string |  |
| record | normal-item-group-40 | 1 | string |  |
| record | normal-item-group-41 | 1 | string |  |
| record | normal-item-group-42 | 1 | string |  |
| record | normal-item-group-43 | 1 | string |  |
| record | normal-item-group-44 | 1 | string |  |
| record | normal-item-group-45 | 1 | string |  |
| record | normal-item-group-46 | 1 | string |  |
| record | normal-item-group-47 | 1 | string |  |
| record | normal-item-group-48 | 1 | string |  |
| record | normal-item-group-49 | 1 | string |  |
| record | normal-item-group-5 | 1 | string |  |
| record | normal-item-group-50 | 1 | string |  |
| record | normal-item-group-51 | 1 | string |  |
| record | normal-item-group-52 | 1 | string |  |
| record | normal-item-group-53 | 1 | string |  |
| record | normal-item-group-54 | 1 | string |  |
| record | normal-item-group-55 | 1 | string |  |
| record | normal-item-group-56 | 1 | string |  |
| record | normal-item-group-57 | 1 | string |  |
| record | normal-item-group-58 | 1 | string |  |
| record | normal-item-group-59 | 1 | string |  |
| record | normal-item-group-6 | 1 | string |  |
| record | normal-item-group-60 | 1 | string |  |
| record | normal-item-group-61 | 1 | string |  |
| record | normal-item-group-62 | 1 | string |  |
| record | normal-item-group-63 | 1 | string |  |
| record | normal-item-group-64 | 1 | string |  |
| record | normal-item-group-65 | 1 | string |  |
| record | normal-item-group-66 | 1 | string |  |
| record | normal-item-group-67 | 1 | string |  |
| record | normal-item-group-68 | 1 | string |  |
| record | normal-item-group-69 | 1 | string |  |
| record | normal-item-group-7 | 1 | string |  |
| record | normal-item-group-70 | 1 | string |  |
| record | normal-item-group-71 | 1 | string |  |
| record | normal-item-group-72 | 1 | string |  |
| record | normal-item-group-73 | 1 | string |  |
| record | normal-item-group-74 | 1 | string |  |
| record | normal-item-group-75 | 1 | string |  |
| record | normal-item-group-76 | 1 | string |  |
| record | normal-item-group-77 | 1 | string |  |
| record | normal-item-group-78 | 1 | string |  |
| record | normal-item-group-79 | 1 | string |  |
| record | normal-item-group-8 | 1 | string |  |
| record | normal-item-group-80 | 1 | string |  |
| record | normal-item-group-81 | 1 | string |  |
| record | normal-item-group-82 | 1 | string |  |
| record | normal-item-group-83 | 1 | string |  |
| record | normal-item-group-84 | 1 | string |  |
| record | normal-item-group-85 | 1 | string |  |
| record | normal-item-group-86 | 1 | string |  |
| record | normal-item-group-87 | 1 | string |  |
| record | normal-item-group-88 | 1 | string |  |
| record | normal-item-group-89 | 1 | string |  |
| record | normal-item-group-9 | 1 | string |  |
| record | normal-item-group-90 | 1 | string |  |
| record | premium-draw-reward-group-prob-weight-1 | 1 | int |  |
| record | premium-draw-reward-group-prob-weight-10 | 1 | int |  |
| record | premium-draw-reward-group-prob-weight-11 | 1 | int |  |
| record | premium-draw-reward-group-prob-weight-12 | 1 | int |  |
| record | premium-draw-reward-group-prob-weight-13 | 1 | int |  |
| record | premium-draw-reward-group-prob-weight-2 | 1 | int |  |
| record | premium-draw-reward-group-prob-weight-3 | 1 | int |  |
| record | premium-draw-reward-group-prob-weight-4 | 1 | int |  |
| record | premium-draw-reward-group-prob-weight-5 | 1 | int |  |
| record | premium-draw-reward-group-prob-weight-6 | 1 | int |  |
| record | premium-draw-reward-group-prob-weight-7 | 1 | int |  |
| record | premium-draw-reward-group-prob-weight-8 | 1 | int |  |
| record | premium-draw-reward-group-prob-weight-9 | 1 | int |  |
| record | premium-draw-reward-item-group-1 | 1 | string |  |
| record | premium-draw-reward-item-group-10 | 1 | string |  |
| record | premium-draw-reward-item-group-11 | 1 | string |  |
| record | premium-draw-reward-item-group-12 | 1 | string |  |
| record | premium-draw-reward-item-group-13 | 1 | string |  |
| record | premium-draw-reward-item-group-2 | 1 | string |  |
| record | premium-draw-reward-item-group-3 | 1 | string |  |
| record | premium-draw-reward-item-group-4 | 1 | string |  |
| record | premium-draw-reward-item-group-5 | 1 | string |  |
| record | premium-draw-reward-item-group-6 | 1 | string |  |
| record | premium-draw-reward-item-group-7 | 1 | string |  |
| record | premium-draw-reward-item-group-8 | 1 | string |  |
| record | premium-draw-reward-item-group-9 | 1 | string |  |
| record | rare-group-free-prob-1 | 1 | int |  |
| record | rare-group-free-prob-10 | 1 | int |  |
| record | rare-group-free-prob-11 | 1 | int |  |
| record | rare-group-free-prob-12 | 1 | int |  |
| record | rare-group-free-prob-13 | 1 | int |  |
| record | rare-group-free-prob-14 | 1 | int |  |
| record | rare-group-free-prob-15 | 1 | int |  |
| record | rare-group-free-prob-16 | 1 | int |  |
| record | rare-group-free-prob-17 | 1 | int |  |
| record | rare-group-free-prob-18 | 1 | int |  |
| record | rare-group-free-prob-19 | 1 | int |  |
| record | rare-group-free-prob-2 | 1 | int |  |
| record | rare-group-free-prob-3 | 1 | int |  |
| record | rare-group-free-prob-4 | 1 | int |  |
| record | rare-group-free-prob-5 | 1 | int |  |
| record | rare-group-free-prob-6 | 1 | int |  |
| record | rare-group-free-prob-7 | 1 | int |  |
| record | rare-group-free-prob-8 | 1 | int |  |
| record | rare-group-free-prob-9 | 1 | int |  |
| record | rare-group-free-redice-count-1 | 1 | int |  |
| record | rare-group-free-redice-count-10 | 1 | int |  |
| record | rare-group-free-redice-count-11 | 1 | int |  |
| record | rare-group-free-redice-count-12 | 1 | int |  |
| record | rare-group-free-redice-count-13 | 1 | int |  |
| record | rare-group-free-redice-count-14 | 1 | int |  |
| record | rare-group-free-redice-count-15 | 1 | int |  |
| record | rare-group-free-redice-count-16 | 1 | int |  |
| record | rare-group-free-redice-count-17 | 1 | int |  |
| record | rare-group-free-redice-count-18 | 1 | int |  |
| record | rare-group-free-redice-count-19 | 1 | int |  |
| record | rare-group-free-redice-count-2 | 1 | int |  |
| record | rare-group-free-redice-count-3 | 1 | int |  |
| record | rare-group-free-redice-count-4 | 1 | int |  |
| record | rare-group-free-redice-count-5 | 1 | int |  |
| record | rare-group-free-redice-count-6 | 1 | int |  |
| record | rare-group-free-redice-count-7 | 1 | int |  |
| record | rare-group-free-redice-count-8 | 1 | int |  |
| record | rare-group-free-redice-count-9 | 1 | int |  |
| record | rare-group-premium-prob-1 | 1 | int |  |
| record | rare-group-premium-prob-10 | 1 | int |  |
| record | rare-group-premium-prob-11 | 1 | int |  |
| record | rare-group-premium-prob-12 | 1 | int |  |
| record | rare-group-premium-prob-13 | 1 | int |  |
| record | rare-group-premium-prob-14 | 1 | int |  |
| record | rare-group-premium-prob-15 | 1 | int |  |
| record | rare-group-premium-prob-16 | 1 | int |  |
| record | rare-group-premium-prob-17 | 1 | int |  |
| record | rare-group-premium-prob-18 | 1 | int |  |
| record | rare-group-premium-prob-19 | 1 | int |  |
| record | rare-group-premium-prob-2 | 1 | int |  |
| record | rare-group-premium-prob-3 | 1 | int |  |
| record | rare-group-premium-prob-4 | 1 | int |  |
| record | rare-group-premium-prob-5 | 1 | int |  |
| record | rare-group-premium-prob-6 | 1 | int |  |
| record | rare-group-premium-prob-7 | 1 | int |  |
| record | rare-group-premium-prob-8 | 1 | int |  |
| record | rare-group-premium-prob-9 | 1 | int |  |
| record | rare-group-premium-redice-count-1 | 1 | int |  |
| record | rare-group-premium-redice-count-10 | 1 | int |  |
| record | rare-group-premium-redice-count-11 | 1 | int |  |
| record | rare-group-premium-redice-count-12 | 1 | int |  |
| record | rare-group-premium-redice-count-13 | 1 | int |  |
| record | rare-group-premium-redice-count-14 | 1 | int |  |
| record | rare-group-premium-redice-count-15 | 1 | int |  |
| record | rare-group-premium-redice-count-16 | 1 | int |  |
| record | rare-group-premium-redice-count-17 | 1 | int |  |
| record | rare-group-premium-redice-count-18 | 1 | int |  |
| record | rare-group-premium-redice-count-19 | 1 | int |  |
| record | rare-group-premium-redice-count-2 | 1 | int |  |
| record | rare-group-premium-redice-count-3 | 1 | int |  |
| record | rare-group-premium-redice-count-4 | 1 | int |  |
| record | rare-group-premium-redice-count-5 | 1 | int |  |
| record | rare-group-premium-redice-count-6 | 1 | int |  |
| record | rare-group-premium-redice-count-7 | 1 | int |  |
| record | rare-group-premium-redice-count-8 | 1 | int |  |
| record | rare-group-premium-redice-count-9 | 1 | int |  |
| record | rare-item-group-1 | 1 | string |  |
| record | rare-item-group-10 | 1 | string |  |
| record | rare-item-group-11 | 1 | string |  |
| record | rare-item-group-12 | 1 | string |  |
| record | rare-item-group-13 | 1 | string |  |
| record | rare-item-group-14 | 1 | string |  |
| record | rare-item-group-15 | 1 | string |  |
| record | rare-item-group-16 | 1 | string |  |
| record | rare-item-group-17 | 1 | string |  |
| record | rare-item-group-18 | 1 | string |  |
| record | rare-item-group-19 | 1 | string |  |
| record | rare-item-group-2 | 1 | string |  |
| record | rare-item-group-3 | 1 | string |  |
| record | rare-item-group-4 | 1 | string |  |
| record | rare-item-group-5 | 1 | string |  |
| record | rare-item-group-6 | 1 | string |  |
| record | rare-item-group-7 | 1 | string |  |
| record | rare-item-group-8 | 1 | string |  |
| record | rare-item-group-9 | 1 | string |  |

### reward

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 123409 | string |  |
| record | fixed-item-1 | 75412 | string |  |
| record | fixed-item-2 | 27753 | string |  |
| record | fixed-item-3 | 9422 | string |  |
| record | fixed-item-4 | 7011 | string |  |
| record | fixed-item-5 | 3860 | string |  |
| record | fixed-item-6 | 1785 | string |  |
| record | fixed-item-7 | 1169 | string |  |
| record | fixed-item-8 | 472 | string |  |
| record | fixed-item-max-1 | 75449 | int |  |
| record | fixed-item-max-2 | 27704 | int |  |
| record | fixed-item-max-3 | 9450 | int |  |
| record | fixed-item-max-4 | 7033 | int |  |
| record | fixed-item-max-5 | 3881 | int |  |
| record | fixed-item-max-6 | 1809 | int |  |
| record | fixed-item-max-7 | 1193 | int |  |
| record | fixed-item-max-8 | 472 | int |  |
| record | fixed-item-min-1 | 75419 | int |  |
| record | fixed-item-min-2 | 27746 | int |  |
| record | fixed-item-min-3 | 9444 | int |  |
| record | fixed-item-min-4 | 7033 | int |  |
| record | fixed-item-min-5 | 3884 | int |  |
| record | fixed-item-min-6 | 1809 | int |  |
| record | fixed-item-min-7 | 1193 | int |  |
| record | fixed-item-min-8 | 472 | int |  |
| record | group-1-2-probability | 4623 | int |  |
| record | group-1-assured-count | 4620 | int |  |
| record | group-1-item-1 | 4625 | string |  |
| record | group-1-item-10 | 26 | string |  |
| record | group-1-item-11 | 24 | string |  |
| record | group-1-item-12 | 23 | string |  |
| record | group-1-item-2 | 4215 | string |  |
| record | group-1-item-3 | 938 | string |  |
| record | group-1-item-4 | 699 | string |  |
| record | group-1-item-5 | 627 | string |  |
| record | group-1-item-6 | 564 | string |  |
| record | group-1-item-7 | 554 | string |  |
| record | group-1-item-8 | 546 | string |  |
| record | group-1-item-9 | 26 | string |  |
| record | group-1-prob-weight | 4625 | int |  |
| record | group-2-assured-count | 1026 | int |  |
| record | group-2-item-1 | 1026 | string |  |
| record | group-2-item-10 | 23 | string |  |
| record | group-2-item-11 | 23 | string |  |
| record | group-2-item-12 | 22 | string |  |
| record | group-2-item-2 | 898 | string |  |
| record | group-2-item-3 | 294 | string |  |
| record | group-2-item-4 | 177 | string |  |
| record | group-2-item-5 | 155 | string |  |
| record | group-2-item-6 | 106 | string |  |
| record | group-2-item-7 | 100 | string |  |
| record | group-2-item-8 | 99 | string |  |
| record | group-2-item-9 | 23 | string |  |
| record | group-2-prob-weight | 1027 | int |  |
| record | group-3-item-1 | 8341 | string |  |
| record | group-3-item-10 | 688 | string |  |
| record | group-3-item-11 | 675 | string |  |
| record | group-3-item-12 | 660 | string |  |
| record | group-3-item-13 | 509 | string |  |
| record | group-3-item-14 | 506 | string |  |
| record | group-3-item-15 | 503 | string |  |
| record | group-3-item-16 | 439 | string |  |
| record | group-3-item-17 | 437 | string |  |
| record | group-3-item-18 | 430 | string |  |
| record | group-3-item-19 | 122 | string |  |
| record | group-3-item-2 | 4274 | string |  |
| record | group-3-item-20 | 122 | string |  |
| record | group-3-item-21 | 18 | string |  |
| record | group-3-item-22 | 18 | string |  |
| record | group-3-item-23 | 18 | string |  |
| record | group-3-item-24 | 18 | string |  |
| record | group-3-item-3 | 3701 | string |  |
| record | group-3-item-4 | 3365 | string |  |
| record | group-3-item-5 | 3164 | string |  |
| record | group-3-item-6 | 2651 | string |  |
| record | group-3-item-7 | 2348 | string |  |
| record | group-3-item-8 | 1347 | string |  |
| record | group-3-item-9 | 1023 | string |  |
| record | group-3-item-prob-weight-1 | 8357 | int |  |
| record | group-3-item-prob-weight-10 | 692 | int |  |
| record | group-3-item-prob-weight-11 | 674 | int |  |
| record | group-3-item-prob-weight-12 | 657 | int |  |
| record | group-3-item-prob-weight-13 | 502 | int |  |
| record | group-3-item-prob-weight-14 | 499 | int |  |
| record | group-3-item-prob-weight-15 | 496 | int |  |
| record | group-3-item-prob-weight-16 | 432 | int |  |
| record | group-3-item-prob-weight-17 | 430 | int |  |
| record | group-3-item-prob-weight-18 | 423 | int |  |
| record | group-3-item-prob-weight-19 | 122 | int |  |
| record | group-3-item-prob-weight-2 | 4267 | int |  |
| record | group-3-item-prob-weight-20 | 122 | int |  |
| record | group-3-item-prob-weight-21 | 18 | int |  |
| record | group-3-item-prob-weight-22 | 18 | int |  |
| record | group-3-item-prob-weight-23 | 18 | int |  |
| record | group-3-item-prob-weight-24 | 18 | int |  |
| record | group-3-item-prob-weight-3 | 3701 | int |  |
| record | group-3-item-prob-weight-4 | 3375 | int |  |
| record | group-3-item-prob-weight-5 | 3165 | int |  |
| record | group-3-item-prob-weight-6 | 2650 | int |  |
| record | group-3-item-prob-weight-7 | 2347 | int |  |
| record | group-3-item-prob-weight-8 | 1346 | int |  |
| record | group-3-item-prob-weight-9 | 1022 | int |  |
| record | group-3-probability | 8352 | int |  |
| record | group-4-item-1 | 181 | string |  |
| record | group-4-item-2 | 117 | string |  |
| record | group-4-item-3 | 91 | string |  |
| record | group-4-item-4 | 81 | string |  |
| record | group-4-item-5 | 15 | string |  |
| record | group-4-item-6 | 10 | string |  |
| record | group-4-item-max-1 | 181 | int |  |
| record | group-4-item-max-2 | 117 | int |  |
| record | group-4-item-max-3 | 91 | int |  |
| record | group-4-item-max-4 | 81 | int |  |
| record | group-4-item-max-5 | 15 | int |  |
| record | group-4-item-max-6 | 10 | int |  |
| record | group-4-item-min-1 | 181 | int |  |
| record | group-4-item-min-2 | 117 | int |  |
| record | group-4-item-min-3 | 91 | int |  |
| record | group-4-item-min-4 | 81 | int |  |
| record | group-4-item-min-5 | 15 | int |  |
| record | group-4-item-min-6 | 10 | int |  |
| record | group-4-probability | 181 | int |  |
| record | group-4-selected-count | 181 | int |  |
| record | group-5-item-1 | 61 | string |  |
| record | group-5-item-2 | 33 | string |  |
| record | group-5-item-3 | 15 | string |  |
| record | group-5-item-4 | 11 | string |  |
| record | group-5-item-max-1 | 61 | int |  |
| record | group-5-item-max-2 | 33 | int |  |
| record | group-5-item-max-3 | 15 | int |  |
| record | group-5-item-max-4 | 11 | int |  |
| record | group-5-item-min-1 | 61 | int |  |
| record | group-5-item-min-2 | 33 | int |  |
| record | group-5-item-min-3 | 15 | int |  |
| record | group-5-item-min-4 | 11 | int |  |
| record | group-5-probability | 61 | int |  |
| record | group-5-selected-count | 61 | int |  |
| record | id | 123409 | int |  |
| record | money-max | 3797 | int |  |
| record | money-min | 3797 | int |  |
| record | money-order-of-magnitude | 9 | int |  |
| record | money-probability | 3856 | int |  |
| record | rare-item-1 | 54942 | string |  |
| record | rare-item-1-max | 54950 | int |  |
| record | rare-item-1-min | 54950 | int |  |
| record | rare-item-10 | 215 | string |  |
| record | rare-item-11 | 172 | string |  |
| record | rare-item-12 | 132 | string |  |
| record | rare-item-13 | 110 | string |  |
| record | rare-item-14 | 87 | string |  |
| record | rare-item-15 | 66 | string |  |
| record | rare-item-16 | 47 | string |  |
| record | rare-item-17 | 2 | string |  |
| record | rare-item-2 | 15163 | string |  |
| record | rare-item-3 | 3429 | string |  |
| record | rare-item-4 | 3049 | string |  |
| record | rare-item-5 | 2611 | string |  |
| record | rare-item-6 | 1961 | string |  |
| record | rare-item-7 | 1299 | string |  |
| record | rare-item-8 | 896 | string |  |
| record | rare-item-9 | 420 | string |  |
| record | rare-item-prob-weight-1 | 54944 | int |  |
| record | rare-item-prob-weight-10 | 215 | int |  |
| record | rare-item-prob-weight-11 | 172 | int |  |
| record | rare-item-prob-weight-12 | 132 | int |  |
| record | rare-item-prob-weight-13 | 110 | int |  |
| record | rare-item-prob-weight-14 | 87 | int |  |
| record | rare-item-prob-weight-15 | 66 | int |  |
| record | rare-item-prob-weight-16 | 47 | int |  |
| record | rare-item-prob-weight-17 | 2 | int |  |
| record | rare-item-prob-weight-2 | 15164 | int |  |
| record | rare-item-prob-weight-3 | 3437 | int |  |
| record | rare-item-prob-weight-4 | 3057 | int |  |
| record | rare-item-prob-weight-5 | 2610 | int |  |
| record | rare-item-prob-weight-6 | 1961 | int |  |
| record | rare-item-prob-weight-7 | 1300 | int |  |
| record | rare-item-prob-weight-8 | 896 | int |  |
| record | rare-item-prob-weight-9 | 420 | int |  |
| record | selected-item-1 | 188 | string |  |
| record | selected-item-10 | 2 | string |  |
| record | selected-item-11 | 2 | string |  |
| record | selected-item-12 | 2 | string |  |
| record | selected-item-13 | 1 | string |  |
| record | selected-item-14 | 1 | string |  |
| record | selected-item-15 | 1 | string |  |
| record | selected-item-16 | 1 | string |  |
| record | selected-item-17 | 1 | string |  |
| record | selected-item-18 | 1 | string |  |
| record | selected-item-19 | 1 | string |  |
| record | selected-item-2 | 188 | string |  |
| record | selected-item-20 | 1 | string |  |
| record | selected-item-3 | 122 | string |  |
| record | selected-item-4 | 75 | string |  |
| record | selected-item-5 | 62 | string |  |
| record | selected-item-6 | 50 | string |  |
| record | selected-item-7 | 29 | string |  |
| record | selected-item-8 | 27 | string |  |
| record | selected-item-9 | 18 | string |  |
| record | selected-item-assured-count | 188 | int |  |
| record | selected-item-count-1 | 188 | int |  |
| record | selected-item-count-10 | 2 | int |  |
| record | selected-item-count-11 | 2 | int |  |
| record | selected-item-count-12 | 2 | int |  |
| record | selected-item-count-13 | 1 | int |  |
| record | selected-item-count-14 | 1 | int |  |
| record | selected-item-count-15 | 1 | int |  |
| record | selected-item-count-16 | 1 | int |  |
| record | selected-item-count-17 | 1 | int |  |
| record | selected-item-count-18 | 1 | int |  |
| record | selected-item-count-19 | 1 | int |  |
| record | selected-item-count-2 | 188 | int |  |
| record | selected-item-count-20 | 1 | int |  |
| record | selected-item-count-3 | 122 | int |  |
| record | selected-item-count-4 | 75 | int |  |
| record | selected-item-count-5 | 62 | int |  |
| record | selected-item-count-6 | 50 | int |  |
| record | selected-item-count-7 | 29 | int |  |
| record | selected-item-count-8 | 27 | int |  |
| record | selected-item-count-9 | 18 | int |  |

### script

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| Script | alias | 6 | string |  |
| Script/decision | event | 5 | enum-like string ❓ | npc-spawn |
| Script/decision/filter-set/filter | spawn | 5 | string |  |
| Script/decision/filter-set/filter | subject | 5 | string |  |
| Script/decision/filter-set/filter | type | 5 | enum-like string ❓ | npc-spawn |
| Script/decision/reaction-set/reaction | seq | 5 | string |  |
| Script/decision/reaction-set/reaction | target | 5 | string |  |
| Script/decision/reaction-set/reaction | text | 5 | string |  |
| Script/decision/reaction-set/reaction | type | 10 | enum-like string ❓ | debug-trace, set-npc-act |
| Script/quest-decision | alias | 10 | string |  |
| Script/quest-decision/reaction-set/reaction | attack-target | 5 | string |  |
| Script/quest-decision/reaction-set/reaction | spawn-1 | 10 | string |  |
| Script/quest-decision/reaction-set/reaction | text | 10 | string |  |
| Script/quest-decision/reaction-set/reaction | type | 20 | enum-like string ❓ | debug-trace, spawn-npc |
| decision | event | 3 | enum-like string ❓ | act-finished, attacked, npc-special |
| decision/filter-set/filter | op | 1 | enum-like string ❓ | eq |
| decision/filter-set/filter | subject | 5 | string |  |
| decision/filter-set/filter | subject2 | 3 | string |  |
| decision/filter-set/filter | type | 5 | enum-like string ❓ | act-sequence, event-broadcasted, number, pc |
| decision/filter-set/filter | value | 4 | string |  |
| decision/reaction-set/reaction | amount | 10 | int |  |
| decision/reaction-set/reaction | attackable | 1 | enum-like string ❓ | y |
| decision/reaction-set/reaction | brain | 1 | enum-like string ❓ | boss |
| decision/reaction-set/reaction | effect | 2 | string |  |
| decision/reaction-set/reaction | hate-on | 1 | enum-like string ❓ | y |
| decision/reaction-set/reaction | npc | 1 | string |  |
| decision/reaction-set/reaction | opponent | 11 | string |  |
| decision/reaction-set/reaction | seq | 1 | string |  |
| decision/reaction-set/reaction | target | 14 | string |  |
| decision/reaction-set/reaction | text | 3 | string |  |
| decision/reaction-set/reaction | type | 18 | enum-like string ❓ | debug-trace, diff-npc-hate, invoke-effect, reset-npc-hate, set-npc-act, set-npc-brain |
| script | alias | 17257 | string |  |
| script | hanger | 46 | enum-like string ❓ | object |
| script | hide-seq-start | 1 | enum-like string ❓ | y |
| script | parent | 5230 | string |  |
| script/decision | event | 56382 | string |  |
| script/decision | forward | 9623 | enum-like string ❓ | y |
| script/decision | foward | 167 | enum-like string ❓ | n, y |
| script/decision | fowrard | 6 | enum-like string ❓ | y |
| script/decision/decision | event | 8 | enum-like string ❓ | attack, invoked-effect, killed, npc-combat-end |
| script/decision/decision | forward | 4 | enum-like string ❓ | y |
| script/decision/decision/filter-set/filter | field | 1 | enum-like string ❓ | dead |
| script/decision/decision/filter-set/filter | op | 4 | enum-like string ❓ | eq, neq |
| script/decision/decision/filter-set/filter | subject | 8 | string |  |
| script/decision/decision/filter-set/filter | subject2 | 8 | string |  |
| script/decision/decision/filter-set/filter | type | 8 | enum-like string ❓ | effect-id, event-broadcasted, prop |
| script/decision/decision/filter-set/filter | value | 8 | string |  |
| script/decision/decision/reaction-set | probability | 1 | int |  |
| script/decision/decision/reaction-set/reaction | amount | 1 | int |  |
| script/decision/decision/reaction-set/reaction | effect | 8 | string |  |
| script/decision/decision/reaction-set/reaction | from | 1 | string |  |
| script/decision/decision/reaction-set/reaction | reg | 1 | int |  |
| script/decision/decision/reaction-set/reaction | social | 1 | string |  |
| script/decision/decision/reaction-set/reaction | target | 9 | string |  |
| script/decision/decision/reaction-set/reaction | text | 3 | string |  |
| script/decision/decision/reaction-set/reaction | type | 13 | enum-like string ❓ | debug-trace, invoke-effect, play-social, set-npc-number |
| script/decision/filter | op | 2 | enum-like string ❓ | eq, ge |
| script/decision/filter | subject | 2 | string |  |
| script/decision/filter | subject2 | 2 | string |  |
| script/decision/filter | type | 2 | enum-like string ❓ | number |
| script/decision/filter | value | 2 | int |  |
| script/decision/filter-set/filter | amount | 1 | int |  |
| script/decision/filter-set/filter | effect-slot | 19 | enum-like string ❓ | buff |
| script/decision/filter-set/filter | effect-type | 19 | string |  |
| script/decision/filter-set/filter | either | 874 | enum-like string ❓ | n, y |
| script/decision/filter-set/filter | env | 40 | string |  |
| script/decision/filter-set/filter | env-state | 207 | string |  |
| script/decision/filter-set/filter | env2-state | 348 | string |  |
| script/decision/filter-set/filter | env2spawn | 40 | string |  |
| script/decision/filter-set/filter | field | 4287 | enum-like string ❓ | combat-mode, dead, gp, hp, x, y |
| script/decision/filter-set/filter | flag | 2066 | enum-like string ❓ | n, y |
| script/decision/filter-set/filter | flagtype | 1989 | string |  |
| script/decision/filter-set/filter | index | 191 | int |  |
| script/decision/filter-set/filter | item | 1 | string |  |
| script/decision/filter-set/filter | leader | 237 | enum-like string ❓ | n, y |
| script/decision/filter-set/filter | op | 42967 | enum-like string ❓ | ep, eq, ge, gt, le, lt, neq |
| script/decision/filter-set/filter | party | 672 | string |  |
| script/decision/filter-set/filter | quest | 3 | string |  |
| script/decision/filter-set/filter | spawn | 10583 | string |  |
| script/decision/filter-set/filter | subject | 97932 | string |  |
| script/decision/filter-set/filter | subject1 | 14 | string |  |
| script/decision/filter-set/filter | subject2 | 102152 | string |  |
| script/decision/filter-set/filter | subjetct2 | 1 | string |  |
| script/decision/filter-set/filter | target | 40 | string |  |
| script/decision/filter-set/filter | target2 | 70 | string |  |
| script/decision/filter-set/filter | type | 102200 | string |  |
| script/decision/filter-set/filter | value | 82809 | string |  |
| script/decision/filter-set/filter | value-1 | 1193 | string |  |
| script/decision/filter-set/filter | value-2 | 87 | string |  |
| script/decision/filter-set/filter | value-3 | 12 | string |  |
| script/decision/filter-set/reaction | amount | 14 | int |  |
| script/decision/filter-set/reaction | despawn-force | 2 | enum-like string ❓ | y |
| script/decision/filter-set/reaction | from | 1 | string |  |
| script/decision/filter-set/reaction | reg | 14 | int |  |
| script/decision/filter-set/reaction | respawn-after-despawn | 2 | enum-like string ❓ | n |
| script/decision/filter-set/reaction | social | 1 | string |  |
| script/decision/filter-set/reaction | target | 14 | string |  |
| script/decision/filter-set/reaction | target-1 | 2 | string |  |
| script/decision/filter-set/reaction | target-2 | 2 | string |  |
| script/decision/filter-set/reaction | text | 1 | string |  |
| script/decision/filter-set/reaction | type | 18 | enum-like string ❓ | debug-trace, despawn-npc, diff-party-number, play-social, set-npc-number |
| script/decision/filter/filter | subject | 8 | string |  |
| script/decision/filter/filter | subject2 | 8 | string |  |
| script/decision/filter/filter | type | 8 | enum-like string ❓ | event-broadcasted, pc, summoned |
| script/decision/filter/filter | value | 4 | enum-like string ❓ | n |
| script/decision/reaction | attackable | 1 | enum-like string ❓ | y |
| script/decision/reaction | brain | 1 | enum-like string ❓ | monster |
| script/decision/reaction | effect | 3 | string |  |
| script/decision/reaction | effect-type | 2 | string |  |
| script/decision/reaction | enable | 1 | enum-like string ❓ | n |
| script/decision/reaction | from | 6 | string |  |
| script/decision/reaction | hate-on | 1 | enum-like string ❓ | y |
| script/decision/reaction | npc | 1 | string |  |
| script/decision/reaction | percent | 1 | int |  |
| script/decision/reaction | seq | 1 | string |  |
| script/decision/reaction | social | 6 | string |  |
| script/decision/reaction | spawn-1 | 9 | string |  |
| script/decision/reaction | spawn-delay | 9 | int |  |
| script/decision/reaction | special-id | 1 | int |  |
| script/decision/reaction | target | 9 | string |  |
| script/decision/reaction | target2 | 2 | string |  |
| script/decision/reaction | text | 6 | string |  |
| script/decision/reaction | type | 31 | enum-like string ❓ | damage, debug-trace, dispel-by-type, invoke-effect, npc-fire-special, play-social, set-env-enable, set-npc-act, set-npc-brain, spawn-npc |
| script/decision/reaction-set | probability | 6575 | int |  |
| script/decision/reaction-set | probability10000 | 9 | int |  |
| script/decision/reaction-set/reaciton | from | 2 | string |  |
| script/decision/reaction-set/reaciton | social | 2 | string |  |
| script/decision/reaction-set/reaciton | type | 2 | enum-like string ❓ | play-social |
| script/decision/reaction-set/reaction | amount | 51287 | int |  |
| script/decision/reaction-set/reaction | area | 43 | string |  |
| script/decision/reaction-set/reaction | attack-taret | 2 | string |  |
| script/decision/reaction-set/reaction | attack-target | 7194 | string |  |
| script/decision/reaction-set/reaction | attackable | 4130 | enum-like string ❓ | n, y |
| script/decision/reaction-set/reaction | attck-target | 1 | string |  |
| script/decision/reaction-set/reaction | attr | 12 | string |  |
| script/decision/reaction-set/reaction | brain | 4209 | string |  |
| script/decision/reaction-set/reaction | caster | 2105 | string |  |
| script/decision/reaction-set/reaction | cinematic | 124 | string |  |
| script/decision/reaction-set/reaction | combat-mode | 249 | enum-like string ❓ | n, y |
| script/decision/reaction-set/reaction | despawn-delay | 27 | int |  |
| script/decision/reaction-set/reaction | despawn-force | 18162 | enum-like string ❓ | n, y |
| script/decision/reaction-set/reaction | despawn-social | 4 | string |  |
| script/decision/reaction-set/reaction | despawn-social-1 | 1312 | string |  |
| script/decision/reaction-set/reaction | despawn-social-10 | 8 | string |  |
| script/decision/reaction-set/reaction | despawn-social-2 | 41 | string |  |
| script/decision/reaction-set/reaction | despawn-social-3 | 31 | string |  |
| script/decision/reaction-set/reaction | despawn-social-4 | 13 | string |  |
| script/decision/reaction-set/reaction | despawn-social-5 | 9 | string |  |
| script/decision/reaction-set/reaction | despawn-social-6 | 8 | string |  |
| script/decision/reaction-set/reaction | despawn-social-7 | 8 | string |  |
| script/decision/reaction-set/reaction | despawn-social-8 | 8 | string |  |
| script/decision/reaction-set/reaction | despawn-social-9 | 8 | string |  |
| script/decision/reaction-set/reaction | despawww3n-force | 1 | enum-like string ❓ | n |
| script/decision/reaction-set/reaction | detect | 4 | string |  |
| script/decision/reaction-set/reaction | dispel-force | 102 | enum-like string ❓ | y |
| script/decision/reaction-set/reaction | dspawn-force | 12 | enum-like string ❓ | y |
| script/decision/reaction-set/reaction | duration | 208 | int |  |
| script/decision/reaction-set/reaction | effect | 23238 | string |  |
| script/decision/reaction-set/reaction | effect-type | 2040 | string |  |
| script/decision/reaction-set/reaction | enable | 10569 | enum-like string ❓ | n, y |
| script/decision/reaction-set/reaction | end | 3007 | int |  |
| script/decision/reaction-set/reaction | enter-cinematic | 2 | string |  |
| script/decision/reaction-set/reaction | field-item | 35 | string |  |
| script/decision/reaction-set/reaction | filter-1 | 208 | string |  |
| script/decision/reaction-set/reaction | filter-2 | 61 | string |  |
| script/decision/reaction-set/reaction | filter-3 | 15 | string |  |
| script/decision/reaction-set/reaction | flag | 1728 | enum-like string ❓ | n, y |
| script/decision/reaction-set/reaction | from | 9156 | string |  |
| script/decision/reaction-set/reaction | gather-count | 208 | int |  |
| script/decision/reaction-set/reaction | group | 10378 | string |  |
| script/decision/reaction-set/reaction | group-1 | 6141 | string |  |
| script/decision/reaction-set/reaction | group-10 | 52 | string |  |
| script/decision/reaction-set/reaction | group-2 | 1241 | string |  |
| script/decision/reaction-set/reaction | group-3 | 866 | string |  |
| script/decision/reaction-set/reaction | group-4 | 634 | string |  |
| script/decision/reaction-set/reaction | group-5 | 426 | string |  |
| script/decision/reaction-set/reaction | group-6 | 335 | string |  |
| script/decision/reaction-set/reaction | group-7 | 181 | string |  |
| script/decision/reaction-set/reaction | group-8 | 138 | string |  |
| script/decision/reaction-set/reaction | group-9 | 85 | string |  |
| script/decision/reaction-set/reaction | hate-on | 4130 | enum-like string ❓ | n, y |
| script/decision/reaction-set/reaction | immediately | 2907 | enum-like string ❓ | n, y |
| script/decision/reaction-set/reaction | index | 2532 | int |  |
| script/decision/reaction-set/reaction | index-1 | 7366 | string |  |
| script/decision/reaction-set/reaction | index-10 | 1017 | int |  |
| script/decision/reaction-set/reaction | index-11 | 3 | int |  |
| script/decision/reaction-set/reaction | index-12 | 3 | int |  |
| script/decision/reaction-set/reaction | index-13 | 3 | int |  |
| script/decision/reaction-set/reaction | index-14 | 3 | int |  |
| script/decision/reaction-set/reaction | index-15 | 3 | int |  |
| script/decision/reaction-set/reaction | index-2 | 3845 | int |  |
| script/decision/reaction-set/reaction | index-3 | 2595 | int |  |
| script/decision/reaction-set/reaction | index-4 | 1974 | int |  |
| script/decision/reaction-set/reaction | index-5 | 1565 | int |  |
| script/decision/reaction-set/reaction | index-6 | 1343 | int |  |
| script/decision/reaction-set/reaction | index-7 | 1190 | int |  |
| script/decision/reaction-set/reaction | index-8 | 1144 | int |  |
| script/decision/reaction-set/reaction | index-9 | 1065 | int |  |
| script/decision/reaction-set/reaction | init-enable | 112 | enum-like string ❓ | n, y |
| script/decision/reaction-set/reaction | invoker | 24 | string |  |
| script/decision/reaction-set/reaction | leave-cinematic | 6 | string |  |
| script/decision/reaction-set/reaction | master | 5 | string |  |
| script/decision/reaction-set/reaction | max | 3007 | int |  |
| script/decision/reaction-set/reaction | min | 3007 | int |  |
| script/decision/reaction-set/reaction | npc | 4069 | string |  |
| script/decision/reaction-set/reaction | object | 2997 | string |  |
| script/decision/reaction-set/reaction | opponent | 22749 | string |  |
| script/decision/reaction-set/reaction | param-1 | 2126 | string |  |
| script/decision/reaction-set/reaction | param-2 | 92 | string |  |
| script/decision/reaction-set/reaction | param-3 | 33 | string |  |
| script/decision/reaction-set/reaction | param-4 | 23 | string |  |
| script/decision/reaction-set/reaction | param-5 | 20 | string |  |
| script/decision/reaction-set/reaction | param-6 | 20 | string |  |
| script/decision/reaction-set/reaction | party | 4 | string |  |
| script/decision/reaction-set/reaction | pc-spawn | 93 | int |  |
| script/decision/reaction-set/reaction | percent | 573 | int |  |
| script/decision/reaction-set/reaction | phase-zone-enter-cinematic-1 | 1 | string |  |
| script/decision/reaction-set/reaction | phase-zone-leave-cinematic-1 | 3 | string |  |
| script/decision/reaction-set/reaction | phase-zone-pc-spawn-1 | 14 | int |  |
| script/decision/reaction-set/reaction | phase-zone-pc-spawn-2 | 4 | int |  |
| script/decision/reaction-set/reaction | phase-zone-pc-spawn-3 | 3 | int |  |
| script/decision/reaction-set/reaction | phase-zone-pc-spawn-4 | 1 | int |  |
| script/decision/reaction-set/reaction | phase-zone-pc-spawn-8 | 1 | int |  |
| script/decision/reaction-set/reaction | prob-1 | 11 | int |  |
| script/decision/reaction-set/reaction | prob-10 | 8 | int |  |
| script/decision/reaction-set/reaction | prob-2 | 11 | int |  |
| script/decision/reaction-set/reaction | prob-3 | 11 | int |  |
| script/decision/reaction-set/reaction | prob-4 | 8 | int |  |
| script/decision/reaction-set/reaction | prob-5 | 8 | int |  |
| script/decision/reaction-set/reaction | prob-6 | 8 | int |  |
| script/decision/reaction-set/reaction | prob-7 | 8 | int |  |
| script/decision/reaction-set/reaction | prob-8 | 8 | int |  |
| script/decision/reaction-set/reaction | prob-9 | 8 | int |  |
| script/decision/reaction-set/reaction | probability | 132 | int |  |
| script/decision/reaction-set/reaction | public-raid-event | 2 | string |  |
| script/decision/reaction-set/reaction | radius | 208 | int |  |
| script/decision/reaction-set/reaction | ref-area | 56 | string |  |
| script/decision/reaction-set/reaction | ref-object | 152 | string |  |
| script/decision/reaction-set/reaction | ref-type | 208 | enum-like string ❓ | area, object |
| script/decision/reaction-set/reaction | reg | 31481 | int |  |
| script/decision/reaction-set/reaction | requester | 3 | string |  |
| script/decision/reaction-set/reaction | reset-stage | 250 | string |  |
| script/decision/reaction-set/reaction | respawn-after-despawn | 6794 | enum-like string ❓ | n, y |
| script/decision/reaction-set/reaction | score | 320 | int |  |
| script/decision/reaction-set/reaction | seq | 16653 | string |  |
| script/decision/reaction-set/reaction | seq-idx | 3 | int |  |
| script/decision/reaction-set/reaction | sight | 87 | enum-like string ❓ | one, party, team, zone |
| script/decision/reaction-set/reaction | social | 9283 | string |  |
| script/decision/reaction-set/reaction | spanw-delay | 1 | int |  |
| script/decision/reaction-set/reaction | spawn-1 | 11781 | string |  |
| script/decision/reaction-set/reaction | spawn-10 | 25 | string |  |
| script/decision/reaction-set/reaction | spawn-2 | 1080 | string |  |
| script/decision/reaction-set/reaction | spawn-3 | 547 | string |  |
| script/decision/reaction-set/reaction | spawn-4 | 340 | string |  |
| script/decision/reaction-set/reaction | spawn-5 | 205 | string |  |
| script/decision/reaction-set/reaction | spawn-6 | 156 | string |  |
| script/decision/reaction-set/reaction | spawn-7 | 74 | string |  |
| script/decision/reaction-set/reaction | spawn-8 | 68 | string |  |
| script/decision/reaction-set/reaction | spawn-9 | 45 | string |  |
| script/decision/reaction-set/reaction | spawn-angle-theta | 129 | int |  |
| script/decision/reaction-set/reaction | spawn-area | 10 | int |  |
| script/decision/reaction-set/reaction | spawn-dealy | 1 | int |  |
| script/decision/reaction-set/reaction | spawn-delay | 10511 | string |  |
| script/decision/reaction-set/reaction | spawn-force | 604 | enum-like string ❓ | n, y |
| script/decision/reaction-set/reaction | spawn-radius | 2628 | int |  |
| script/decision/reaction-set/reaction | spawn-radius-diff | 107 | int |  |
| script/decision/reaction-set/reaction | spawn-social | 6 | string |  |
| script/decision/reaction-set/reaction | spawn-target | 3133 | string |  |
| script/decision/reaction-set/reaction | special-id | 8732 | int |  |
| script/decision/reaction-set/reaction | start | 3007 | int |  |
| script/decision/reaction-set/reaction | state | 4485 | enum-like string ❓ | close, empty, n, open |
| script/decision/reaction-set/reaction | state1 | 77 | enum-like string ❓ | clos, close, open |
| script/decision/reaction-set/reaction | state2 | 5945 | enum-like string ❓ | close, empty, open |
| script/decision/reaction-set/reaction | subscriber | 208 | string |  |
| script/decision/reaction-set/reaction | target | 141003 | string |  |
| script/decision/reaction-set/reaction | target-1 | 12318 | string |  |
| script/decision/reaction-set/reaction | target-10 | 28 | string |  |
| script/decision/reaction-set/reaction | target-2 | 1750 | string |  |
| script/decision/reaction-set/reaction | target-3 | 1432 | string |  |
| script/decision/reaction-set/reaction | target-4 | 550 | string |  |
| script/decision/reaction-set/reaction | target-5 | 257 | string |  |
| script/decision/reaction-set/reaction | target-6 | 193 | string |  |
| script/decision/reaction-set/reaction | target-7 | 105 | string |  |
| script/decision/reaction-set/reaction | target-8 | 95 | string |  |
| script/decision/reaction-set/reaction | target-9 | 82 | string |  |
| script/decision/reaction-set/reaction | target1 | 40 | string |  |
| script/decision/reaction-set/reaction | target2 | 23450 | string |  |
| script/decision/reaction-set/reaction | teleport | 4 | string |  |
| script/decision/reaction-set/reaction | text | 38416 | string |  |
| script/decision/reaction-set/reaction | to | 274 | string |  |
| script/decision/reaction-set/reaction | transit-cond-idx | 4383 | int |  |
| script/decision/reaction-set/reaction | type | 239611 | string |  |
| script/decision/reaction-set/reaction | undying | 4838 | enum-like string ❓ | n, y |
| script/decision/reaction-set/reaction | use-spawn-target-yaw | 129 | enum-like string ❓ | y |
| script/decision/reaction-set/reaction | zone | 104 | string |  |
| script/decision/reaction-set/reaction | zreg | 1300 | int |  |
| script/decision/reaction-set/reeaction | seq | 5 | string |  |
| script/decision/reaction-set/reeaction | target | 5 | string |  |
| script/decision/reaction-set/reeaction | type | 5 | enum-like string ❓ | set-npc-act |
| script/decison | event | 1 | enum-like string ❓ | npc-combat-start |
| script/decison/reaction-set/reaction | text | 1 | string |  |
| script/decison/reaction-set/reaction | type | 1 | enum-like string ❓ | debug-trace |
| script/filter-set/filter | subject | 4 | string |  |
| script/filter-set/filter | type | 4 | enum-like string ❓ | pc, summoned |
| script/quest-decision | alias | 4801 | string |  |
| script/quest-decision/filter-set/filter | op | 4 | enum-like string ❓ | eq |
| script/quest-decision/filter-set/filter | spawn | 1 | string |  |
| script/quest-decision/filter-set/filter | subject | 9 | string |  |
| script/quest-decision/filter-set/filter | subject2 | 9 | string |  |
| script/quest-decision/filter-set/filter | type | 9 | enum-like string ❓ | npc-spawn, number, pc |
| script/quest-decision/filter-set/filter | value | 5 | string |  |
| script/quest-decision/reaction | state | 1 | enum-like string ❓ | open |
| script/quest-decision/reaction | state2 | 1 | enum-like string ❓ | open |
| script/quest-decision/reaction | target | 1 | string |  |
| script/quest-decision/reaction | target2 | 1 | string |  |
| script/quest-decision/reaction | type | 1 | enum-like string ❓ | set-env-state |
| script/quest-decision/reaction-set | probability | 33 | int |  |
| script/quest-decision/reaction-set/reaciton | from | 2 | string |  |
| script/quest-decision/reaction-set/reaciton | social | 2 | string |  |
| script/quest-decision/reaction-set/reaciton | type | 2 | enum-like string ❓ | play-social |
| script/quest-decision/reaction-set/reaction | amount | 718 | int |  |
| script/quest-decision/reaction-set/reaction | area | 11 | string |  |
| script/quest-decision/reaction-set/reaction | attack-target | 232 | string |  |
| script/quest-decision/reaction-set/reaction | attackable | 1426 | enum-like string ❓ | n, y |
| script/quest-decision/reaction-set/reaction | brain | 1513 | enum-like string ❓ | boss, citizen, monster |
| script/quest-decision/reaction-set/reaction | caster | 11 | string |  |
| script/quest-decision/reaction-set/reaction | cinematic | 128 | string |  |
| script/quest-decision/reaction-set/reaction | despawn-delay | 3 | int |  |
| script/quest-decision/reaction-set/reaction | despawn-force | 5846 | enum-like string ❓ | n, y |
| script/quest-decision/reaction-set/reaction | despawn-social-1 | 51 | string |  |
| script/quest-decision/reaction-set/reaction | dspawn-force | 4 | enum-like string ❓ | y |
| script/quest-decision/reaction-set/reaction | effect | 269 | string |  |
| script/quest-decision/reaction-set/reaction | effect-type | 14 | string |  |
| script/quest-decision/reaction-set/reaction | enable | 1941 | enum-like string ❓ | n, y |
| script/quest-decision/reaction-set/reaction | end | 20 | int |  |
| script/quest-decision/reaction-set/reaction | enter-cinematic | 18 | string |  |
| script/quest-decision/reaction-set/reaction | field-item | 12 | string |  |
| script/quest-decision/reaction-set/reaction | flag | 481 | enum-like string ❓ | n, y |
| script/quest-decision/reaction-set/reaction | from | 813 | string |  |
| script/quest-decision/reaction-set/reaction | group | 3159 | string |  |
| script/quest-decision/reaction-set/reaction | group-1 | 1615 | string |  |
| script/quest-decision/reaction-set/reaction | group-10 | 12 | string |  |
| script/quest-decision/reaction-set/reaction | group-2 | 561 | string |  |
| script/quest-decision/reaction-set/reaction | group-3 | 325 | string |  |
| script/quest-decision/reaction-set/reaction | group-4 | 222 | string |  |
| script/quest-decision/reaction-set/reaction | group-5 | 152 | string |  |
| script/quest-decision/reaction-set/reaction | group-6 | 69 | string |  |
| script/quest-decision/reaction-set/reaction | group-7 | 31 | string |  |
| script/quest-decision/reaction-set/reaction | group-8 | 20 | string |  |
| script/quest-decision/reaction-set/reaction | group-9 | 14 | string |  |
| script/quest-decision/reaction-set/reaction | hate-on | 1427 | enum-like string ❓ | n, y |
| script/quest-decision/reaction-set/reaction | immediately | 10 | enum-like string ❓ | y |
| script/quest-decision/reaction-set/reaction | index-1 | 3139 | int |  |
| script/quest-decision/reaction-set/reaction | index-10 | 1310 | int |  |
| script/quest-decision/reaction-set/reaction | index-11 | 21 | int |  |
| script/quest-decision/reaction-set/reaction | index-12 | 21 | int |  |
| script/quest-decision/reaction-set/reaction | index-13 | 15 | int |  |
| script/quest-decision/reaction-set/reaction | index-14 | 15 | int |  |
| script/quest-decision/reaction-set/reaction | index-15 | 12 | int |  |
| script/quest-decision/reaction-set/reaction | index-2 | 2404 | int |  |
| script/quest-decision/reaction-set/reaction | index-3 | 1925 | int |  |
| script/quest-decision/reaction-set/reaction | index-4 | 1707 | int |  |
| script/quest-decision/reaction-set/reaction | index-5 | 1578 | int |  |
| script/quest-decision/reaction-set/reaction | index-6 | 1452 | int |  |
| script/quest-decision/reaction-set/reaction | index-7 | 1371 | int |  |
| script/quest-decision/reaction-set/reaction | index-8 | 1355 | int |  |
| script/quest-decision/reaction-set/reaction | index-9 | 1328 | int |  |
| script/quest-decision/reaction-set/reaction | leave-cinematic | 25 | string |  |
| script/quest-decision/reaction-set/reaction | max | 20 | int |  |
| script/quest-decision/reaction-set/reaction | min | 20 | int |  |
| script/quest-decision/reaction-set/reaction | npc | 1028 | string |  |
| script/quest-decision/reaction-set/reaction | object | 116 | string |  |
| script/quest-decision/reaction-set/reaction | opponent | 165 | string |  |
| script/quest-decision/reaction-set/reaction | param-1 | 11 | string |  |
| script/quest-decision/reaction-set/reaction | pc-spawn | 233 | int |  |
| script/quest-decision/reaction-set/reaction | percent | 2 | int |  |
| script/quest-decision/reaction-set/reaction | phase-zone-enter-cinematic-1 | 11 | string |  |
| script/quest-decision/reaction-set/reaction | phase-zone-enter-cinematic-2 | 7 | string |  |
| script/quest-decision/reaction-set/reaction | phase-zone-enter-cinematic-3 | 1 | string |  |
| script/quest-decision/reaction-set/reaction | phase-zone-enter-cinematic-4 | 1 | string |  |
| script/quest-decision/reaction-set/reaction | phase-zone-enter-cinematic-5 | 1 | string |  |
| script/quest-decision/reaction-set/reaction | phase-zone-leave-cinematic-1 | 15 | string |  |
| script/quest-decision/reaction-set/reaction | phase-zone-leave-cinematic-2 | 8 | string |  |
| script/quest-decision/reaction-set/reaction | phase-zone-leave-cinematic-3 | 5 | string |  |
| script/quest-decision/reaction-set/reaction | phase-zone-leave-cinematic-4 | 3 | string |  |
| script/quest-decision/reaction-set/reaction | phase-zone-leave-cinematic-5 | 1 | string |  |
| script/quest-decision/reaction-set/reaction | phase-zone-leave-cinematic-6 | 2 | string |  |
| script/quest-decision/reaction-set/reaction | phase-zone-pc-spawn-1 | 85 | int |  |
| script/quest-decision/reaction-set/reaction | phase-zone-pc-spawn-12 | 1 | int |  |
| script/quest-decision/reaction-set/reaction | phase-zone-pc-spawn-2 | 35 | int |  |
| script/quest-decision/reaction-set/reaction | phase-zone-pc-spawn-3 | 10 | int |  |
| script/quest-decision/reaction-set/reaction | phase-zone-pc-spawn-4 | 3 | int |  |
| script/quest-decision/reaction-set/reaction | phase-zone-pc-spawn-5 | 1 | int |  |
| script/quest-decision/reaction-set/reaction | phase-zone-pc-spawn-6 | 2 | int |  |
| script/quest-decision/reaction-set/reaction | phase-zone-pc-spawn-7 | 2 | int |  |
| script/quest-decision/reaction-set/reaction | phase-zone-pc-spawn-9 | 4 | int |  |
| script/quest-decision/reaction-set/reaction | play-social-delay | 1 | int |  |
| script/quest-decision/reaction-set/reaction | probability | 2 | int |  |
| script/quest-decision/reaction-set/reaction | reg | 669 | int |  |
| script/quest-decision/reaction-set/reaction | respawn-after-despawn | 4364 | enum-like string ❓ | n, y |
| script/quest-decision/reaction-set/reaction | respawn-after-despwan | 2 | enum-like string ❓ | n |
| script/quest-decision/reaction-set/reaction | seq | 2833 | string |  |
| script/quest-decision/reaction-set/reaction | sight | 2 | enum-like string ❓ | party |
| script/quest-decision/reaction-set/reaction | social | 992 | string |  |
| script/quest-decision/reaction-set/reaction | spawn-1 | 4926 | string |  |
| script/quest-decision/reaction-set/reaction | spawn-10 | 37 | string |  |
| script/quest-decision/reaction-set/reaction | spawn-2 | 875 | string |  |
| script/quest-decision/reaction-set/reaction | spawn-3 | 564 | string |  |
| script/quest-decision/reaction-set/reaction | spawn-4 | 374 | string |  |
| script/quest-decision/reaction-set/reaction | spawn-5 | 188 | string |  |
| script/quest-decision/reaction-set/reaction | spawn-6 | 107 | string |  |
| script/quest-decision/reaction-set/reaction | spawn-7 | 70 | string |  |
| script/quest-decision/reaction-set/reaction | spawn-8 | 55 | string |  |
| script/quest-decision/reaction-set/reaction | spawn-9 | 47 | string |  |
| script/quest-decision/reaction-set/reaction | spawn-dealy | 2 | int |  |
| script/quest-decision/reaction-set/reaction | spawn-delay | 857 | int |  |
| script/quest-decision/reaction-set/reaction | spawn-force | 70 | enum-like string ❓ | y |
| script/quest-decision/reaction-set/reaction | spawn-radius | 63 | int |  |
| script/quest-decision/reaction-set/reaction | spawn-target | 85 | string |  |
| script/quest-decision/reaction-set/reaction | special-id | 40 | int |  |
| script/quest-decision/reaction-set/reaction | start | 20 | int |  |
| script/quest-decision/reaction-set/reaction | state | 1306 | enum-like string ❓ | close, empty, oepn, open |
| script/quest-decision/reaction-set/reaction | state1 | 9 | enum-like string ❓ | open |
| script/quest-decision/reaction-set/reaction | state2 | 2156 | enum-like string ❓ | close, empty, oepn, open |
| script/quest-decision/reaction-set/reaction | target | 8314 | string |  |
| script/quest-decision/reaction-set/reaction | target-1 | 5380 | string |  |
| script/quest-decision/reaction-set/reaction | target-10 | 97 | string |  |
| script/quest-decision/reaction-set/reaction | target-2 | 1402 | string |  |
| script/quest-decision/reaction-set/reaction | target-3 | 1088 | string |  |
| script/quest-decision/reaction-set/reaction | target-4 | 842 | string |  |
| script/quest-decision/reaction-set/reaction | target-5 | 467 | string |  |
| script/quest-decision/reaction-set/reaction | target-6 | 306 | string |  |
| script/quest-decision/reaction-set/reaction | target-7 | 211 | string |  |
| script/quest-decision/reaction-set/reaction | target-8 | 168 | string |  |
| script/quest-decision/reaction-set/reaction | target-9 | 130 | string |  |
| script/quest-decision/reaction-set/reaction | target1 | 3 | string |  |
| script/quest-decision/reaction-set/reaction | target2 | 4424 | string |  |
| script/quest-decision/reaction-set/reaction | teleport | 95 | string |  |
| script/quest-decision/reaction-set/reaction | text | 4860 | string |  |
| script/quest-decision/reaction-set/reaction | to | 386 | string |  |
| script/quest-decision/reaction-set/reaction | transit-cond-idx | 10 | int |  |
| script/quest-decision/reaction-set/reaction | type | 31826 | string |  |
| script/quest-decision/reaction-set/reaction | undying | 262 | enum-like string ❓ | n, y |
| script/quest-decision/reaction-set/reaction | zone | 238 | string |  |
| script/reaction-set | probability | 2 | int |  |
| script/reaction-set/reaction | from | 2 | string |  |
| script/reaction-set/reaction | social | 2 | string |  |
| script/reaction-set/reaction | text | 2 | string |  |
| script/reaction-set/reaction | type | 4 | enum-like string ❓ | debug-trace, play-social |
| script/script | alias | 1 | string |  |
| script/script/decision | event | 1 | enum-like string ❓ | npc-combat-start |
| script/script/decision/filter-set/filter | subject | 1 | string |  |
| script/script/decision/filter-set/filter | subject2 | 1 | string |  |
| script/script/decision/filter-set/filter | type | 1 | enum-like string ❓ | event-broadcasted |
| script/script/decision/filter-set/filter | value | 1 | enum-like string ❓ | n |
| script/script/decision/reaction-set/reaction | from | 1 | string |  |
| script/script/decision/reaction-set/reaction | social | 1 | string |  |
| script/script/decision/reaction-set/reaction | type | 1 | enum-like string ❓ | play-social |

### season-contents-schedule

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | accumulation-period | 3043 | int |  |
| record | closing-notice-period | 3043 | int |  |
| record | default-rating-score | 3043 | int |  |
| record | enable-weekly-ending-schedule | 3043 | enum-like string ❓ | n, y |
| record | enable-weekly-ending-schedule-for-qa | 3043 | enum-like string ❓ | n, y |
| record | forced-termination | 3043 | enum-like string ❓ | n, y |
| record | player-type | 3043 | enum-like string ❓ | challenge-party, guild, pc |
| record | ranking-minimum-score | 3043 | int |  |
| record | reward-group-data | 2458 | int |  |
| record | season-contents-schedule-number | 3043 | int |  |
| record | season-contents-type | 3043 | string |  |
| record | season-end-time | 3043 | string |  |
| record | season-name | 3043 | string |  |
| record | season-reward-expiration-time | 3043 | string |  |
| record | season-start-time | 3043 | string |  |
| record | season-type | 3043 | enum-like string ❓ | free, normal |
| record | service-mode | 3043 | string |  |
| record | weekly-ending-day | 246 | enum-like string ❓ | wed |
| record | weekly-ending-hour | 246 | int |  |
| record | weekly-ending-schedule-for-qa-1 | 2210 | string |  |
| record | weekly-ending-schedule-for-qa-10 | 18 | string |  |
| record | weekly-ending-schedule-for-qa-11 | 12 | string |  |
| record | weekly-ending-schedule-for-qa-12 | 12 | string |  |
| record | weekly-ending-schedule-for-qa-13 | 12 | string |  |
| record | weekly-ending-schedule-for-qa-2 | 1634 | string |  |
| record | weekly-ending-schedule-for-qa-3 | 239 | string |  |
| record | weekly-ending-schedule-for-qa-4 | 91 | string |  |
| record | weekly-ending-schedule-for-qa-5 | 35 | string |  |
| record | weekly-ending-schedule-for-qa-6 | 18 | string |  |
| record | weekly-ending-schedule-for-qa-7 | 18 | string |  |
| record | weekly-ending-schedule-for-qa-8 | 18 | string |  |
| record | weekly-ending-schedule-for-qa-9 | 18 | string |  |

### set-item

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 372 | string |  |
| record | count-2-effect-1 | 28 | string |  |
| record | count-2-skill-skin | 1 | string |  |
| record | count-3-effect-1 | 285 | string |  |
| record | count-3-skill-modify-info-group-1 | 133 | string |  |
| record | count-3-skill-modify-info-group-2 | 133 | string |  |
| record | count-3-skill-modify-info-group-3 | 133 | string |  |
| record | count-3-skill-modify-info-group-6 | 133 | string |  |
| record | count-3-skill-modify-info-group-7 | 133 | string |  |
| record | count-3-skill-modify-info-group-8 | 133 | string |  |
| record | count-4-effect-1 | 15 | string |  |
| record | count-5-effect-1 | 334 | string |  |
| record | count-5-effect-2 | 48 | string |  |
| record | count-5-skill-modify-info-group-1 | 133 | string |  |
| record | count-5-skill-modify-info-group-2 | 133 | string |  |
| record | count-5-skill-modify-info-group-3 | 133 | string |  |
| record | count-5-skill-modify-info-group-6 | 133 | string |  |
| record | count-5-skill-modify-info-group-7 | 133 | string |  |
| record | count-5-skill-modify-info-group-8 | 133 | string |  |
| record | count-8-effect-1 | 249 | string |  |
| record | count-8-effect-2 | 1 | string |  |
| record | count-8-skill-modify-info-group-1 | 133 | string |  |
| record | count-8-skill-modify-info-group-2 | 133 | string |  |
| record | count-8-skill-modify-info-group-3 | 133 | string |  |
| record | count-8-skill-modify-info-group-6 | 133 | string |  |
| record | count-8-skill-modify-info-group-7 | 133 | string |  |
| record | count-8-skill-modify-info-group-8 | 133 | string |  |

### simple-field

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 7 | string |  |
| record | group | 7 | string |  |
| record | id | 7 | int |  |
| record | max-pc-count | 7 | int |  |
| record | required-level | 3 | int |  |
| record | required-preceding-quest-1 | 1 | string |  |
| record | zone | 7 | int |  |

### skill-build-up

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 272 | string |  |
| record | max-level | 272 | int |  |
| record | parent-skill3-id | 272 | int |  |
| record | required-build-up-point-1 | 272 | int |  |
| record | required-build-up-point-10 | 272 | int |  |
| record | required-build-up-point-2 | 272 | int |  |
| record | required-build-up-point-3 | 272 | int |  |
| record | required-build-up-point-4 | 272 | int |  |
| record | required-build-up-point-5 | 272 | int |  |
| record | required-build-up-point-6 | 272 | int |  |
| record | required-build-up-point-7 | 272 | int |  |
| record | required-build-up-point-8 | 272 | int |  |
| record | required-build-up-point-9 | 272 | int |  |
| record | skill-modify-info-1 | 272 | string |  |
| record | skill-modify-info-10 | 272 | string |  |
| record | skill-modify-info-2 | 272 | string |  |
| record | skill-modify-info-3 | 272 | string |  |
| record | skill-modify-info-4 | 272 | string |  |
| record | skill-modify-info-5 | 272 | string |  |
| record | skill-modify-info-6 | 272 | string |  |
| record | skill-modify-info-7 | 272 | string |  |
| record | skill-modify-info-8 | 272 | string |  |
| record | skill-modify-info-9 | 272 | string |  |

### skill-by-equipment

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 205 | string |  |
| record | skill3-id-1 | 205 | int |  |
| record | skill3-id-2 | 36 | int |  |

### skill-caster-function-control

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 3 | string |  |
| record | casting-caster-function-1 | 3 | enum-like string ❓ | block-move |
| record | exec-caster-function-1-1 | 2 | enum-like string ❓ | block-move |
| record | exec-caster-function-2-1 | 1 | enum-like string ❓ | block-move |
| record | exec-caster-function-3-1 | 1 | enum-like string ❓ | block-move |
| record | exec-caster-function-4-1 | 1 | enum-like string ❓ | block-move |
| record | exec-caster-function-5-1 | 1 | enum-like string ❓ | block-move |
| record | flying-caster-function-1-1 | 3 | enum-like string ❓ | block-move |
| record | flying-caster-function-2-1 | 1 | enum-like string ❓ | block-move |
| record | flying-caster-function-3-1 | 1 | enum-like string ❓ | block-move |
| record | flying-caster-function-4-1 | 1 | enum-like string ❓ | block-move |
| record | flying-caster-function-5-1 | 1 | enum-like string ❓ | block-move |

### skill-combo

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | event-type | 133 | enum-like string ❓ | attack, attacked |
| record | next-combo-1-1 | 97 | int |  |
| record | next-combo-1-2 | 10 | int |  |
| record | next-combo-2-1 | 2 | int |  |
| record | next-combo-2-2 | 2 | int |  |
| record | next-combo-2-3 | 2 | int |  |
| record | next-combo-2-4 | 9 | int |  |
| record | next-combo-2-7 | 6 | int |  |
| record | next-combo-2-8 | 2 | int |  |
| record | skill-id | 133 | int |  |
| record | skill-result | 133 | enum-like string ❓ | bounce, critical-hit, dodge, hit, miss, parry, perfect-parry |

### skill-inheritance

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | parent-skill-id | 2674 | int |  |
| record | skill-id | 2674 | int |  |

### skill-level-modify

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | attack-level-modify | 603 | int |  |
| record | contents-type | 603 | enum-like string ❓ | duel, heromask-duel, normal |
| record | defend-level-modify | 603 | int |  |
| record | level-diff | 603 | int |  |
| record | level-diff-sign | 603 | enum-like string ❓ | minus, plus |

### skill-mastery-level-modify

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | attack-level-modify | 603 | int |  |
| record | contents-type | 603 | enum-like string ❓ | duel, heromask-duel, normal |
| record | defend-level-modify | 603 | int |  |
| record | level-diff | 603 | int |  |
| record | level-diff-sign | 603 | enum-like string ❓ | minus, plus |

### skill-modify-info

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 13552 | string |  |
| record | damage-power-percent-modify-percent | 9090 | int |  |
| record | parent-skill3-id-1 | 10435 | int |  |
| record | parent-skill3-id-2 | 3557 | int |  |
| record | parent-skill3-id-3 | 901 | string |  |
| record | parent-skill3-id-4 | 261 | int |  |
| record | recycle-duration-modify-diff | 3240 | int |  |
| record | recycle-duration-modify-percent | 1168 | int |  |
| record | sp-consume-modify-diff-1 | 19 | int |  |
| record | systematization | 1 | string |  |
| record | type | 13547 | string |  |

### skill-modify-info-group

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 6980 | string |  |
| record | job-style | 6980 | string |  |
| record | skill-modify-info-1 | 6509 | string |  |
| record | skill-modify-info-2 | 1517 | string |  |
| record | skill-modify-info-3 | 1076 | string |  |
| record | skill-modify-info-4 | 992 | string |  |

### skill-train-category

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | job | 1226 | string |  |
| record | view-skill-id | 1226 | int |  |

### skill-training-room

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 1 | string |  |
| record | enter-pc-spawn | 1 | string |  |
| record | id | 1 | int |  |
| record | init-creature-state-request-recycle-second | 1 | int |  |
| record | remain-effect-type-1 | 1 | string |  |
| record | remain-effect-type-10 | 1 | string |  |
| record | remain-effect-type-11 | 1 | string |  |
| record | remain-effect-type-12 | 1 | string |  |
| record | remain-effect-type-13 | 1 | string |  |
| record | remain-effect-type-14 | 1 | enum-like string ❓ | boutique-free |
| record | remain-effect-type-15 | 1 | string |  |
| record | remain-effect-type-2 | 1 | string |  |
| record | remain-effect-type-3 | 1 | string |  |
| record | remain-effect-type-4 | 1 | string |  |
| record | remain-effect-type-5 | 1 | string |  |
| record | remain-effect-type-6 | 1 | string |  |
| record | remain-effect-type-7 | 1 | string |  |
| record | remain-effect-type-8 | 1 | string |  |
| record | remain-effect-type-9 | 1 | string |  |
| record | required-level | 1 | int |  |
| record | required-preceding-quest-1 | 1 | string |  |
| record | support-effect-1 | 1 | string |  |
| record | zone | 1 | int |  |

### skill-training-room-group

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | group-required-level | 96 | int |  |
| record | has-reward | 96 | enum-like string ❓ | y |
| record | id | 96 | int |  |
| record | job | 96 | string |  |
| record | job-style | 96 | string |  |
| record | subject-id-1 | 96 | int |  |
| record | subject-id-2 | 90 | int |  |
| record | subject-id-3 | 78 | int |  |
| record | subject-id-4 | 67 | int |  |
| record | subject-id-5 | 62 | int |  |
| record | subject-id-6 | 42 | int |  |
| record | subject-id-7 | 17 | int |  |
| record | subject-id-8 | 8 | int |  |

### skill-training-room-subject

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | id | 552 | int |  |
| record | npc-effect-1 | 3 | string |  |
| record | npc-effect-2 | 3 | string |  |
| record | npc-effect-3 | 3 | string |  |
| record | npc-effect-4 | 3 | string |  |
| record | npc-effect-5 | 3 | string |  |
| record | pc-effect-1 | 496 | string |  |
| record | pc-effect-2 | 30 | string |  |
| record | pc-effect-3 | 5 | string |  |
| record | pc-effect-4 | 2 | string |  |
| record | subject-stage | 552 | string |  |
| record | support-npc | 56 | string |  |
| record | type | 552 | enum-like string ❓ | boss-training, class-training |

### skill-trait

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 504 | string |  |
| record | fixed-skill3-id-1 | 60 | int |  |
| record | fixed-skill3-id-10 | 54 | int |  |
| record | fixed-skill3-id-11 | 54 | int |  |
| record | fixed-skill3-id-12 | 54 | int |  |
| record | fixed-skill3-id-13 | 54 | int |  |
| record | fixed-skill3-id-14 | 54 | int |  |
| record | fixed-skill3-id-15 | 53 | int |  |
| record | fixed-skill3-id-16 | 53 | int |  |
| record | fixed-skill3-id-17 | 53 | int |  |
| record | fixed-skill3-id-18 | 53 | int |  |
| record | fixed-skill3-id-19 | 53 | int |  |
| record | fixed-skill3-id-2 | 54 | int |  |
| record | fixed-skill3-id-20 | 53 | int |  |
| record | fixed-skill3-id-21 | 53 | int |  |
| record | fixed-skill3-id-22 | 53 | int |  |
| record | fixed-skill3-id-23 | 53 | int |  |
| record | fixed-skill3-id-24 | 52 | int |  |
| record | fixed-skill3-id-25 | 52 | int |  |
| record | fixed-skill3-id-26 | 52 | int |  |
| record | fixed-skill3-id-27 | 51 | int |  |
| record | fixed-skill3-id-28 | 51 | int |  |
| record | fixed-skill3-id-29 | 51 | int |  |
| record | fixed-skill3-id-3 | 54 | int |  |
| record | fixed-skill3-id-30 | 51 | int |  |
| record | fixed-skill3-id-31 | 50 | int |  |
| record | fixed-skill3-id-32 | 47 | int |  |
| record | fixed-skill3-id-33 | 46 | int |  |
| record | fixed-skill3-id-34 | 43 | int |  |
| record | fixed-skill3-id-35 | 43 | int |  |
| record | fixed-skill3-id-36 | 43 | int |  |
| record | fixed-skill3-id-37 | 43 | int |  |
| record | fixed-skill3-id-38 | 39 | int |  |
| record | fixed-skill3-id-39 | 37 | int |  |
| record | fixed-skill3-id-4 | 54 | int |  |
| record | fixed-skill3-id-40 | 36 | int |  |
| record | fixed-skill3-id-41 | 34 | int |  |
| record | fixed-skill3-id-42 | 30 | int |  |
| record | fixed-skill3-id-43 | 28 | int |  |
| record | fixed-skill3-id-44 | 25 | int |  |
| record | fixed-skill3-id-45 | 23 | int |  |
| record | fixed-skill3-id-46 | 19 | int |  |
| record | fixed-skill3-id-47 | 19 | int |  |
| record | fixed-skill3-id-48 | 19 | int |  |
| record | fixed-skill3-id-49 | 18 | int |  |
| record | fixed-skill3-id-5 | 54 | int |  |
| record | fixed-skill3-id-50 | 17 | int |  |
| record | fixed-skill3-id-51 | 16 | int |  |
| record | fixed-skill3-id-52 | 15 | int |  |
| record | fixed-skill3-id-53 | 15 | int |  |
| record | fixed-skill3-id-54 | 14 | int |  |
| record | fixed-skill3-id-55 | 13 | int |  |
| record | fixed-skill3-id-56 | 8 | int |  |
| record | fixed-skill3-id-57 | 7 | int |  |
| record | fixed-skill3-id-58 | 7 | int |  |
| record | fixed-skill3-id-59 | 6 | int |  |
| record | fixed-skill3-id-6 | 54 | int |  |
| record | fixed-skill3-id-60 | 3 | int |  |
| record | fixed-skill3-id-61 | 1 | int |  |
| record | fixed-skill3-id-7 | 54 | int |  |
| record | fixed-skill3-id-8 | 54 | int |  |
| record | fixed-skill3-id-9 | 54 | int |  |
| record | job | 504 | string |  |
| record | job-style | 504 | string |  |
| record | tier | 504 | int |  |
| record | tier-variation | 504 | int |  |
| record | variable-skill3-id-1 | 449 | int |  |
| record | variable-skill3-id-10 | 30 | int |  |
| record | variable-skill3-id-11 | 29 | int |  |
| record | variable-skill3-id-12 | 27 | int |  |
| record | variable-skill3-id-13 | 26 | int |  |
| record | variable-skill3-id-14 | 26 | int |  |
| record | variable-skill3-id-15 | 23 | int |  |
| record | variable-skill3-id-16 | 20 | int |  |
| record | variable-skill3-id-17 | 18 | int |  |
| record | variable-skill3-id-18 | 17 | int |  |
| record | variable-skill3-id-19 | 12 | int |  |
| record | variable-skill3-id-2 | 252 | int |  |
| record | variable-skill3-id-20 | 11 | int |  |
| record | variable-skill3-id-21 | 7 | int |  |
| record | variable-skill3-id-22 | 4 | int |  |
| record | variable-skill3-id-23 | 4 | int |  |
| record | variable-skill3-id-24 | 2 | int |  |
| record | variable-skill3-id-25 | 2 | int |  |
| record | variable-skill3-id-26 | 2 | int |  |
| record | variable-skill3-id-27 | 2 | int |  |
| record | variable-skill3-id-28 | 1 | int |  |
| record | variable-skill3-id-29 | 1 | int |  |
| record | variable-skill3-id-3 | 158 | int |  |
| record | variable-skill3-id-30 | 1 | int |  |
| record | variable-skill3-id-31 | 1 | int |  |
| record | variable-skill3-id-32 | 1 | int |  |
| record | variable-skill3-id-4 | 102 | int |  |
| record | variable-skill3-id-5 | 67 | int |  |
| record | variable-skill3-id-6 | 42 | int |  |
| record | variable-skill3-id-7 | 35 | int |  |
| record | variable-skill3-id-8 | 34 | int |  |
| record | variable-skill3-id-9 | 30 | int |  |
| record | variable-skill3-variation-id-1 | 449 | int |  |
| record | variable-skill3-variation-id-10 | 30 | int |  |
| record | variable-skill3-variation-id-11 | 29 | int |  |
| record | variable-skill3-variation-id-12 | 27 | int |  |
| record | variable-skill3-variation-id-13 | 26 | int |  |
| record | variable-skill3-variation-id-14 | 26 | int |  |
| record | variable-skill3-variation-id-15 | 23 | int |  |
| record | variable-skill3-variation-id-16 | 20 | int |  |
| record | variable-skill3-variation-id-17 | 18 | int |  |
| record | variable-skill3-variation-id-18 | 17 | int |  |
| record | variable-skill3-variation-id-19 | 12 | int |  |
| record | variable-skill3-variation-id-2 | 252 | int |  |
| record | variable-skill3-variation-id-20 | 11 | int |  |
| record | variable-skill3-variation-id-21 | 7 | int |  |
| record | variable-skill3-variation-id-22 | 4 | int |  |
| record | variable-skill3-variation-id-23 | 4 | int |  |
| record | variable-skill3-variation-id-24 | 2 | int |  |
| record | variable-skill3-variation-id-25 | 2 | int |  |
| record | variable-skill3-variation-id-26 | 2 | int |  |
| record | variable-skill3-variation-id-27 | 2 | int |  |
| record | variable-skill3-variation-id-28 | 1 | int |  |
| record | variable-skill3-variation-id-29 | 1 | int |  |
| record | variable-skill3-variation-id-3 | 158 | int |  |
| record | variable-skill3-variation-id-30 | 1 | int |  |
| record | variable-skill3-variation-id-31 | 1 | int |  |
| record | variable-skill3-variation-id-32 | 1 | int |  |
| record | variable-skill3-variation-id-4 | 102 | int |  |
| record | variable-skill3-variation-id-5 | 67 | int |  |
| record | variable-skill3-variation-id-6 | 42 | int |  |
| record | variable-skill3-variation-id-7 | 35 | int |  |
| record | variable-skill3-variation-id-8 | 34 | int |  |
| record | variable-skill3-variation-id-9 | 30 | int |  |

### skill3

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 20791 | string |  |
| record | attribute-1 | 10595 | string |  |
| record | attribute-2 | 3170 | string |  |
| record | attribute-3 | 686 | string |  |
| record | attribute-4 | 84 | string |  |
| record | auto-skill-result | 2506 | enum-like string ❓ | critical-hit, hit, miss |
| record | bound-recycle-group | 28 | string |  |
| record | bound-recycle-group-id | 25 | int |  |
| record | burrow-gather | 1139 | enum-like string ❓ | n, y |
| record | campfire | 1 | string |  |
| record | cancel-by-override-skill | 8194 | string |  |
| record | cannot-override-flow | 5976 | string |  |
| record | cast-caster-change-stance | 234 | string |  |
| record | cast-caster-dispel-attribute-1 | 4106 | string |  |
| record | cast-caster-dispel-attribute-2 | 666 | string |  |
| record | cast-caster-dispel-attribute-3 | 520 | string |  |
| record | cast-caster-dispel-attribute-4 | 295 | string |  |
| record | cast-caster-dispel-attribute-5 | 112 | string |  |
| record | cast-caster-dispel-attribute-6 | 77 | string |  |
| record | cast-caster-dispel-attribute-7 | 12 | string |  |
| record | cast-caster-dispel-attribute-8 | 7 | string |  |
| record | cast-condition | 20748 | string |  |
| record | cast-duration | 16513 | int |  |
| record | cast-effect-1 | 7879 | string |  |
| record | cast-effect-2 | 4376 | string |  |
| record | cast-effect-3 | 2491 | string |  |
| record | cast-effect-4 | 1154 | string |  |
| record | cast-effect-5 | 294 | string |  |
| record | cast-effect-6 | 85 | string |  |
| record | cast-effect-to-my-summoned-summoner-1 | 445 | string |  |
| record | cast-effect-to-my-summoned-summoner-2 | 139 | string |  |
| record | cast-effect-to-my-summoned-summoner-3 | 8 | string |  |
| record | cast-effect-to-my-summoned-summoner-4 | 6 | string |  |
| record | cast-effect-to-my-summoned-summoner-distance | 681 | int |  |
| record | caster-function-control | 21 | string |  |
| record | caster-yaw-to-knockback | 312 | enum-like string ❓ | y |
| record | casting-delay | 20720 | enum-like string ❓ | n, y |
| record | category | 20692 | string |  |
| record | change-check-position-by-chain | 43 | enum-like string ❓ | y |
| record | change-npc-yaw | 11452 | string |  |
| record | change-npc-yaw-max-chasing-angle | 29 | string |  |
| record | change-npc-yaw-on-cooldown | 98 | enum-like string ❓ | n, y |
| record | change-npc-yaw-plus-angle | 17 | int |  |
| record | check-flow-repeat-condition | 8 | enum-like string ❓ | y |
| record | children-skill-1 | 76 | string |  |
| record | children-skill-2 | 40 | string |  |
| record | children-skill-3 | 40 | string |  |
| record | children-skill-4 | 40 | string |  |
| record | children-skill-yaw-plus-angle-1 | 36 | int |  |
| record | children-skill-yaw-plus-angle-2 | 32 | int |  |
| record | children-skill-yaw-plus-angle-3 | 32 | int |  |
| record | children-skill-yaw-plus-angle-4 | 32 | int |  |
| record | clear-style-score | 2 | enum-like string ❓ | y |
| record | combo-duration | 2469 | int |  |
| record | consume-hp-type | 281 | enum-like string ❓ | total-max-percent |
| record | consume-hp-value | 281 | int |  |
| record | consume-hyper-energy-point | 14 | string |  |
| record | consume-repeat-sp-type-1 | 2 | enum-like string ❓ | point |
| record | consume-repeat-sp-type-2 | 2 | enum-like string ❓ | point |
| record | consume-repeat-sp-value-2 | 2 | int |  |
| record | consume-sp-type-1 | 3634 | enum-like string ❓ | current-percent, point |
| record | consume-sp-type-2 | 2 | enum-like string ❓ | point |
| record | consume-sp-value-1 | 3530 | int |  |
| record | consume-sp-value-2 | 10 | int |  |
| record | consume-summoned-hp-type | 5 | enum-like string ❓ | total-max-percent |
| record | consume-summoned-hp-value | 5 | int |  |
| record | counterattack-section-duration-1 | 3 | int |  |
| record | counterattack-section-duration-2 | 1 | int |  |
| record | damage-rate-pvp | 3194 | int |  |
| record | damage-rate-standard-stats | 757 | int |  |
| record | dash-attribute | 20791 | string |  |
| record | detach-buff-when-skill-end | 29 | string |  |
| record | exec-caster-effect-1-1 | 6889 | string |  |
| record | exec-caster-effect-1-2 | 2835 | string |  |
| record | exec-caster-effect-1-3 | 1238 | string |  |
| record | exec-caster-effect-1-4 | 387 | string |  |
| record | exec-caster-effect-2-1 | 1541 | string |  |
| record | exec-caster-effect-2-2 | 416 | string |  |
| record | exec-caster-effect-2-3 | 196 | string |  |
| record | exec-caster-effect-2-4 | 87 | string |  |
| record | exec-caster-effect-3-1 | 681 | string |  |
| record | exec-caster-effect-3-2 | 158 | string |  |
| record | exec-caster-effect-3-3 | 52 | string |  |
| record | exec-caster-effect-3-4 | 10 | string |  |
| record | exec-caster-effect-4-1 | 522 | string |  |
| record | exec-caster-effect-4-2 | 120 | string |  |
| record | exec-caster-effect-4-3 | 37 | string |  |
| record | exec-caster-effect-4-4 | 13 | string |  |
| record | exec-caster-effect-5-1 | 369 | string |  |
| record | exec-caster-effect-5-2 | 79 | string |  |
| record | exec-caster-effect-5-3 | 23 | string |  |
| record | exec-caster-effect-5-4 | 9 | string |  |
| record | exec-caster-effect-skill-result-1-1 | 1895 | string |  |
| record | exec-caster-effect-skill-result-1-2 | 1093 | string |  |
| record | exec-caster-effect-skill-result-1-3 | 491 | string |  |
| record | exec-caster-effect-skill-result-1-4 | 148 | string |  |
| record | exec-caster-effect-skill-result-2-1 | 352 | string |  |
| record | exec-caster-effect-skill-result-2-2 | 110 | string |  |
| record | exec-caster-effect-skill-result-2-3 | 77 | string |  |
| record | exec-caster-effect-skill-result-2-4 | 23 | string |  |
| record | exec-caster-effect-skill-result-3-1 | 176 | string |  |
| record | exec-caster-effect-skill-result-3-2 | 45 | string |  |
| record | exec-caster-effect-skill-result-3-3 | 21 | string |  |
| record | exec-caster-effect-skill-result-3-4 | 6 | enum-like string ❓ | bounce, hit-critical-hit, miss |
| record | exec-caster-effect-skill-result-4-1 | 149 | string |  |
| record | exec-caster-effect-skill-result-4-2 | 29 | string |  |
| record | exec-caster-effect-skill-result-4-3 | 13 | string |  |
| record | exec-caster-effect-skill-result-4-4 | 9 | enum-like string ❓ | all, bounce, hit-critical-hit |
| record | exec-caster-effect-skill-result-5-1 | 113 | string |  |
| record | exec-caster-effect-skill-result-5-2 | 22 | string |  |
| record | exec-caster-effect-skill-result-5-3 | 11 | string |  |
| record | exec-caster-effect-skill-result-5-4 | 5 | enum-like string ❓ | bounce, hit-critical-hit |
| record | exec-duration-1 | 20523 | int |  |
| record | exec-duration-2 | 6532 | int |  |
| record | exec-duration-3 | 3206 | int |  |
| record | exec-duration-4 | 2114 | int |  |
| record | exec-duration-5 | 1410 | int |  |
| record | exec-effect-1-1 | 18305 | string |  |
| record | exec-effect-1-2 | 13101 | string |  |
| record | exec-effect-1-3 | 5269 | string |  |
| record | exec-effect-1-4 | 2624 | string |  |
| record | exec-effect-1-5 | 1515 | string |  |
| record | exec-effect-1-6 | 1009 | string |  |
| record | exec-effect-1-7 | 470 | string |  |
| record | exec-effect-1-8 | 91 | string |  |
| record | exec-effect-2-1 | 6147 | string |  |
| record | exec-effect-2-2 | 4533 | string |  |
| record | exec-effect-2-3 | 1650 | string |  |
| record | exec-effect-2-4 | 737 | string |  |
| record | exec-effect-2-5 | 328 | string |  |
| record | exec-effect-2-6 | 154 | string |  |
| record | exec-effect-2-7 | 74 | string |  |
| record | exec-effect-2-8 | 37 | string |  |
| record | exec-effect-3-1 | 3131 | string |  |
| record | exec-effect-3-2 | 2295 | string |  |
| record | exec-effect-3-3 | 824 | string |  |
| record | exec-effect-3-4 | 320 | string |  |
| record | exec-effect-3-5 | 152 | string |  |
| record | exec-effect-3-6 | 69 | string |  |
| record | exec-effect-3-7 | 19 | string |  |
| record | exec-effect-3-8 | 15 | string |  |
| record | exec-effect-4-1 | 2052 | string |  |
| record | exec-effect-4-2 | 1526 | string |  |
| record | exec-effect-4-3 | 579 | string |  |
| record | exec-effect-4-4 | 245 | string |  |
| record | exec-effect-4-5 | 126 | string |  |
| record | exec-effect-4-6 | 64 | string |  |
| record | exec-effect-4-7 | 13 | string |  |
| record | exec-effect-4-8 | 13 | string |  |
| record | exec-effect-5-1 | 1361 | string |  |
| record | exec-effect-5-2 | 1020 | string |  |
| record | exec-effect-5-3 | 446 | string |  |
| record | exec-effect-5-4 | 223 | string |  |
| record | exec-effect-5-5 | 125 | string |  |
| record | exec-effect-5-6 | 58 | string |  |
| record | exec-effect-5-7 | 13 | string |  |
| record | exec-effect-5-8 | 11 | string |  |
| record | exec-gather-type-1 | 20646 | string |  |
| record | exec-gather-type-2 | 6579 | string |  |
| record | exec-gather-type-3 | 3228 | string |  |
| record | exec-gather-type-4 | 2131 | string |  |
| record | exec-gather-type-5 | 1418 | string |  |
| record | exec-negation-dir-gather-1 | 15 | enum-like string ❓ | y |
| record | exec-target-angle-1 | 274 | int |  |
| record | exec-target-angle-2 | 197 | int |  |
| record | exec-target-angle-3 | 101 | int |  |
| record | exec-target-angle-4 | 68 | int |  |
| record | exec-target-angle-5 | 18 | int |  |
| record | exec-target-offset-angle-1 | 10 | int |  |
| record | exec-target-offset-angle-2 | 4 | int |  |
| record | exec-target-offset-angle-3 | 4 | int |  |
| record | exec-target-offset-angle-4 | 4 | int |  |
| record | exec-target-offset-angle-5 | 4 | int |  |
| record | exec-target-offset-distance-1 | 10 | int |  |
| record | exec-target-offset-distance-2 | 4 | int |  |
| record | exec-target-offset-distance-3 | 4 | int |  |
| record | exec-target-offset-distance-4 | 4 | int |  |
| record | exec-target-offset-distance-5 | 4 | int |  |
| record | expanded-flow-repeat-count | 168 | int |  |
| record | expanded-flow-repeat-start-flow-step | 168 | int |  |
| record | fire-miss | 723 | enum-like string ❓ | n, y |
| record | flow-repeat | 20671 | int |  |
| record | flowtype | 20692 | enum-like string ❓ | directly-simslot, keep-mainslot, leave-caster, transfer-simslot |
| record | fly-speed-1 | 4133 | int |  |
| record | fly-speed-2 | 492 | int |  |
| record | fly-speed-3 | 360 | int |  |
| record | fly-speed-4 | 278 | int |  |
| record | fly-speed-5 | 196 | int |  |
| record | force-counter-dir-1 | 145 | string |  |
| record | force-counter-dir-2 | 1 | string |  |
| record | force-counter-skill-1 | 118 | string |  |
| record | force-counter-skill-2 | 1 | string |  |
| record | gain-style-score | 32 | int |  |
| record | gather-laser-except-caster | 19 | enum-like string ❓ | n |
| record | gather-range | 20753 | string |  |
| record | global-recycle-group | 8443 | int |  |
| record | global-recycle-group-duration | 8316 | int |  |
| record | ground-type | 2382 | string |  |
| record | hate | 20632 | int |  |
| record | hostile-type | 20715 | enum-like string ❓ | hostile, none, support |
| record | id | 20791 | int |  |
| record | ignore-cast-fly-los | 80 | enum-like string ❓ | y |
| record | ignore-fire-exec-fail | 197 | enum-like string ❓ | n, y |
| record | is-ego-skill | 140 | enum-like string ❓ | y |
| record | knockback-source-position | 46 | enum-like string ❓ | caster, target |
| record | max-hyper-energy-point | 6 | string |  |
| record | max-style-score | 24 | int |  |
| record | melee-counter-dir-1 | 10372 | string |  |
| record | melee-counter-dir-2 | 1 | string |  |
| record | melee-counter-skill-1 | 117 | string |  |
| record | melee-counter-skill-2 | 1 | string |  |
| record | npc-cooldown | 12371 | int |  |
| record | npc-skill-spawn-group-index-1 | 32 | int |  |
| record | npc-skill-spawn-group-index-2 | 2 | int |  |
| record | occurrence-event-in-exec-1 | 1477 | enum-like string ❓ | n, y |
| record | occurrence-event-in-exec-2 | 457 | enum-like string ❓ | n |
| record | occurrence-event-in-exec-3 | 228 | enum-like string ❓ | n, y |
| record | occurrence-event-in-exec-4 | 153 | enum-like string ❓ | n, y |
| record | occurrence-event-in-exec-5 | 115 | enum-like string ❓ | n |
| record | passive-effect-1 | 78 | string |  |
| record | passive-effect-2 | 4 | string |  |
| record | passive-effect-7 | 1 | enum-like string ❓ | target |
| record | range-counter-dir-1 | 120 | string |  |
| record | range-counter-dir-2 | 1 | string |  |
| record | range-counter-skill-1 | 119 | string |  |
| record | range-counter-skill-2 | 1 | string |  |
| record | recycle-group | 20726 | string |  |
| record | recycle-group-duration | 8257 | int |  |
| record | recycle-group-id | 8277 | int |  |
| record | repeat-chain | 212 | string |  |
| record | replace-target | 162 | enum-like string ❓ | link-target, my-master, my-summoned, my-summoneds-target |
| record | request-user-cancel | 7525 | enum-like string ❓ | n, y |
| record | result-controll | 20791 | string |  |
| record | return-duration-1 | 18 | int |  |
| record | return-duration-2 | 16 | int |  |
| record | return-duration-3 | 13 | int |  |
| record | return-duration-4 | 13 | int |  |
| record | return-duration-5 | 8 | int |  |
| record | return-speed-1 | 587 | int |  |
| record | return-speed-2 | 3 | int |  |
| record | revised-effect-equip-probability-in-exec-1 | 639 | int |  |
| record | revised-effect-equip-probability-in-exec-2 | 138 | int |  |
| record | revised-effect-equip-probability-in-exec-3 | 70 | int |  |
| record | revised-effect-equip-probability-in-exec-4 | 72 | int |  |
| record | revised-effect-equip-probability-in-exec-5 | 59 | int |  |
| record | save-combo | 19220 | enum-like string ❓ | n, y |
| record | sit-down | 192 | enum-like string ❓ | y |
| record | skill-modify-limit | 6362 | string |  |
| record | skill-result-rule | 20731 | enum-like string ❓ | decide-each, decide-execute |
| record | skill-type | 20693 | string |  |
| record | stop-next-skill-critical-hit | 5 | string |  |
| record | stop-next-skill-hit | 5 | string |  |
| record | stop-skill-result-bounce | 1 | enum-like string ❓ | y |
| record | stop-skill-result-counter | 1 | enum-like string ❓ | y |
| record | stop-skill-result-critical-hit | 5 | enum-like string ❓ | y |
| record | stop-skill-result-dodge | 1 | enum-like string ❓ | y |
| record | stop-skill-result-hit | 5 | enum-like string ❓ | y |
| record | stop-skill-result-miss | 1 | enum-like string ❓ | y |
| record | stop-skill-result-parry | 1 | enum-like string ❓ | y |
| record | stop-skill-result-perfect-parry | 1 | enum-like string ❓ | y |
| record | style-duration | 24 | int |  |
| record | succession-prev-skill | 39 | string |  |
| record | succession-prev-skill-id | 39 | int |  |
| record | summoned-job | 6 | string |  |
| record | swing-caster-effect-1-1 | 283 | string |  |
| record | swing-caster-effect-1-2 | 35 | string |  |
| record | swing-caster-effect-1-3 | 19 | string |  |
| record | swing-caster-effect-2-1 | 34 | string |  |
| record | swing-caster-effect-2-2 | 9 | string |  |
| record | swing-caster-effect-2-3 | 6 | string |  |
| record | swing-caster-effect-3-1 | 31 | string |  |
| record | swing-caster-effect-3-2 | 15 | string |  |
| record | swing-caster-effect-3-3 | 12 | string |  |
| record | swing-caster-effect-4-1 | 23 | string |  |
| record | swing-caster-effect-4-2 | 12 | string |  |
| record | swing-caster-effect-4-3 | 12 | string |  |
| record | swing-caster-effect-5-1 | 6 | string |  |
| record | swing-caster-effect-5-2 | 5 | string |  |
| record | swing-caster-effect-5-3 | 6 | string |  |
| record | swing-duration-1 | 10389 | int |  |
| record | swing-duration-2 | 1654 | int |  |
| record | swing-duration-3 | 961 | int |  |
| record | swing-duration-4 | 734 | int |  |
| record | swing-duration-5 | 480 | int |  |
| record | swing-duration-formula | 629 | enum-like string ❓ | n |
| record | systematization-1 | 5712 | string |  |
| record | systematization-10 | 15 | string |  |
| record | systematization-11 | 3 | string |  |
| record | systematization-2 | 4949 | string |  |
| record | systematization-3 | 3675 | string |  |
| record | systematization-4 | 2348 | string |  |
| record | systematization-5 | 1379 | string |  |
| record | systematization-6 | 694 | string |  |
| record | systematization-7 | 173 | string |  |
| record | systematization-8 | 86 | string |  |
| record | systematization-9 | 55 | string |  |
| record | target-filter | 20671 | string |  |
| record | target-kill-effect-1 | 127 | string |  |
| record | target-kill-effect-2 | 27 | string |  |
| record | throw-link-target | 5 | enum-like string ❓ | y |
| record | toggle | 1 | enum-like string ❓ | wide |
| record | trigger-effect-1 | 16 | string |  |
| record | trigger-effect-2 | 10 | string |  |
| record | trigger-effect-3 | 7 | string |  |
| record | trigger-effect-4 | 3 | string |  |
| record | trigger-gather-type | 16 | string |  |
| record | trigger-prev-skill | 58 | string |  |
| record | trigger-prev-skill-id | 58 | int |  |
| record | type | 20791 | enum-like string ❓ | action, active-skill, passive-skill |
| record | variation-id | 20791 | int |  |

### skillAttributeRule

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | attribute | 51 | string |  |
| record | auto-skill-result-1 | 14 | enum-like string ❓ | bounce, critical-hit, miss, perfect-parry |
| record | auto-skill-result-2 | 2 | enum-like string ❓ | critical-hit, dodge |
| record | auto-skill-result-3 | 1 | enum-like string ❓ | miss |
| record | auto-skill-result-4 | 1 | enum-like string ❓ | perfect-parry |
| record | auto-skill-result-5 | 1 | enum-like string ❓ | dodge |
| record | auto-skill-result-6 | 1 | enum-like string ❓ | perfect-parry |
| record | block-caster-flag-1 | 8 | string |  |
| record | block-caster-flag-2 | 1 | enum-like string ❓ | chilblain |
| record | block-caster-flag-3 | 1 | enum-like string ❓ | shadow |
| record | dir-1 | 14 | enum-like string ❓ | back, front, front-back |
| record | dir-2 | 2 | enum-like string ❓ | front-back |
| record | dir-3 | 1 | enum-like string ❓ | front-back |
| record | dir-4 | 1 | enum-like string ❓ | front |
| record | dir-5 | 1 | enum-like string ❓ | front-back |
| record | dir-6 | 1 | enum-like string ❓ | front |
| record | target-flag-1 | 15 | string |  |
| record | target-flag-2 | 2 | enum-like string ❓ | concentration, stun |
| record | target-flag-3 | 1 | enum-like string ❓ | magnetic-seal |
| record | target-flag-4 | 1 | string |  |
| record | target-flag-5 | 1 | enum-like string ❓ | hide |
| record | target-flag-6 | 1 | string |  |

### skillSystematization

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 155 | string |  |
| record | id | 155 | int |  |

### social

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 35266 | string |  |
| record | duration | 34734 | int |  |
| record | spawn-cinematic | 2 | string |  |

### soul-npc-skill

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 16 | string |  |
| record | skill3-1 | 16 | string |  |
| record | skill3-10 | 16 | string |  |
| record | skill3-2 | 16 | string |  |
| record | skill3-3 | 16 | string |  |
| record | skill3-4 | 16 | string |  |
| record | skill3-5 | 16 | string |  |
| record | skill3-6 | 16 | string |  |
| record | skill3-7 | 16 | string |  |
| record | skill3-8 | 16 | string |  |
| record | skill3-9 | 16 | string |  |
| record | soul-npc-skill-category | 16 | enum-like string ❓ | assist, control, damage, special |

### special-skill-gauge

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | gauge-increase-per-sec | 12 | int |  |
| record | job | 12 | string |  |

### stance

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | change-default-effect-flag-1 | 1 | enum-like string ❓ | stun |
| record | change-default-effect-flag-2 | 1 | enum-like string ❓ | kneel |
| record | change-default-effect-flag-3 | 1 | enum-like string ❓ | down |
| record | change-default-effect-flag-4 | 1 | enum-like string ❓ | knockback |
| record | default-sp | 45 | int |  |
| record | effect-1 | 17 | string |  |
| record | effect-2 | 4 | string |  |
| record | effect-3 | 4 | string |  |
| record | effect-4 | 2 | string |  |
| record | impossible-equip | 4 | enum-like string ❓ | y |
| record | initialize | 45 | enum-like string ❓ | n, y |
| record | max-sp | 45 | int |  |
| record | stance-type | 45 | string |  |

### store

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 151 | string |  |
| record | buy-rate | 151 | int |  |
| record | faction | 14 | string |  |
| record | faction-reputation-1 | 8 | int |  |
| record | faction-reputation-10 | 8 | int |  |
| record | faction-reputation-11 | 8 | int |  |
| record | faction-reputation-12 | 8 | int |  |
| record | faction-reputation-13 | 8 | int |  |
| record | faction-reputation-14 | 8 | int |  |
| record | faction-reputation-15 | 2 | int |  |
| record | faction-reputation-16 | 2 | int |  |
| record | faction-reputation-17 | 2 | int |  |
| record | faction-reputation-18 | 2 | int |  |
| record | faction-reputation-19 | 2 | int |  |
| record | faction-reputation-2 | 10 | int |  |
| record | faction-reputation-20 | 2 | int |  |
| record | faction-reputation-21 | 2 | int |  |
| record | faction-reputation-22 | 2 | int |  |
| record | faction-reputation-23 | 2 | int |  |
| record | faction-reputation-24 | 2 | int |  |
| record | faction-reputation-25 | 2 | int |  |
| record | faction-reputation-26 | 2 | int |  |
| record | faction-reputation-27 | 2 | int |  |
| record | faction-reputation-3 | 4 | int |  |
| record | faction-reputation-4 | 4 | int |  |
| record | faction-reputation-5 | 4 | int |  |
| record | faction-reputation-6 | 4 | int |  |
| record | faction-reputation-7 | 4 | int |  |
| record | faction-reputation-8 | 4 | int |  |
| record | faction-reputation-9 | 8 | int |  |
| record | id | 151 | int |  |
| record | item-1 | 150 | string |  |
| record | item-10 | 11 | string |  |
| record | item-11 | 10 | string |  |
| record | item-12 | 10 | string |  |
| record | item-13 | 9 | string |  |
| record | item-14 | 9 | string |  |
| record | item-15 | 5 | string |  |
| record | item-16 | 3 | string |  |
| record | item-17 | 2 | string |  |
| record | item-18 | 2 | string |  |
| record | item-19 | 2 | string |  |
| record | item-2 | 147 | string |  |
| record | item-20 | 2 | string |  |
| record | item-21 | 2 | string |  |
| record | item-22 | 2 | string |  |
| record | item-23 | 2 | string |  |
| record | item-24 | 2 | string |  |
| record | item-25 | 2 | string |  |
| record | item-26 | 2 | string |  |
| record | item-27 | 2 | string |  |
| record | item-3 | 145 | string |  |
| record | item-4 | 131 | string |  |
| record | item-5 | 127 | string |  |
| record | item-6 | 123 | string |  |
| record | item-7 | 122 | string |  |
| record | item-8 | 55 | string |  |
| record | item-9 | 17 | string |  |
| record | resell-rate | 151 | int |  |
| record | sell-rate | 151 | int |  |

### store-by-item

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | acquire-item-1 | 41 | string |  |
| record | acquire-item-10 | 14 | string |  |
| record | acquire-item-11 | 14 | string |  |
| record | acquire-item-12 | 14 | string |  |
| record | acquire-item-13 | 14 | string |  |
| record | acquire-item-14 | 14 | string |  |
| record | acquire-item-15 | 13 | string |  |
| record | acquire-item-16 | 9 | string |  |
| record | acquire-item-17 | 7 | string |  |
| record | acquire-item-18 | 7 | string |  |
| record | acquire-item-19 | 6 | string |  |
| record | acquire-item-2 | 37 | string |  |
| record | acquire-item-20 | 6 | string |  |
| record | acquire-item-21 | 5 | string |  |
| record | acquire-item-22 | 5 | string |  |
| record | acquire-item-23 | 4 | string |  |
| record | acquire-item-24 | 1 | string |  |
| record | acquire-item-25 | 1 | string |  |
| record | acquire-item-26 | 1 | string |  |
| record | acquire-item-27 | 1 | string |  |
| record | acquire-item-28 | 1 | string |  |
| record | acquire-item-29 | 1 | string |  |
| record | acquire-item-3 | 36 | string |  |
| record | acquire-item-30 | 1 | string |  |
| record | acquire-item-31 | 1 | string |  |
| record | acquire-item-32 | 1 | string |  |
| record | acquire-item-4 | 32 | string |  |
| record | acquire-item-5 | 29 | string |  |
| record | acquire-item-6 | 28 | string |  |
| record | acquire-item-7 | 20 | string |  |
| record | acquire-item-8 | 19 | string |  |
| record | acquire-item-9 | 14 | string |  |
| record | alias | 46 | string |  |
| record | dispose-item-1 | 41 | string |  |
| record | dispose-item-10 | 14 | string |  |
| record | dispose-item-11 | 14 | string |  |
| record | dispose-item-12 | 14 | string |  |
| record | dispose-item-13 | 14 | string |  |
| record | dispose-item-14 | 14 | string |  |
| record | dispose-item-15 | 13 | string |  |
| record | dispose-item-16 | 9 | string |  |
| record | dispose-item-17 | 7 | string |  |
| record | dispose-item-18 | 7 | string |  |
| record | dispose-item-19 | 6 | string |  |
| record | dispose-item-2 | 37 | string |  |
| record | dispose-item-20 | 6 | string |  |
| record | dispose-item-21 | 5 | string |  |
| record | dispose-item-22 | 5 | string |  |
| record | dispose-item-23 | 4 | string |  |
| record | dispose-item-24 | 1 | string |  |
| record | dispose-item-25 | 1 | string |  |
| record | dispose-item-26 | 1 | string |  |
| record | dispose-item-27 | 1 | string |  |
| record | dispose-item-28 | 1 | string |  |
| record | dispose-item-29 | 1 | string |  |
| record | dispose-item-3 | 36 | string |  |
| record | dispose-item-30 | 1 | string |  |
| record | dispose-item-31 | 1 | string |  |
| record | dispose-item-32 | 1 | string |  |
| record | dispose-item-4 | 32 | string |  |
| record | dispose-item-5 | 29 | string |  |
| record | dispose-item-6 | 28 | string |  |
| record | dispose-item-7 | 20 | string |  |
| record | dispose-item-8 | 19 | string |  |
| record | dispose-item-9 | 14 | string |  |
| record | dispose-item-count-1 | 41 | int |  |
| record | dispose-item-count-10 | 14 | int |  |
| record | dispose-item-count-11 | 14 | int |  |
| record | dispose-item-count-12 | 14 | int |  |
| record | dispose-item-count-13 | 14 | int |  |
| record | dispose-item-count-14 | 14 | int |  |
| record | dispose-item-count-15 | 13 | int |  |
| record | dispose-item-count-16 | 9 | int |  |
| record | dispose-item-count-17 | 7 | int |  |
| record | dispose-item-count-18 | 7 | int |  |
| record | dispose-item-count-19 | 6 | int |  |
| record | dispose-item-count-2 | 37 | int |  |
| record | dispose-item-count-20 | 6 | int |  |
| record | dispose-item-count-21 | 5 | int |  |
| record | dispose-item-count-22 | 5 | int |  |
| record | dispose-item-count-23 | 4 | int |  |
| record | dispose-item-count-24 | 1 | int |  |
| record | dispose-item-count-25 | 1 | int |  |
| record | dispose-item-count-26 | 1 | int |  |
| record | dispose-item-count-27 | 1 | int |  |
| record | dispose-item-count-28 | 1 | int |  |
| record | dispose-item-count-29 | 1 | int |  |
| record | dispose-item-count-3 | 36 | int |  |
| record | dispose-item-count-30 | 1 | int |  |
| record | dispose-item-count-31 | 1 | int |  |
| record | dispose-item-count-32 | 1 | int |  |
| record | dispose-item-count-4 | 32 | int |  |
| record | dispose-item-count-5 | 29 | int |  |
| record | dispose-item-count-6 | 28 | int |  |
| record | dispose-item-count-7 | 20 | int |  |
| record | dispose-item-count-8 | 19 | int |  |
| record | dispose-item-count-9 | 14 | int |  |
| record | id | 46 | int |  |

### store2

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 408 | string |  |
| record | buy-price-1 | 401 | string |  |
| record | buy-price-10 | 183 | string |  |
| record | buy-price-100 | 2 | string |  |
| record | buy-price-101 | 2 | string |  |
| record | buy-price-102 | 2 | string |  |
| record | buy-price-103 | 2 | string |  |
| record | buy-price-104 | 2 | string |  |
| record | buy-price-105 | 2 | string |  |
| record | buy-price-106 | 2 | string |  |
| record | buy-price-107 | 2 | string |  |
| record | buy-price-108 | 2 | string |  |
| record | buy-price-109 | 1 | string |  |
| record | buy-price-11 | 177 | string |  |
| record | buy-price-110 | 1 | string |  |
| record | buy-price-111 | 1 | string |  |
| record | buy-price-12 | 165 | string |  |
| record | buy-price-13 | 152 | string |  |
| record | buy-price-14 | 149 | string |  |
| record | buy-price-15 | 104 | string |  |
| record | buy-price-16 | 103 | string |  |
| record | buy-price-17 | 95 | string |  |
| record | buy-price-18 | 88 | string |  |
| record | buy-price-19 | 85 | string |  |
| record | buy-price-2 | 327 | string |  |
| record | buy-price-20 | 83 | string |  |
| record | buy-price-21 | 80 | string |  |
| record | buy-price-22 | 78 | string |  |
| record | buy-price-23 | 77 | string |  |
| record | buy-price-24 | 75 | string |  |
| record | buy-price-25 | 73 | string |  |
| record | buy-price-26 | 47 | string |  |
| record | buy-price-27 | 45 | string |  |
| record | buy-price-28 | 42 | string |  |
| record | buy-price-29 | 39 | string |  |
| record | buy-price-3 | 313 | string |  |
| record | buy-price-30 | 37 | string |  |
| record | buy-price-31 | 37 | string |  |
| record | buy-price-32 | 37 | string |  |
| record | buy-price-33 | 37 | string |  |
| record | buy-price-34 | 36 | string |  |
| record | buy-price-35 | 35 | string |  |
| record | buy-price-36 | 32 | string |  |
| record | buy-price-37 | 32 | string |  |
| record | buy-price-38 | 29 | string |  |
| record | buy-price-39 | 27 | string |  |
| record | buy-price-4 | 308 | string |  |
| record | buy-price-40 | 25 | string |  |
| record | buy-price-41 | 25 | string |  |
| record | buy-price-42 | 25 | string |  |
| record | buy-price-43 | 22 | string |  |
| record | buy-price-44 | 21 | string |  |
| record | buy-price-45 | 20 | string |  |
| record | buy-price-46 | 20 | string |  |
| record | buy-price-47 | 20 | string |  |
| record | buy-price-48 | 19 | string |  |
| record | buy-price-49 | 17 | string |  |
| record | buy-price-5 | 302 | string |  |
| record | buy-price-50 | 17 | string |  |
| record | buy-price-51 | 17 | string |  |
| record | buy-price-52 | 17 | string |  |
| record | buy-price-53 | 17 | string |  |
| record | buy-price-54 | 16 | string |  |
| record | buy-price-55 | 15 | string |  |
| record | buy-price-56 | 15 | string |  |
| record | buy-price-57 | 14 | string |  |
| record | buy-price-58 | 14 | string |  |
| record | buy-price-59 | 14 | string |  |
| record | buy-price-6 | 299 | string |  |
| record | buy-price-60 | 14 | string |  |
| record | buy-price-61 | 14 | string |  |
| record | buy-price-62 | 14 | string |  |
| record | buy-price-63 | 14 | string |  |
| record | buy-price-64 | 14 | string |  |
| record | buy-price-65 | 14 | string |  |
| record | buy-price-66 | 13 | string |  |
| record | buy-price-67 | 13 | string |  |
| record | buy-price-68 | 13 | string |  |
| record | buy-price-69 | 13 | string |  |
| record | buy-price-7 | 293 | string |  |
| record | buy-price-70 | 13 | string |  |
| record | buy-price-71 | 13 | string |  |
| record | buy-price-72 | 13 | string |  |
| record | buy-price-73 | 13 | string |  |
| record | buy-price-74 | 12 | string |  |
| record | buy-price-75 | 12 | string |  |
| record | buy-price-76 | 12 | string |  |
| record | buy-price-77 | 11 | string |  |
| record | buy-price-78 | 11 | string |  |
| record | buy-price-79 | 11 | string |  |
| record | buy-price-8 | 200 | string |  |
| record | buy-price-80 | 11 | string |  |
| record | buy-price-81 | 11 | string |  |
| record | buy-price-82 | 11 | string |  |
| record | buy-price-83 | 11 | string |  |
| record | buy-price-84 | 11 | string |  |
| record | buy-price-85 | 11 | string |  |
| record | buy-price-86 | 11 | string |  |
| record | buy-price-87 | 11 | string |  |
| record | buy-price-88 | 6 | string |  |
| record | buy-price-89 | 5 | string |  |
| record | buy-price-9 | 186 | string |  |
| record | buy-price-90 | 4 | string |  |
| record | buy-price-91 | 4 | string |  |
| record | buy-price-92 | 4 | string |  |
| record | buy-price-93 | 4 | string |  |
| record | buy-price-94 | 4 | string |  |
| record | buy-price-95 | 4 | string |  |
| record | buy-price-96 | 4 | string |  |
| record | buy-price-97 | 4 | string |  |
| record | buy-price-98 | 3 | string |  |
| record | buy-price-99 | 2 | string |  |
| record | faction | 30 | string |  |
| record | id | 408 | int |  |
| record | item-1 | 401 | string |  |
| record | item-10 | 183 | string |  |
| record | item-100 | 2 | string |  |
| record | item-101 | 2 | string |  |
| record | item-102 | 2 | string |  |
| record | item-103 | 2 | string |  |
| record | item-104 | 2 | string |  |
| record | item-105 | 2 | string |  |
| record | item-106 | 2 | string |  |
| record | item-107 | 2 | string |  |
| record | item-108 | 2 | string |  |
| record | item-109 | 1 | string |  |
| record | item-11 | 177 | string |  |
| record | item-110 | 1 | string |  |
| record | item-111 | 1 | string |  |
| record | item-12 | 165 | string |  |
| record | item-13 | 152 | string |  |
| record | item-14 | 149 | string |  |
| record | item-15 | 104 | string |  |
| record | item-16 | 103 | string |  |
| record | item-17 | 95 | string |  |
| record | item-18 | 88 | string |  |
| record | item-19 | 85 | string |  |
| record | item-2 | 327 | string |  |
| record | item-20 | 83 | string |  |
| record | item-21 | 80 | string |  |
| record | item-22 | 78 | string |  |
| record | item-23 | 77 | string |  |
| record | item-24 | 75 | string |  |
| record | item-25 | 73 | string |  |
| record | item-26 | 47 | string |  |
| record | item-27 | 45 | string |  |
| record | item-28 | 42 | string |  |
| record | item-29 | 39 | string |  |
| record | item-3 | 313 | string |  |
| record | item-30 | 37 | string |  |
| record | item-31 | 37 | string |  |
| record | item-32 | 37 | string |  |
| record | item-33 | 37 | string |  |
| record | item-34 | 36 | string |  |
| record | item-35 | 35 | string |  |
| record | item-36 | 32 | string |  |
| record | item-37 | 32 | string |  |
| record | item-38 | 29 | string |  |
| record | item-39 | 27 | string |  |
| record | item-4 | 308 | string |  |
| record | item-40 | 25 | string |  |
| record | item-41 | 25 | string |  |
| record | item-42 | 25 | string |  |
| record | item-43 | 22 | string |  |
| record | item-44 | 21 | string |  |
| record | item-45 | 20 | string |  |
| record | item-46 | 20 | string |  |
| record | item-47 | 20 | string |  |
| record | item-48 | 19 | string |  |
| record | item-49 | 17 | string |  |
| record | item-5 | 302 | string |  |
| record | item-50 | 17 | string |  |
| record | item-51 | 17 | string |  |
| record | item-52 | 17 | string |  |
| record | item-53 | 17 | string |  |
| record | item-54 | 16 | string |  |
| record | item-55 | 15 | string |  |
| record | item-56 | 15 | string |  |
| record | item-57 | 14 | string |  |
| record | item-58 | 14 | string |  |
| record | item-59 | 14 | string |  |
| record | item-6 | 299 | string |  |
| record | item-60 | 14 | string |  |
| record | item-61 | 14 | string |  |
| record | item-62 | 14 | string |  |
| record | item-63 | 14 | string |  |
| record | item-64 | 14 | string |  |
| record | item-65 | 14 | string |  |
| record | item-66 | 13 | string |  |
| record | item-67 | 13 | string |  |
| record | item-68 | 13 | string |  |
| record | item-69 | 13 | string |  |
| record | item-7 | 293 | string |  |
| record | item-70 | 13 | string |  |
| record | item-71 | 13 | string |  |
| record | item-72 | 13 | string |  |
| record | item-73 | 13 | string |  |
| record | item-74 | 12 | string |  |
| record | item-75 | 12 | string |  |
| record | item-76 | 12 | string |  |
| record | item-77 | 11 | string |  |
| record | item-78 | 11 | string |  |
| record | item-79 | 11 | string |  |
| record | item-8 | 200 | string |  |
| record | item-80 | 11 | string |  |
| record | item-81 | 11 | string |  |
| record | item-82 | 11 | string |  |
| record | item-83 | 11 | string |  |
| record | item-84 | 11 | string |  |
| record | item-85 | 11 | string |  |
| record | item-86 | 11 | string |  |
| record | item-87 | 11 | string |  |
| record | item-88 | 6 | string |  |
| record | item-89 | 5 | string |  |
| record | item-9 | 186 | string |  |
| record | item-90 | 4 | string |  |
| record | item-91 | 4 | string |  |
| record | item-92 | 4 | string |  |
| record | item-93 | 4 | string |  |
| record | item-94 | 4 | string |  |
| record | item-95 | 4 | string |  |
| record | item-96 | 4 | string |  |
| record | item-97 | 4 | string |  |
| record | item-98 | 3 | string |  |
| record | item-99 | 2 | string |  |

### summoned

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | id | 8 | int |  |
| record | immune-effect-attribute-1 | 4 | string |  |
| record | immune-effect-attribute-2 | 4 | string |  |
| record | immune-effect-attribute-3 | 4 | string |  |
| record | immune-effect-attribute-4 | 4 | string |  |
| record | immune-effect-attribute-5 | 4 | string |  |
| record | immune-effect-attribute-6 | 4 | string |  |
| record | immune-effect-attribute-7 | 4 | string |  |
| record | immune-effect-attribute-8 | 4 | string |  |
| record | immune-skill-attribute-1 | 4 | string |  |
| record | immune-skill-attribute-2 | 4 | string |  |
| record | immune-skill-attribute-3 | 4 | string |  |
| record | immune-skill-attribute-4 | 4 | string |  |
| record | job | 8 | string |  |
| record | lifetime-duration | 8 | int |  |
| record | race | 8 | string |  |
| record | radius | 8 | int |  |
| record | respawn-in-social | 8 | string |  |
| record | respawn-out-social | 8 | string |  |
| record | sex | 8 | enum-like string ❓ | sex-none |
| record | spawn-social | 4 | string |  |
| record | speed | 8 | int |  |
| record | stance | 8 | string |  |

### summonedLevel

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | abnormal-attack-base-percent | 1928 | int |  |
| record | abnormal-attack-power-modify | 1928 | int |  |
| record | abnormal-attack-power-value | 1928 | int |  |
| record | abnormal-defend-base-percent | 1928 | int |  |
| record | abnormal-defend-power-modify | 1928 | int |  |
| record | abnormal-defend-power-value | 1928 | int |  |
| record | aoe-defend-base-percent | 1928 | int |  |
| record | aoe-defend-power-modify | 1928 | int |  |
| record | aoe-defend-power-value | 1928 | int |  |
| record | attack-attribute-base-percent | 1928 | int |  |
| record | attack-attribute-modify | 1928 | int |  |
| record | attack-attribute-value | 1928 | int |  |
| record | attack-counter-damage-modify | 1928 | int |  |
| record | attack-critical-base-percent | 1928 | int |  |
| record | attack-critical-damage-modify | 1928 | int |  |
| record | attack-critical-damage-percent | 1928 | int |  |
| record | attack-critical-damage-value | 1928 | int |  |
| record | attack-critical-value | 1928 | int |  |
| record | attack-critical-value-modify | 1928 | int |  |
| record | attack-damage-modify-diff | 1928 | int |  |
| record | attack-damage-modify-percent | 1928 | int |  |
| record | attack-hit-base-percent | 1928 | int |  |
| record | attack-hit-value | 1928 | int |  |
| record | attack-hit-value-modify | 1928 | int |  |
| record | attack-parry-pierce-modify | 1928 | int |  |
| record | attack-perfect-parry-damage-modify | 1928 | int |  |
| record | attack-pierce-modify | 1928 | int |  |
| record | attack-pierce-value | 1448 | int |  |
| record | attack-power-creature-max | 1928 | int |  |
| record | attack-power-creature-min | 1928 | int |  |
| record | attack-power-equip-max | 1928 | int |  |
| record | attack-power-equip-min | 1928 | int |  |
| record | attack-stiff-duration-base-percent | 1928 | int |  |
| record | attack-stiff-duration-value | 1928 | int |  |
| record | attack-stiff-duration-value-modify | 1928 | int |  |
| record | cast-duration-base-percent | 1928 | int |  |
| record | cast-duration-value | 1928 | int |  |
| record | cast-duration-value-modify | 1928 | int |  |
| record | contents-type | 1928 | enum-like string ❓ | battle-royal, duel, dungeon, normal |
| record | defend-bounce-percent | 1928 | int |  |
| record | defend-counter-reduce-modify | 1928 | int |  |
| record | defend-counter-reduce-percent | 1928 | int |  |
| record | defend-critical-base-percent | 1928 | int |  |
| record | defend-critical-damage-modify | 1928 | int |  |
| record | defend-critical-damage-percent | 1440 | int |  |
| record | defend-critical-value | 1928 | int |  |
| record | defend-critical-value-modify | 1928 | int |  |
| record | defend-damage-modify-diff | 1928 | int |  |
| record | defend-damage-modify-percent | 1928 | int |  |
| record | defend-dodge-base-percent | 1928 | int |  |
| record | defend-dodge-value | 1928 | int |  |
| record | defend-dodge-value-modify | 1928 | int |  |
| record | defend-force-damage-reduce-percent | 1928 | int |  |
| record | defend-force-value-modify | 1928 | int |  |
| record | defend-immune-base-percent | 1928 | int |  |
| record | defend-miss-base-percent | 1928 | int |  |
| record | defend-parry-base-percent | 1928 | int |  |
| record | defend-parry-reduce-diff | 1928 | int |  |
| record | defend-parry-reduce-modify | 1928 | int |  |
| record | defend-parry-reduce-percent | 1928 | int |  |
| record | defend-parry-value | 1928 | int |  |
| record | defend-parry-value-modify | 1928 | int |  |
| record | defend-perfect-parry-base-percent | 1928 | int |  |
| record | defend-perfect-parry-reduce-modify | 1928 | int |  |
| record | defend-perfect-parry-reduce-percent | 1928 | int |  |
| record | defend-physical-damage-reduce-percent | 1928 | int |  |
| record | defend-physical-value-modify | 1928 | int |  |
| record | defend-power-creature-value | 1928 | int |  |
| record | defend-power-equip-value | 1928 | int |  |
| record | defend-resist-power-creature-value | 1928 | int |  |
| record | defend-resist-power-equip-value | 1928 | int |  |
| record | defend-stiff-duration-base-percent | 1928 | int |  |
| record | defend-stiff-duration-value | 1928 | int |  |
| record | defend-stiff-duration-value-modify | 1928 | int |  |
| record | hate-base-percent | 1928 | int |  |
| record | hate-power-creature-value | 1928 | int |  |
| record | hate-power-modify | 1928 | int |  |
| record | heal-power-base-percent | 1928 | int |  |
| record | heal-power-modify | 1928 | int |  |
| record | hp-regen | 1928 | int |  |
| record | hp-regen-combat | 1928 | int |  |
| record | is-duelbot-summoned | 1928 | enum-like string ❓ | n, y |
| record | job | 1928 | string |  |
| record | level | 1928 | int |  |
| record | max-hp | 1928 | int |  |
| record | race | 1928 | string |  |
| record | summon-ability-rate-abnormal-attack-base-percent | 1928 | int |  |
| record | summon-ability-rate-abnormal-attack-power-value | 1928 | int |  |
| record | summon-ability-rate-abnormal-defend-base-percent | 1928 | int |  |
| record | summon-ability-rate-abnormal-defend-power-value | 1928 | int |  |
| record | summon-ability-rate-attack-attribute-base-percent | 1928 | int |  |
| record | summon-ability-rate-attack-attribute-value | 1928 | int |  |
| record | summon-ability-rate-attack-critical-base-percent | 1928 | int |  |
| record | summon-ability-rate-attack-critical-damage-value | 1928 | int |  |
| record | summon-ability-rate-attack-critical-value | 1928 | int |  |
| record | summon-ability-rate-attack-damage-modify-diff | 1928 | int |  |
| record | summon-ability-rate-attack-damage-modify-percent | 1928 | int |  |
| record | summon-ability-rate-attack-hit-base-percent | 1928 | int |  |
| record | summon-ability-rate-attack-hit-value | 1928 | int |  |
| record | summon-ability-rate-attack-pierce-value | 1928 | int |  |
| record | summon-ability-rate-attack-power-creature-max | 1928 | int |  |
| record | summon-ability-rate-attack-power-creature-min | 1928 | int |  |
| record | summon-ability-rate-attack-stiff-duration-base-percent | 1928 | int |  |
| record | summon-ability-rate-attack-stiff-duration-value | 1928 | int |  |
| record | summon-ability-rate-cast-duration-base-percent | 1928 | int |  |
| record | summon-ability-rate-cast-duration-value | 1928 | int |  |
| record | summon-ability-rate-defend-critical-base-percent | 1928 | int |  |
| record | summon-ability-rate-defend-critical-value | 1928 | int |  |
| record | summon-ability-rate-defend-damage-modify-diff | 1928 | int |  |
| record | summon-ability-rate-defend-damage-modify-percent | 1928 | int |  |
| record | summon-ability-rate-defend-dodge-base-percent | 1928 | int |  |
| record | summon-ability-rate-defend-dodge-value | 1928 | int |  |
| record | summon-ability-rate-defend-force-damage-reduce-percent | 1928 | int |  |
| record | summon-ability-rate-defend-parry-base-percent | 1928 | int |  |
| record | summon-ability-rate-defend-parry-value | 1928 | int |  |
| record | summon-ability-rate-defend-physical-damage-reduce-percent | 1928 | int |  |
| record | summon-ability-rate-defend-power-creature-value | 1928 | int |  |
| record | summon-ability-rate-defend-resist-power-creature-value | 1928 | int |  |
| record | summon-ability-rate-defend-stiff-duration-base-percent | 1928 | int |  |
| record | summon-ability-rate-defend-stiff-duration-value | 1928 | int |  |
| record | summon-ability-rate-hp-regen | 1928 | int |  |
| record | summon-ability-rate-hp-regen-combat | 1928 | int |  |
| record | summon-ability-rate-max-hp | 1928 | int |  |
| record | summon-ability-rate-pve-boss-level-npc-attack-power-equip-max | 480 | int |  |
| record | summon-ability-rate-pve-boss-level-npc-attack-power-equip-min | 480 | int |  |
| record | summon-ability-rate-pve-boss-level-npc-defend-power-equip-value | 480 | int |  |
| record | summon-ability-rate-pvp-attack-power-equip-max | 480 | int |  |
| record | summon-ability-rate-pvp-attack-power-equip-min | 480 | int |  |
| record | summon-ability-rate-pvp-defend-power-equip-value | 480 | int |  |

### survey

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |

### teleport

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | activated-faction | 10 | int |  |
| record | alias | 151 | string |  |
| record | distance-from-start-teleport | 151 | int |  |
| record | id | 151 | int |  |
| record | joined-faction | 2 | enum-like string ❓ | y |
| record | price-percent | 151 | int |  |
| record | retired | 2 | enum-like string ❓ | y |
| record | teleport-position | 151 | string |  |

### tendency-field

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 2 | string |  |
| record | check-score-interval-minute | 2 | int |  |
| record | check-score-number-of-repetition | 2 | int |  |
| record | check-score-required-min-faction-member-count | 1 | int |  |
| record | check-score-required-tendency-score-1 | 2 | int |  |
| record | check-score-required-tendency-score-2 | 2 | int |  |
| record | defeat-bonus-point-percent | 1 | int |  |
| record | defeat-faction-score-bonus-percent | 1 | int |  |
| record | defeat-faction-separation-faction-score | 1 | int |  |
| record | defeat-field-play-point-bonus-percent | 1 | int |  |
| record | disable-move-to-arena | 2 | enum-like string ❓ | n |
| record | disable-move-to-skill-training-room | 2 | enum-like string ❓ | n |
| record | enable-change-channel | 2 | enum-like string ❓ | n, y |
| record | enable-change-faction-costume | 2 | enum-like string ❓ | n, y |
| record | enable-infinite-hyper-energy | 2 | enum-like string ❓ | y |
| record | enable-set-guild-meeting-place | 2 | enum-like string ❓ | n, y |
| record | enable-warp-to-guild-member | 2 | enum-like string ❓ | n, y |
| record | faction1-kill-quest | 1 | string |  |
| record | faction2-kill-quest | 1 | string |  |
| record | group | 2 | string |  |
| record | id | 2 | int |  |
| record | in-progress-duration-minute | 2 | int |  |
| record | main-zone | 2 | string |  |
| record | playable-time | 1 | string |  |
| record | quest-pack-simple-quest-explicit-refresh-bonus-multiple-1 | 2 | int |  |
| record | quest-pack-simple-quest-explicit-refresh-bonus-multiple-2 | 2 | int |  |
| record | quest-pack-simple-quest-explicit-refresh-bonus-multiple-3 | 2 | int |  |
| record | quest-pack-simple-quest-explicit-refresh-bonus-multiple-4 | 2 | int |  |
| record | quest-pack-simple-quest-explicit-refresh-bonus-multiple-5 | 2 | int |  |
| record | quest-pack-simple-quest-explicit-refresh-bonus-multiple-6 | 2 | int |  |
| record | quest-pack-simple-quest-explicit-refresh-bonus-prob-1 | 2 | int |  |
| record | quest-pack-simple-quest-explicit-refresh-bonus-prob-2 | 2 | int |  |
| record | quest-pack-simple-quest-explicit-refresh-bonus-prob-3 | 2 | int |  |
| record | quest-pack-simple-quest-explicit-refresh-bonus-prob-4 | 2 | int |  |
| record | quest-pack-simple-quest-explicit-refresh-bonus-prob-5 | 2 | int |  |
| record | quest-pack-simple-quest-explicit-refresh-bonus-prob-6 | 2 | int |  |
| record | quest-pack-simple-quest-implicit-refresh-bonus-multiple-1 | 2 | int |  |
| record | quest-pack-simple-quest-implicit-refresh-bonus-multiple-2 | 2 | int |  |
| record | quest-pack-simple-quest-implicit-refresh-bonus-multiple-3 | 2 | int |  |
| record | quest-pack-simple-quest-implicit-refresh-bonus-multiple-4 | 2 | int |  |
| record | quest-pack-simple-quest-implicit-refresh-bonus-multiple-5 | 2 | int |  |
| record | quest-pack-simple-quest-implicit-refresh-bonus-multiple-6 | 2 | int |  |
| record | quest-pack-simple-quest-implicit-refresh-bonus-prob-1 | 2 | int |  |
| record | quest-pack-simple-quest-implicit-refresh-bonus-prob-2 | 2 | int |  |
| record | quest-pack-simple-quest-implicit-refresh-bonus-prob-3 | 2 | int |  |
| record | quest-pack-simple-quest-implicit-refresh-bonus-prob-4 | 2 | int |  |
| record | quest-pack-simple-quest-implicit-refresh-bonus-prob-5 | 2 | int |  |
| record | quest-pack-simple-quest-implicit-refresh-bonus-prob-6 | 2 | int |  |
| record | quest-pack-simple-quest-refresh-money | 2 | int |  |
| record | reentrance-faction1-spawn | 1 | string |  |
| record | reentrance-faction2-spawn | 1 | string |  |
| record | required-activated-main-faction | 2 | enum-like string ❓ | n, y |
| record | required-level | 2 | int |  |
| record | required-preceding-quest-1 | 1 | string |  |
| record | restrict-enter-zone-when-manifest-tendency | 2 | enum-like string ❓ | n, y |
| record | stand-by-duration-minute | 2 | int |  |
| record | sub-zone-1 | 1 | string |  |
| record | sub-zone-2 | 1 | string |  |
| record | sub-zone-3 | 1 | string |  |
| record | sub-zone-4 | 1 | string |  |
| record | tendency-1-faction1-sub-quest-1 | 1 | string |  |
| record | tendency-1-faction1-sub-quest-2 | 1 | string |  |
| record | tendency-1-faction1-sub-quest-3 | 1 | string |  |
| record | tendency-1-faction1-sub-quest-4 | 1 | string |  |
| record | tendency-1-faction1-sub-quest-5 | 1 | string |  |
| record | tendency-1-faction1-warfare-boss-1 | 1 | string |  |
| record | tendency-1-faction1-warfare-boss-2 | 1 | string |  |
| record | tendency-1-faction1-warfare-boss-3 | 1 | string |  |
| record | tendency-1-faction1-warfare-boss-4 | 1 | string |  |
| record | tendency-1-faction1-warfare-boss-5 | 1 | string |  |
| record | tendency-1-faction1-warfare-quest-1 | 1 | string |  |
| record | tendency-1-faction1-warfare-quest-2 | 1 | string |  |
| record | tendency-1-faction1-warfare-quest-3 | 1 | string |  |
| record | tendency-1-faction1-warfare-quest-4 | 1 | string |  |
| record | tendency-1-faction1-warfare-quest-5 | 1 | string |  |
| record | tendency-1-faction2-sub-quest-1 | 1 | string |  |
| record | tendency-1-faction2-sub-quest-2 | 1 | string |  |
| record | tendency-1-faction2-sub-quest-3 | 1 | string |  |
| record | tendency-1-faction2-sub-quest-4 | 1 | string |  |
| record | tendency-1-faction2-sub-quest-5 | 1 | string |  |
| record | tendency-1-faction2-warfare-boss-1 | 1 | string |  |
| record | tendency-1-faction2-warfare-boss-2 | 1 | string |  |
| record | tendency-1-faction2-warfare-boss-3 | 1 | string |  |
| record | tendency-1-faction2-warfare-boss-4 | 1 | string |  |
| record | tendency-1-faction2-warfare-boss-5 | 1 | string |  |
| record | tendency-1-faction2-warfare-quest-1 | 1 | string |  |
| record | tendency-1-faction2-warfare-quest-2 | 1 | string |  |
| record | tendency-1-faction2-warfare-quest-3 | 1 | string |  |
| record | tendency-1-faction2-warfare-quest-4 | 1 | string |  |
| record | tendency-1-faction2-warfare-quest-5 | 1 | string |  |
| record | tendency-1-max-play-value | 2 | int |  |
| record | tendency-1-npc-reset-stage-finish-1 | 2 | string |  |
| record | tendency-1-npc-reset-stage-finish-2 | 1 | string |  |
| record | tendency-1-npc-reset-stage-finish-3 | 1 | string |  |
| record | tendency-1-npc-reset-stage-finish-4 | 1 | string |  |
| record | tendency-1-npc-reset-stage-finish-5 | 1 | string |  |
| record | tendency-1-npc-reset-stage-start-1 | 2 | string |  |
| record | tendency-1-npc-reset-stage-start-2 | 1 | string |  |
| record | tendency-1-npc-reset-stage-start-3 | 1 | string |  |
| record | tendency-1-npc-reset-stage-start-4 | 1 | string |  |
| record | tendency-1-npc-reset-stage-start-5 | 1 | string |  |
| record | tendency-1-reset-score-type | 2 | string |  |
| record | tendency-2-faction1-sub-quest-1 | 1 | string |  |
| record | tendency-2-faction1-sub-quest-2 | 1 | string |  |
| record | tendency-2-faction1-sub-quest-3 | 1 | string |  |
| record | tendency-2-faction1-sub-quest-4 | 1 | string |  |
| record | tendency-2-faction1-sub-quest-5 | 1 | string |  |
| record | tendency-2-faction1-warfare-boss-1 | 1 | string |  |
| record | tendency-2-faction1-warfare-boss-2 | 1 | string |  |
| record | tendency-2-faction1-warfare-boss-3 | 1 | string |  |
| record | tendency-2-faction1-warfare-boss-4 | 1 | string |  |
| record | tendency-2-faction1-warfare-boss-5 | 1 | string |  |
| record | tendency-2-faction1-warfare-quest-1 | 1 | string |  |
| record | tendency-2-faction1-warfare-quest-2 | 1 | string |  |
| record | tendency-2-faction1-warfare-quest-3 | 1 | string |  |
| record | tendency-2-faction1-warfare-quest-4 | 1 | string |  |
| record | tendency-2-faction1-warfare-quest-5 | 1 | string |  |
| record | tendency-2-faction2-sub-quest-1 | 1 | string |  |
| record | tendency-2-faction2-sub-quest-2 | 1 | string |  |
| record | tendency-2-faction2-sub-quest-3 | 1 | string |  |
| record | tendency-2-faction2-sub-quest-4 | 1 | string |  |
| record | tendency-2-faction2-sub-quest-5 | 1 | string |  |
| record | tendency-2-faction2-warfare-boss-1 | 1 | string |  |
| record | tendency-2-faction2-warfare-boss-2 | 1 | string |  |
| record | tendency-2-faction2-warfare-boss-3 | 1 | string |  |
| record | tendency-2-faction2-warfare-boss-4 | 1 | string |  |
| record | tendency-2-faction2-warfare-boss-5 | 1 | string |  |
| record | tendency-2-faction2-warfare-quest-1 | 1 | string |  |
| record | tendency-2-faction2-warfare-quest-2 | 1 | string |  |
| record | tendency-2-faction2-warfare-quest-3 | 1 | string |  |
| record | tendency-2-faction2-warfare-quest-4 | 1 | string |  |
| record | tendency-2-faction2-warfare-quest-5 | 1 | string |  |
| record | tendency-2-max-play-value | 2 | int |  |
| record | tendency-2-npc-reset-stage-finish-1 | 2 | string |  |
| record | tendency-2-npc-reset-stage-finish-2 | 1 | string |  |
| record | tendency-2-npc-reset-stage-finish-3 | 1 | string |  |
| record | tendency-2-npc-reset-stage-finish-4 | 1 | string |  |
| record | tendency-2-npc-reset-stage-finish-5 | 1 | string |  |
| record | tendency-2-npc-reset-stage-start-1 | 2 | string |  |
| record | tendency-2-npc-reset-stage-start-2 | 1 | string |  |
| record | tendency-2-npc-reset-stage-start-3 | 1 | string |  |
| record | tendency-2-npc-reset-stage-start-4 | 1 | string |  |
| record | tendency-2-npc-reset-stage-start-5 | 1 | string |  |
| record | tendency-2-reset-score-type | 2 | string |  |
| record | tendency-3-faction1-sub-quest-1 | 1 | string |  |
| record | tendency-3-faction1-sub-quest-2 | 1 | string |  |
| record | tendency-3-faction1-sub-quest-3 | 1 | string |  |
| record | tendency-3-faction1-sub-quest-4 | 1 | string |  |
| record | tendency-3-faction1-sub-quest-5 | 1 | string |  |
| record | tendency-3-faction1-warfare-quest-1 | 1 | string |  |
| record | tendency-3-faction1-warfare-quest-2 | 1 | string |  |
| record | tendency-3-faction1-warfare-quest-3 | 1 | string |  |
| record | tendency-3-faction1-warfare-quest-4 | 1 | string |  |
| record | tendency-3-faction1-warfare-quest-5 | 1 | string |  |
| record | tendency-3-faction2-sub-quest-1 | 1 | string |  |
| record | tendency-3-faction2-sub-quest-2 | 1 | string |  |
| record | tendency-3-faction2-sub-quest-3 | 1 | string |  |
| record | tendency-3-faction2-sub-quest-4 | 1 | string |  |
| record | tendency-3-faction2-sub-quest-5 | 1 | string |  |
| record | tendency-3-faction2-warfare-quest-1 | 1 | string |  |
| record | tendency-3-faction2-warfare-quest-2 | 1 | string |  |
| record | tendency-3-faction2-warfare-quest-3 | 1 | string |  |
| record | tendency-3-faction2-warfare-quest-4 | 1 | string |  |
| record | tendency-3-faction2-warfare-quest-5 | 1 | string |  |
| record | tendency-3-max-play-value | 2 | int |  |
| record | tendency-3-npc-reset-stage-finish-1 | 2 | string |  |
| record | tendency-3-npc-reset-stage-finish-2 | 1 | string |  |
| record | tendency-3-npc-reset-stage-finish-3 | 1 | string |  |
| record | tendency-3-npc-reset-stage-finish-4 | 1 | string |  |
| record | tendency-3-npc-reset-stage-finish-5 | 1 | string |  |
| record | tendency-3-npc-reset-stage-start-1 | 2 | string |  |
| record | tendency-3-npc-reset-stage-start-2 | 1 | string |  |
| record | tendency-3-npc-reset-stage-start-3 | 1 | string |  |
| record | tendency-3-npc-reset-stage-start-4 | 1 | string |  |
| record | tendency-3-npc-reset-stage-start-5 | 1 | string |  |
| record | tendency-3-reset-score-type | 2 | string |  |
| record | tendency-3-warfare-boss-1 | 1 | string |  |
| record | tendency-3-warfare-boss-2 | 1 | string |  |
| record | tendency-3-warfare-boss-3 | 1 | string |  |
| record | tendency-3-warfare-boss-4 | 1 | string |  |
| record | tendency-3-warfare-boss-5 | 1 | string |  |
| record | type | 2 | enum-like string ❓ | buyudo, normal |
| record | victory-faction-separation-faction-score | 1 | int |  |
| record | victory-npc-reset-stage-tendency-1 | 1 | string |  |
| record | victory-npc-reset-stage-tendency-2 | 1 | string |  |
| record | warfare-defeat-faction-hard-user-simple-quest-pack-play-section-1 | 1 | int |  |
| record | warfare-defeat-faction-hard-user-simple-quest-pack-play-section-2 | 1 | int |  |
| record | warfare-defeat-faction-light-user-simple-quest-pack-play-section-1 | 1 | int |  |
| record | warfare-defeat-faction-light-user-simple-quest-pack-play-section-2 | 1 | int |  |
| record | warfare-draw-simple-quest-pack-play-section-1 | 1 | int |  |
| record | warfare-draw-simple-quest-pack-play-section-2 | 1 | int |  |
| record | warfare-victory-faction-hard-user-simple-quest-pack-play-section-1 | 1 | int |  |
| record | warfare-victory-faction-hard-user-simple-quest-pack-play-section-2 | 1 | int |  |
| record | warfare-victory-faction-light-user-simple-quest-pack-play-section-1 | 1 | int |  |
| record | warfare-victory-faction-light-user-simple-quest-pack-play-section-2 | 1 | int |  |

### unlocated-store

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | store2 | 7 | string |  |

### weapon-appearance-change-cost

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | cost-apply-money | 1287 | int |  |
| record | cost-remove-money | 1287 | int |  |
| record | cost-remove-separable | 1287 | enum-like string ❓ | yes |
| record | growth-efficient-level | 1287 | int |  |
| record | item-grade | 1287 | int |  |
| record | weapon-cost-type | 1287 | string |  |

### zone

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 1533 | string |  |
| record | area | 1518 | int |  |
| record | arena-return-area-1 | 119 | string |  |
| record | available-sextet-party | 1531 | enum-like string ❓ | n, y |
| record | basecamp | 25 | string |  |
| record | chat-channel | 1474 | string |  |
| record | default-phase-pc-spawn-id | 1532 | int |  |
| record | default-phase-zone | 1533 | enum-like string ❓ | n, y |
| record | deprecated-zone | 1531 | enum-like string ❓ | n, y |
| record | district | 157 | string |  |
| record | download-level | 1483 | int |  |
| record | effect-link-type | 1533 | enum-like string ❓ | effect-blocked, effect-dungeon, effect-instant, effect-instant-higher, effect-instant-normal, effect-instant-veryhigher, effect-persistant, effect-raid-dungeon, effect-single, effect-single-hero, effect-single-higher, effect-single-normal, effect-single-veryhigher |
| record | enable-ghost-mode | 1310 | enum-like string ❓ | basic, none, phase |
| record | enable-in-out-detector | 211 | enum-like string ❓ | n, y |
| record | enter-kismet-name | 1292 | string |  |
| record | id | 1533 | int |  |
| record | is-arena-entrance | 1527 | enum-like string ❓ | n, y |
| record | leave-kismet-name | 1277 | string |  |
| record | light-client-enable | 151 | enum-like string ❓ | n, y |
| record | loadingscene | 182 | string |  |
| record | map | 1518 | string |  |
| record | max-channel | 1533 | int |  |
| record | min-channel | 321 | int |  |
| record | npc-max | 1533 | int |  |
| record | pc-max | 133 | int |  |
| record | phase-zone-1 | 393 | string |  |
| record | phase-zone-10 | 8 | string |  |
| record | phase-zone-11 | 8 | string |  |
| record | phase-zone-12 | 8 | string |  |
| record | phase-zone-13 | 1 | string |  |
| record | phase-zone-2 | 90 | string |  |
| record | phase-zone-3 | 44 | string |  |
| record | phase-zone-4 | 24 | string |  |
| record | phase-zone-5 | 17 | string |  |
| record | phase-zone-6 | 13 | string |  |
| record | phase-zone-7 | 10 | string |  |
| record | phase-zone-8 | 10 | string |  |
| record | phase-zone-9 | 8 | string |  |
| record | recharge-exp-boost | 1533 | enum-like string ❓ | n, y |
| record | recommend-pc-count | 1533 | int |  |
| record | reentrance-pcspawn | 1090 | string |  |
| record | reentrance-pcspawn-in-arena | 243 | string |  |
| record | region | 1533 | int |  |
| record | respawn | 1027 | string |  |
| record | respawn-by-activated-faction-1 | 1 | string |  |
| record | respawn-by-activated-faction-2 | 1 | string |  |
| record | respawn-by-activated-faction-required-faction-1 | 1 | string |  |
| record | respawn-by-activated-faction-required-faction-2 | 1 | string |  |
| record | script | 1533 | string |  |
| record | sector-xmax | 1533 | int |  |
| record | sector-xmin | 1533 | int |  |
| record | sector-ymax | 1533 | int |  |
| record | sector-ymin | 1533 | int |  |
| record | sharable-combatmode | 1533 | enum-like string ❓ | n, y |
| record | sight-cube | 1533 | enum-like string ❓ | n, y |
| record | terrain | 1533 | int |  |
| record | transfer-pcspawn-by-deprecated | 79 | string |  |
| record | tutorial | 1533 | enum-like string ❓ | n, y |
| record | write-equipinfo-log | 1531 | enum-like string ❓ | n, y |
| record | zone-type | 546 | enum-like string ❓ | duel, dungeon, instant, none, persistant, single, single-dungeon |
| record | zone-type2 | 1533 | enum-like string ❓ | instant, none, persistent, single |

### zone-pc-random-spawn

| Element path | Attribute | Present count | Inferred type | Enum-like names (at most 20) |
| --- | --- | ---: | --- | --- |
| record | alias | 1 | string |  |
| record | pc-spawn-1 | 1 | string |  |
| record | pc-spawn-2 | 1 | string |  |
| record | pc-spawn-3 | 1 | string |  |
| record | pc-spawn-4 | 1 | string |  |
| record | pc-spawn-5 | 1 | string |  |
| record | zone | 1 | int |  |

## Cross-file reference candidates

❓ These are byte-derived candidates, not confirmed runtime links. Normalize an attribute name by removing nonletters; match a table name, with optional id/alias suffix. Count nonempty occurrences, and exact id/alias matches across all files of that target table. Zero and empty defaults are not resolved here. Other fields can contain references that this name rule misses.

| Source table | Attribute | Candidate target | Nonempty occurrences | Id matches | Alias matches |
| --- | --- | --- | ---: | ---: | ---: |
| ArenaMatchingRule | level | level | 360 | 0 | 0 |
| BossNpc | difficulty-type-modify | difficulty-type-modify | 105 | 0 | 101 |
| BotScoringLevel | level | level | 5 | 0 | 0 |
| ChallengeList | reward-1 | reward | 17 | 0 | 0 |
| ChallengeList | reward-2 | reward | 17 | 0 | 0 |
| ChallengeList | reward-3 | reward | 17 | 0 | 0 |
| ChallengeList | reward-4 | reward | 10 | 0 | 0 |
| DuelMatchingRule | level | level | 100 | 0 | 0 |
| DuelMemberDisplaySkill | job-style | job-style | 63 | 0 | 0 |
| DuelMemberDisplaySkill | skill-id-1 | skill3 | 63 | 63 | 0 |
| DuelMemberDisplaySkill | skill-id-2 | skill3 | 63 | 63 | 0 |
| DuelMemberDisplaySkill | skill-id-3 | skill3 | 63 | 63 | 0 |
| DuelMemberDisplaySkill | skill-id-4 | skill3 | 63 | 63 | 0 |
| FielditemDrop | fielditem-1 | fielditem | 1000 | 0 | 998 |
| FielditemDrop | fielditem-2 | fielditem | 49 | 0 | 49 |
| FielditemDrop | fielditem-3 | fielditem | 33 | 0 | 33 |
| FielditemDrop | quest-id | quest | 31 | 31 | 0 |
| Fishing-Paste | item | item | 7 | 0 | 7 |
| GradeBenefits | effect | effect | 20 | 0 | 20 |
| GradeBenefits | level | level | 130 | 0 | 0 |
| InterDungeonList | cave | cave | 7 | 0 | 0 |
| InterDungeonList | cave | cave2 | 7 | 0 | 7 |
| InterDungeonList | dungeon | dungeon | 42 | 0 | 41 |
| InterDungeonList | public-raid | public-raid | 6 | 0 | 6 |
| ItemGrowth | level | level | 4120 | 0 | 0 |
| ItemSkill | skill-id | skill3 | 8885 | 8885 | 0 |
| ItemSpirit | level | level | 58 | 0 | 0 |
| ItemTransformRecipe | weapon-gem-transform-result | WeaponGemTransformResult | 72 | 72 | 0 |
| ItemUsableRelation | attraction-group | attraction-group | 74 | 0 | 73 |
| ItemUsableRelation | duel | duel | 74 | 0 | 0 |
| JumpingCharacter | job | job | 13 | 0 | 0 |
| JumpingQuest | job | job | 2520 | 0 | 0 |
| JumpingQuest | quest-id | quest | 2520 | 2520 | 0 |
| JumpingQuest2 | job | job | 1008 | 0 | 0 |
| JumpingQuest2 | quest-id | quest | 1008 | 1008 | 0 |
| JumpingTeleport | job | job | 1254 | 0 | 0 |
| JumpingTeleport | teleport-id | teleport | 1254 | 1254 | 0 |
| JumpingTeleport2 | teleport | teleport | 132 | 132 | 0 |
| PcCafeBenefits | effect | effect | 12 | 0 | 12 |
| PcCafeBenefits | job | job | 12 | 0 | 0 |
| PcCafeBenefits | quest-reward | QuestReward | 12 | 0 | 0 |
| PcInitialSetting | job | job | 93 | 0 | 0 |
| PcInitialSetting | level | level | 93 | 0 | 0 |
| PcInitialSetting | quest-1 | quest | 93 | 0 | 93 |
| PcInitialSetting | zone | zone | 93 | 0 | 93 |
| PcSkill3 | job-style | job-style | 1258 | 0 | 0 |
| PcSkill3 | skill3-1 | skill3 | 1032 | 0 | 976 |
| PcSkill3 | skill3-10 | skill3 | 9 | 0 | 8 |
| PcSkill3 | skill3-11 | skill3 | 8 | 0 | 7 |
| PcSkill3 | skill3-12 | skill3 | 8 | 0 | 7 |
| PcSkill3 | skill3-13 | skill3 | 7 | 0 | 7 |
| PcSkill3 | skill3-14 | skill3 | 6 | 0 | 6 |
| PcSkill3 | skill3-15 | skill3 | 5 | 0 | 5 |
| PcSkill3 | skill3-16 | skill3 | 5 | 0 | 5 |
| PcSkill3 | skill3-17 | skill3 | 4 | 0 | 4 |
| PcSkill3 | skill3-18 | skill3 | 4 | 0 | 4 |
| PcSkill3 | skill3-19 | skill3 | 4 | 0 | 4 |
| PcSkill3 | skill3-2 | skill3 | 763 | 0 | 713 |
| PcSkill3 | skill3-20 | skill3 | 4 | 0 | 4 |
| PcSkill3 | skill3-21 | skill3 | 4 | 0 | 4 |
| PcSkill3 | skill3-22 | skill3 | 4 | 0 | 4 |
| PcSkill3 | skill3-23 | skill3 | 4 | 0 | 4 |
| PcSkill3 | skill3-24 | skill3 | 4 | 0 | 4 |
| PcSkill3 | skill3-25 | skill3 | 4 | 0 | 4 |
| PcSkill3 | skill3-26 | skill3 | 2 | 0 | 2 |
| PcSkill3 | skill3-27 | skill3 | 2 | 0 | 2 |
| PcSkill3 | skill3-28 | skill3 | 2 | 0 | 2 |
| PcSkill3 | skill3-29 | skill3 | 2 | 0 | 2 |
| PcSkill3 | skill3-3 | skill3 | 303 | 0 | 283 |
| PcSkill3 | skill3-30 | skill3 | 2 | 0 | 2 |
| PcSkill3 | skill3-31 | skill3 | 1 | 0 | 1 |
| PcSkill3 | skill3-32 | skill3 | 1 | 0 | 1 |
| PcSkill3 | skill3-33 | skill3 | 1 | 0 | 1 |
| PcSkill3 | skill3-4 | skill3 | 171 | 0 | 160 |
| PcSkill3 | skill3-5 | skill3 | 85 | 0 | 78 |
| PcSkill3 | skill3-6 | skill3 | 44 | 0 | 41 |
| PcSkill3 | skill3-7 | skill3 | 24 | 0 | 22 |
| PcSkill3 | skill3-8 | skill3 | 23 | 0 | 21 |
| PcSkill3 | skill3-9 | skill3 | 15 | 0 | 11 |
| QuestRewardSkill3 | job-style | job-style | 235 | 0 | 0 |
| QuestRewardSkill3 | skill3-1 | skill3 | 227 | 0 | 196 |
| QuestRewardSkill3 | skill3-2 | skill3 | 138 | 0 | 124 |
| QuestRewardSkill3 | skill3-3 | skill3 | 92 | 0 | 92 |
| QuestRewardSkill3 | skill3-4 | skill3 | 40 | 0 | 40 |
| SkillCastCondition3 | soul-mask-1 | SoulMask | 471 | 0 | 407 |
| SkillCastCondition3 | soul-mask-2 | SoulMask | 314 | 0 | 187 |
| SkillCastCondition3 | stance | stance | 1415 | 0 | 0 |
| SoulMask | effect-1 | effect | 13 | 0 | 13 |
| SoulMask | effect-2 | effect | 7 | 0 | 7 |
| SoulMaskSet | soul-mask-id-1 | SoulMask | 32 | 0 | 0 |
| SoulMaskSet | soul-mask-id-2 | SoulMask | 32 | 0 | 0 |
| SummonedMasteryLevel | job | job | 560 | 0 | 0 |
| SummonedMasteryLevel | mastery-level | mastery-level | 560 | 0 | 0 |
| TrainSkillVersion | job | job | 14 | 0 | 0 |
| WeaponGemSlot | level | level | 5 | 0 | 0 |
| WeaponGemTransformResult | item | item | 325 | 0 | 325 |
| ZoneArea | zone | zone | 76936 | 76936 | 0 |
| ZoneAreaGroup | zone | zone | 160 | 160 | 0 |
| ZoneBaseCamp | zone | zone | 68 | 68 | 0 |
| ZoneCampfire | campfire | campfire | 466 | 0 | 465 |
| ZoneCampfire | zone | zone | 466 | 466 | 0 |
| ZoneCellAttribute | zone | zone | 4 | 4 | 0 |
| ZoneConvoy | zone | zone | 39 | 39 | 0 |
| ZoneEnv | effect-1 | effect | 14 | 0 | 14 |
| ZoneEnv | reward | reward | 36 | 0 | 36 |
| ZoneEnv | script | script | 93 | 0 | 39 |
| ZoneEnv | zone | zone | 98 | 98 | 0 |
| ZoneEnv2 | effect-1 | effect | 395 | 0 | 395 |
| ZoneEnv2 | fielditemdrop | FielditemDrop | 497 | 0 | 481 |
| ZoneEnv2 | fishing-field | fishing-field | 8 | 0 | 8 |
| ZoneEnv2 | reward | reward | 407 | 0 | 405 |
| ZoneEnv2 | script | script | 8151 | 0 | 2581 |
| ZoneEnv2 | social | social | 1 | 0 | 0 |
| ZoneEnv2Spawn | zone | zone | 11832 | 11779 | 0 |
| ZoneNpcRandomSpawn | act-sequence | act-sequence | 159 | 0 | 159 |
| ZoneNpcRandomSpawn | npc-1 | npc | 159 | 0 | 159 |
| ZoneNpcRandomSpawn | npc-2 | npc | 101 | 0 | 101 |
| ZoneNpcRandomSpawn | npc-3 | npc | 91 | 0 | 91 |
| ZoneNpcRandomSpawn | npc-4 | npc | 65 | 0 | 65 |
| ZoneNpcRandomSpawn | zone | zone | 159 | 159 | 0 |
| ZoneNpcSpawn | act-sequence | act-sequence | 68454 | 0 | 65236 |
| ZoneNpcSpawn | npc | npc | 68455 | 0 | 68270 |
| ZoneNpcSpawn | zone | zone | 68455 | 68455 | 0 |
| ZoneNpcSpawnEvent | zone | zone | 1 | 1 | 0 |
| ZoneNpcSquad | zone | zone | 10795 | 10795 | 0 |
| ZonePathway | zone | zone | 77786 | 77786 | 0 |
| ZonePcSpawn | zone | zone | 2951 | 2951 | 0 |
| ZoneRespawn | zone | zone | 1652 | 1652 | 0 |
| ZoneRule | zone | zone | 17 | 17 | 0 |
| ZoneTeleportPosition | zone | zone | 156 | 156 | 0 |
| ZoneTeleportSwitch | zone | zone | 190 | 190 | 0 |
| ZoneTriggerEventStage | zone | zone | 16 | 16 | 0 |
| achievement | job | job | 1558 | 0 | 0 |
| achievement-register | boss-npc | BossNpc | 202 | 0 | 0 |
| achievement-register | item | item | 1 | 0 | 1 |
| achievement-register | item-brand | item-brand | 34 | 0 | 34 |
| achievement-register | job | job | 1067 | 0 | 0 |
| achievement-register | npc | npc | 16 | 0 | 16 |
| achievement-register | npc-1 | npc | 84 | 0 | 82 |
| achievement-register | npc-2 | npc | 21 | 0 | 20 |
| achievement-register | npc-3 | npc | 5 | 0 | 5 |
| achievement-register | npc-4 | npc | 4 | 0 | 4 |
| achievement-register | npc-5 | npc | 1 | 0 | 1 |
| achievement-register | quest | quest | 53 | 0 | 53 |
| achievement-register | skill-training-room-group | skill-training-room-group | 96 | 96 | 0 |
| act-sequence | social | social | 8910 | 0 | 7808 |
| badge-set-item | effect-1 | effect | 384 | 0 | 384 |
| badge-set-item | effect-2 | effect | 313 | 0 | 313 |
| battle-royal-field | public-raid | public-raid | 1 | 0 | 1 |
| battle-royal-field | zone | zone | 1 | 1 | 0 |
| battle-royal-field-effect-pouch | effect-1 | effect | 25 | 0 | 11 |
| battle-royal-field-effect-pouch | effect-2 | effect | 25 | 0 | 10 |
| battle-royal-field-effect-pouch | effect-3 | effect | 21 | 0 | 9 |
| boss-challenge | zone | zone | 1 | 1 | 0 |
| boss-challenge-round | boss-npc | BossNpc | 6 | 0 | 0 |
| campfire | party-match | partymatch | 101 | 0 | 92 |
| cave | zone-1 | zone | 18 | 18 | 0 |
| cave | zone-10 | zone | 4 | 4 | 0 |
| cave | zone-11 | zone | 2 | 2 | 0 |
| cave | zone-12 | zone | 2 | 2 | 0 |
| cave | zone-13 | zone | 2 | 2 | 0 |
| cave | zone-14 | zone | 2 | 2 | 0 |
| cave | zone-15 | zone | 2 | 2 | 0 |
| cave | zone-16 | zone | 2 | 2 | 0 |
| cave | zone-17 | zone | 2 | 2 | 0 |
| cave | zone-18 | zone | 2 | 2 | 0 |
| cave | zone-19 | zone | 2 | 2 | 0 |
| cave | zone-2 | zone | 13 | 13 | 0 |
| cave | zone-20 | zone | 2 | 2 | 0 |
| cave | zone-21 | zone | 2 | 2 | 0 |
| cave | zone-22 | zone | 2 | 2 | 0 |
| cave | zone-23 | zone | 2 | 2 | 0 |
| cave | zone-24 | zone | 2 | 2 | 0 |
| cave | zone-25 | zone | 2 | 2 | 0 |
| cave | zone-26 | zone | 2 | 2 | 0 |
| cave | zone-27 | zone | 2 | 2 | 0 |
| cave | zone-28 | zone | 2 | 2 | 0 |
| cave | zone-29 | zone | 2 | 2 | 0 |
| cave | zone-3 | zone | 13 | 13 | 0 |
| cave | zone-30 | zone | 2 | 2 | 0 |
| cave | zone-31 | zone | 2 | 2 | 0 |
| cave | zone-32 | zone | 2 | 2 | 0 |
| cave | zone-33 | zone | 1 | 1 | 0 |
| cave | zone-34 | zone | 1 | 1 | 0 |
| cave | zone-35 | zone | 1 | 1 | 0 |
| cave | zone-36 | zone | 1 | 1 | 0 |
| cave | zone-37 | zone | 1 | 1 | 0 |
| cave | zone-38 | zone | 1 | 1 | 0 |
| cave | zone-4 | zone | 11 | 11 | 0 |
| cave | zone-5 | zone | 11 | 11 | 0 |
| cave | zone-6 | zone | 11 | 11 | 0 |
| cave | zone-7 | zone | 9 | 9 | 0 |
| cave | zone-8 | zone | 8 | 8 | 0 |
| cave | zone-9 | zone | 5 | 5 | 0 |
| cave2 | zone-1 | zone | 710 | 699 | 0 |
| cave2 | zone-10 | zone | 6 | 6 | 0 |
| cave2 | zone-11 | zone | 6 | 6 | 0 |
| cave2 | zone-12 | zone | 6 | 6 | 0 |
| cave2 | zone-13 | zone | 1 | 1 | 0 |
| cave2 | zone-2 | zone | 26 | 25 | 0 |
| cave2 | zone-3 | zone | 9 | 9 | 0 |
| cave2 | zone-4 | zone | 7 | 7 | 0 |
| cave2 | zone-5 | zone | 6 | 6 | 0 |
| cave2 | zone-6 | zone | 6 | 6 | 0 |
| cave2 | zone-7 | zone | 6 | 6 | 0 |
| cave2 | zone-8 | zone | 6 | 6 | 0 |
| cave2 | zone-9 | zone | 6 | 6 | 0 |
| challenge-party | boss-npc | BossNpc | 1 | 0 | 0 |
| challenge-party | zone | zone | 1 | 0 | 1 |
| classic-field-zone | zone-1 | zone | 12 | 12 | 0 |
| classic-field-zone | zone-2 | zone | 3 | 3 | 0 |
| combat-sequence | skill | skill3 | 33591 | 623 | 31409 |
| combat-sequence | skill-1 | skill3 | 24 | 0 | 24 |
| combat-sequence | skill-2 | skill3 | 24 | 0 | 24 |
| combat-sequence | skill-3 | skill3 | 24 | 0 | 24 |
| combat-sequence | skill3 | skill3 | 74828 | 623 | 72013 |
| combat-sequence | skill3-1 | skill3 | 107 | 0 | 107 |
| combat-sequence | skill3-10 | skill3 | 13 | 0 | 13 |
| combat-sequence | skill3-2 | skill3 | 107 | 0 | 107 |
| combat-sequence | skill3-3 | skill3 | 107 | 0 | 107 |
| combat-sequence | skill3-4 | skill3 | 81 | 0 | 80 |
| combat-sequence | skill3-5 | skill3 | 51 | 0 | 50 |
| combat-sequence | skill3-6 | skill3 | 51 | 0 | 50 |
| combat-sequence | skill3-7 | skill3 | 42 | 0 | 41 |
| combat-sequence | skill3-8 | skill3 | 42 | 0 | 41 |
| combat-sequence | skill3-9 | skill3 | 13 | 0 | 12 |
| combat-sequence | social | social | 13055 | 0 | 11871 |
| combat-sequence | stance | stance | 809 | 0 | 0 |
| duel | effect | effect | 18 | 0 | 18 |
| duel | zone | zone | 36 | 36 | 0 |
| duel-bot | job | job | 315 | 0 | 0 |
| duel-bot | job-style | job-style | 315 | 0 | 0 |
| duel-bot | level | level | 315 | 0 | 0 |
| duel-bot | mastery-level | mastery-level | 315 | 0 | 0 |
| duel-bot-challenge | zone | zone | 1 | 1 | 0 |
| duel-bot-group | duel-bot-1 | duel-bot | 29 | 0 | 29 |
| duel-bot-group | duel-bot-10 | duel-bot | 9 | 0 | 9 |
| duel-bot-group | duel-bot-2 | duel-bot | 29 | 0 | 29 |
| duel-bot-group | duel-bot-3 | duel-bot | 29 | 0 | 29 |
| duel-bot-group | duel-bot-4 | duel-bot | 29 | 0 | 29 |
| duel-bot-group | duel-bot-5 | duel-bot | 29 | 0 | 29 |
| duel-bot-group | duel-bot-6 | duel-bot | 29 | 0 | 9 |
| duel-bot-group | duel-bot-7 | duel-bot | 29 | 0 | 29 |
| duel-bot-group | duel-bot-8 | duel-bot | 29 | 0 | 29 |
| duel-bot-group | duel-bot-9 | duel-bot | 23 | 0 | 23 |
| duel-bot-training-room | zone | zone | 1 | 1 | 0 |
| duel-bot-training-room-reward | achievement | achievement | 270 | 270 | 0 |
| duel-bot-training-room-reward | job | job | 270 | 0 | 0 |
| duel-zone | zone-1 | zone | 1 | 1 | 0 |
| duel-zone | zone-2 | zone | 1 | 1 | 0 |
| dungeon | zone-1 | zone | 51 | 0 | 44 |
| dungeon | zone-2 | zone | 34 | 0 | 30 |
| dungeon | zone-3 | zone | 20 | 0 | 16 |
| dungeon | zone-4 | zone | 9 | 0 | 9 |
| dungeon | zone-5 | zone | 4 | 0 | 4 |
| dungeon | zone-6 | zone | 2 | 0 | 2 |
| dungeon | zone-7 | zone | 2 | 0 | 2 |
| dungeon | zone-8 | zone | 2 | 0 | 2 |
| dungeon | zone-9 | zone | 1 | 0 | 1 |
| effect | content-quota-id | content-quota | 2 | 2 | 0 |
| effect | level | level | 82765 | 0 | 0 |
| effect | skill3 | skill3 | 43 | 0 | 13 |
| effect | soul-mask-id | SoulMask | 22 | 0 | 0 |
| effect | stance | stance | 53042 | 0 | 0 |
| effect | zone-id | zone | 34061 | 9 | 0 |
| effect-group | effect-1 | effect | 38786 | 38 | 38344 |
| effect-group | effect-2 | effect | 10516 | 4 | 10401 |
| effect-group | effect-3 | effect | 4856 | 2 | 4766 |
| effect-group | effect-4 | effect | 1946 | 0 | 1928 |
| effect-group | effect-5 | effect | 819 | 0 | 815 |
| effect-group | effect-6 | effect | 340 | 0 | 338 |
| effect-group | effect-7 | effect | 197 | 0 | 190 |
| effect-group | effect-8 | effect | 93 | 0 | 93 |
| event-skill | skill-1-1 | skill3 | 1 | 0 | 0 |
| event-skill | skill-1-2 | skill3 | 13 | 0 | 0 |
| event-skill | skill-2-2 | skill3 | 1 | 0 | 0 |
| event-skill | skill-2-4 | skill3 | 1 | 0 | 0 |
| event-skill | skill-2-8 | skill3 | 1 | 0 | 0 |
| expand-inventory | level | level | 45 | 0 | 0 |
| faction-battle-field-zone | zone | zone | 1 | 1 | 0 |
| faction-level | level | level | 16 | 0 | 0 |
| field-zone | zone-1 | zone | 5 | 5 | 0 |
| field-zone | zone-2 | zone | 2 | 2 | 0 |
| field-zone | zone-3 | zone | 2 | 2 | 0 |
| field-zone | zone-4 | zone | 1 | 1 | 0 |
| filter | job-1 | job | 96 | 0 | 0 |
| filter | job-2 | job | 96 | 0 | 0 |
| filter | job-3 | job | 24 | 0 | 0 |
| filter | job-style-1 | job-style | 96 | 0 | 0 |
| filter | job-style-2 | job-style | 96 | 0 | 0 |
| filter | job-style-3 | job-style | 24 | 0 | 0 |
| friend-noti-location | zone1-1 | zone | 324 | 324 | 0 |
| friend-noti-location | zone1-10 | zone | 21 | 21 | 0 |
| friend-noti-location | zone1-11 | zone | 19 | 19 | 0 |
| friend-noti-location | zone1-12 | zone | 17 | 17 | 0 |
| friend-noti-location | zone1-13 | zone | 15 | 15 | 0 |
| friend-noti-location | zone1-14 | zone | 14 | 14 | 0 |
| friend-noti-location | zone1-15 | zone | 13 | 13 | 0 |
| friend-noti-location | zone1-16 | zone | 8 | 8 | 0 |
| friend-noti-location | zone1-17 | zone | 7 | 7 | 0 |
| friend-noti-location | zone1-18 | zone | 7 | 7 | 0 |
| friend-noti-location | zone1-19 | zone | 7 | 7 | 0 |
| friend-noti-location | zone1-2 | zone | 232 | 232 | 0 |
| friend-noti-location | zone1-20 | zone | 7 | 7 | 0 |
| friend-noti-location | zone1-21 | zone | 7 | 7 | 0 |
| friend-noti-location | zone1-22 | zone | 6 | 6 | 0 |
| friend-noti-location | zone1-23 | zone | 6 | 6 | 0 |
| friend-noti-location | zone1-24 | zone | 6 | 6 | 0 |
| friend-noti-location | zone1-25 | zone | 6 | 6 | 0 |
| friend-noti-location | zone1-26 | zone | 6 | 6 | 0 |
| friend-noti-location | zone1-27 | zone | 5 | 5 | 0 |
| friend-noti-location | zone1-28 | zone | 5 | 5 | 0 |
| friend-noti-location | zone1-29 | zone | 5 | 5 | 0 |
| friend-noti-location | zone1-3 | zone | 119 | 119 | 0 |
| friend-noti-location | zone1-30 | zone | 5 | 5 | 0 |
| friend-noti-location | zone1-31 | zone | 4 | 4 | 0 |
| friend-noti-location | zone1-32 | zone | 4 | 4 | 0 |
| friend-noti-location | zone1-33 | zone | 4 | 4 | 0 |
| friend-noti-location | zone1-34 | zone | 4 | 4 | 0 |
| friend-noti-location | zone1-35 | zone | 4 | 4 | 0 |
| friend-noti-location | zone1-36 | zone | 4 | 4 | 0 |
| friend-noti-location | zone1-37 | zone | 3 | 3 | 0 |
| friend-noti-location | zone1-38 | zone | 3 | 3 | 0 |
| friend-noti-location | zone1-39 | zone | 3 | 3 | 0 |
| friend-noti-location | zone1-4 | zone | 92 | 91 | 0 |
| friend-noti-location | zone1-40 | zone | 2 | 2 | 0 |
| friend-noti-location | zone1-41 | zone | 2 | 2 | 0 |
| friend-noti-location | zone1-42 | zone | 2 | 2 | 0 |
| friend-noti-location | zone1-43 | zone | 2 | 2 | 0 |
| friend-noti-location | zone1-44 | zone | 2 | 2 | 0 |
| friend-noti-location | zone1-45 | zone | 2 | 2 | 0 |
| friend-noti-location | zone1-46 | zone | 2 | 2 | 0 |
| friend-noti-location | zone1-47 | zone | 2 | 2 | 0 |
| friend-noti-location | zone1-48 | zone | 2 | 2 | 0 |
| friend-noti-location | zone1-49 | zone | 2 | 2 | 0 |
| friend-noti-location | zone1-5 | zone | 61 | 61 | 0 |
| friend-noti-location | zone1-50 | zone | 2 | 2 | 0 |
| friend-noti-location | zone1-51 | zone | 2 | 2 | 0 |
| friend-noti-location | zone1-52 | zone | 2 | 2 | 0 |
| friend-noti-location | zone1-53 | zone | 2 | 2 | 0 |
| friend-noti-location | zone1-54 | zone | 2 | 2 | 0 |
| friend-noti-location | zone1-55 | zone | 2 | 2 | 0 |
| friend-noti-location | zone1-56 | zone | 2 | 2 | 0 |
| friend-noti-location | zone1-57 | zone | 2 | 2 | 0 |
| friend-noti-location | zone1-58 | zone | 2 | 2 | 0 |
| friend-noti-location | zone1-59 | zone | 2 | 2 | 0 |
| friend-noti-location | zone1-6 | zone | 51 | 51 | 0 |
| friend-noti-location | zone1-60 | zone | 2 | 2 | 0 |
| friend-noti-location | zone1-61 | zone | 2 | 2 | 0 |
| friend-noti-location | zone1-62 | zone | 1 | 1 | 0 |
| friend-noti-location | zone1-63 | zone | 1 | 1 | 0 |
| friend-noti-location | zone1-7 | zone | 39 | 39 | 0 |
| friend-noti-location | zone1-8 | zone | 34 | 34 | 0 |
| friend-noti-location | zone1-9 | zone | 31 | 31 | 0 |
| guild-discount | item-growth | ItemGrowth | 5 | 5 | 0 |
| guildlevel | guild-discount | guild-discount | 16 | 0 | 10 |
| guildlevel | level | level | 16 | 0 | 0 |
| item | closet-group-id | closet-group | 3227 | 3082 | 0 |
| item | faction | faction | 21 | 0 | 21 |
| item | item-skill-1 | ItemSkill | 15030 | 0 | 14512 |
| item | item-skill-10 | ItemSkill | 15002 | 0 | 3761 |
| item | item-skill-11 | ItemSkill | 14982 | 0 | 2994 |
| item | item-skill-12 | ItemSkill | 14982 | 0 | 2981 |
| item | item-skill-13 | ItemSkill | 14982 | 0 | 2074 |
| item | item-skill-14 | ItemSkill | 14982 | 0 | 1944 |
| item | item-skill-15 | ItemSkill | 14982 | 0 | 1537 |
| item | item-skill-16 | ItemSkill | 14982 | 0 | 1514 |
| item | item-skill-2 | ItemSkill | 15026 | 0 | 13743 |
| item | item-skill-3 | ItemSkill | 15026 | 0 | 12338 |
| item | item-skill-4 | ItemSkill | 15026 | 0 | 11062 |
| item | item-skill-5 | ItemSkill | 15006 | 0 | 10079 |
| item | item-skill-6 | ItemSkill | 15006 | 0 | 9561 |
| item | item-skill-7 | ItemSkill | 15002 | 0 | 8488 |
| item | item-skill-8 | ItemSkill | 15002 | 0 | 7549 |
| item | item-skill-9 | ItemSkill | 15002 | 0 | 4000 |
| item | item-usable-group | item-usable-group | 34474 | 0 | 34474 |
| item | level | level | 144241 | 0 | 0 |
| item | pet-food-recovery | pet-food-recovery | 2 | 0 | 2 |
| item | set-item | set-item | 25130 | 0 | 24810 |
| item | skill-by-equipment | skill-by-equipment | 1 | 0 | 1 |
| item | skill-modify-info-group-1 | skill-modify-info-group | 10890 | 0 | 10890 |
| item | skill-modify-info-group-2 | skill-modify-info-group | 10890 | 0 | 10180 |
| item | skill-modify-info-group-3 | skill-modify-info-group | 8973 | 0 | 216 |
| item | skill-modify-info-group-6 | skill-modify-info-group | 10890 | 0 | 10395 |
| item | skill-modify-info-group-7 | skill-modify-info-group | 10890 | 0 | 9685 |
| item | skill-modify-info-group-8 | skill-modify-info-group | 8973 | 0 | 3749 |
| item | skill-skin | SkillSkin | 13 | 0 | 0 |
| item | skill3 | skill3 | 712 | 0 | 706 |
| item | slate-scroll | SlateScroll | 18 | 0 | 18 |
| item-buy-price | faction-level | faction-level | 146 | 0 | 0 |
| jackpot-boss-zone | zone | zone | 1 | 1 | 0 |
| job | job | job | 14 | 0 | 0 |
| job-style | job | job | 64 | 0 | 0 |
| job-style | job-style | job-style | 64 | 0 | 0 |
| level | level | level | 60 | 0 | 0 |
| mastery-ability | job | job | 1200 | 0 | 0 |
| mastery-ability | mastery-level | mastery-level | 1200 | 0 | 0 |
| mastery-level | mastery-level | mastery-level | 50 | 0 | 0 |
| mastery-stat-point-pick | effect | effect | 325 | 0 | 325 |
| mysterious-store-item-group | item-1 | item | 28 | 0 | 28 |
| mysterious-store-item-group | item-2 | item | 28 | 0 | 28 |
| mysterious-store-item-group | item-3 | item | 28 | 0 | 28 |
| mysterious-store-item-group | item-4 | item | 28 | 0 | 28 |
| mysterious-store-item-group | item-5 | item | 28 | 0 | 28 |
| mysterious-store-item-group | item-6 | item | 28 | 0 | 28 |
| mysterious-store-item-group | item-7 | item | 28 | 0 | 28 |
| mysterious-store-item-group | item-8 | item | 28 | 0 | 28 |
| newbie-care | achievement-register | achievement-register | 6 | 0 | 6 |
| npc | boss-npc | BossNpc | 2717 | 0 | 2327 |
| npc | contribution-reward | ContributionReward | 221 | 0 | 158 |
| npc | faction | faction | 1599 | 1599 | 0 |
| npc | fielditemdrop | FielditemDrop | 3456 | 0 | 3315 |
| npc | level | level | 34434 | 0 | 0 |
| npc | mastery-level | mastery-level | 34434 | 0 | 0 |
| npc | mysterious-store | mysterious-store | 2 | 0 | 2 |
| npc | stance | stance | 34434 | 0 | 0 |
| npc | store2-1 | store | 525 | 0 | 138 |
| npc | store2-1 | store2 | 525 | 0 | 470 |
| npc | store2-2 | store | 149 | 0 | 0 |
| npc | store2-2 | store2 | 149 | 0 | 149 |
| npc | store2-3 | store | 44 | 0 | 0 |
| npc | store2-3 | store2 | 44 | 0 | 44 |
| npc | store2-4 | store | 29 | 0 | 0 |
| npc | store2-4 | store2 | 29 | 0 | 29 |
| npc | store2-5 | store | 4 | 0 | 0 |
| npc | store2-5 | store2 | 4 | 0 | 4 |
| npc | store2-6 | store | 1 | 0 | 0 |
| npc | store2-6 | store2 | 1 | 0 | 1 |
| npc | summoned-beauty-shop | SummonedBeautyShop | 8 | 0 | 8 |
| npc | teleport | teleport | 1 | 0 | 0 |
| npc-brain-parameters | combat-sequence-1 | combat-sequence | 34446 | 0 | 32093 |
| npc-brain-parameters | combat-sequence-10 | combat-sequence | 218 | 0 | 98 |
| npc-brain-parameters | combat-sequence-2 | combat-sequence | 1487 | 0 | 894 |
| npc-brain-parameters | combat-sequence-3 | combat-sequence | 770 | 0 | 441 |
| npc-brain-parameters | combat-sequence-4 | combat-sequence | 620 | 0 | 320 |
| npc-brain-parameters | combat-sequence-5 | combat-sequence | 428 | 0 | 249 |
| npc-brain-parameters | combat-sequence-6 | combat-sequence | 358 | 0 | 193 |
| npc-brain-parameters | combat-sequence-7 | combat-sequence | 326 | 0 | 172 |
| npc-brain-parameters | combat-sequence-8 | combat-sequence | 280 | 0 | 145 |
| npc-brain-parameters | combat-sequence-9 | combat-sequence | 248 | 0 | 124 |
| npc-brain-parameters | script | script | 34446 | 0 | 25963 |
| npc-brain-parameters | skill-1 | skill3 | 6004 | 0 | 5994 |
| npc-brain-parameters | skill-10 | skill3 | 632 | 0 | 632 |
| npc-brain-parameters | skill-11 | skill3 | 498 | 0 | 498 |
| npc-brain-parameters | skill-12 | skill3 | 344 | 0 | 344 |
| npc-brain-parameters | skill-14 | skill3 | 1971 | 0 | 1961 |
| npc-brain-parameters | skill-15 | skill3 | 1548 | 0 | 1548 |
| npc-brain-parameters | skill-16 | skill3 | 1348 | 0 | 1348 |
| npc-brain-parameters | skill-17 | skill3 | 924 | 0 | 924 |
| npc-brain-parameters | skill-18 | skill3 | 39 | 0 | 39 |
| npc-brain-parameters | skill-19 | skill3 | 31 | 0 | 31 |
| npc-brain-parameters | skill-2 | skill3 | 5640 | 0 | 5629 |
| npc-brain-parameters | skill-3 | skill3 | 4291 | 0 | 4280 |
| npc-brain-parameters | skill-4 | skill3 | 2795 | 0 | 2795 |
| npc-brain-parameters | skill-5 | skill3 | 1041 | 0 | 1041 |
| npc-brain-parameters | skill-6 | skill3 | 35 | 0 | 35 |
| npc-brain-parameters | skill-7 | skill3 | 17 | 0 | 17 |
| npc-brain-parameters | skill-8 | skill3 | 3872 | 0 | 3848 |
| npc-brain-parameters | skill-9 | skill3 | 1019 | 0 | 1019 |
| npc-brain-parameters | skill3-1 | skill3 | 6004 | 0 | 5994 |
| npc-brain-parameters | skill3-10 | skill3 | 632 | 0 | 632 |
| npc-brain-parameters | skill3-11 | skill3 | 498 | 0 | 498 |
| npc-brain-parameters | skill3-12 | skill3 | 344 | 0 | 344 |
| npc-brain-parameters | skill3-14 | skill3 | 1971 | 0 | 1961 |
| npc-brain-parameters | skill3-15 | skill3 | 1548 | 0 | 1548 |
| npc-brain-parameters | skill3-16 | skill3 | 1348 | 0 | 1348 |
| npc-brain-parameters | skill3-17 | skill3 | 924 | 0 | 924 |
| npc-brain-parameters | skill3-18 | skill3 | 39 | 0 | 39 |
| npc-brain-parameters | skill3-19 | skill3 | 31 | 0 | 31 |
| npc-brain-parameters | skill3-2 | skill3 | 5640 | 0 | 5629 |
| npc-brain-parameters | skill3-3 | skill3 | 4291 | 0 | 4280 |
| npc-brain-parameters | skill3-4 | skill3 | 2795 | 0 | 2795 |
| npc-brain-parameters | skill3-5 | skill3 | 1041 | 0 | 1041 |
| npc-brain-parameters | skill3-6 | skill3 | 35 | 0 | 35 |
| npc-brain-parameters | skill3-7 | skill3 | 17 | 0 | 17 |
| npc-brain-parameters | skill3-8 | skill3 | 3872 | 0 | 3848 |
| npc-brain-parameters | skill3-9 | skill3 | 1019 | 0 | 1019 |
| npc-brain-parameters | social-1 | social | 2710 | 0 | 2451 |
| npc-brain-parameters | social-10 | social | 3958 | 0 | 3958 |
| npc-brain-parameters | social-11 | social | 3958 | 0 | 3958 |
| npc-brain-parameters | social-12 | social | 3958 | 0 | 3958 |
| npc-brain-parameters | social-2 | social | 1229 | 0 | 1200 |
| npc-brain-parameters | social-3 | social | 621 | 0 | 601 |
| npc-brain-parameters | social-4 | social | 169 | 0 | 159 |
| npc-brain-parameters | social-5 | social | 4746 | 0 | 4746 |
| npc-brain-parameters | social-6 | social | 4746 | 0 | 4746 |
| npc-brain-parameters | social-7 | social | 4748 | 0 | 4746 |
| npc-brain-parameters | social-8 | social | 4746 | 0 | 4746 |
| npc-brain-parameters | social-9 | social | 3959 | 0 | 3958 |
| npc-brain-parameters | stance-1 | stance | 34446 | 0 | 0 |
| npc-brain-parameters | stance-2 | stance | 14087 | 0 | 0 |
| npc-brain-parameters | stance-3 | stance | 15 | 0 | 0 |
| party-battle-field-zone | zone | zone | 3 | 3 | 0 |
| partymatch | faction | faction | 2 | 0 | 2 |
| pc | job | job | 10200 | 0 | 0 |
| pc | level | level | 10200 | 0 | 0 |
| pc-race-sex-job | job | job | 46 | 0 | 0 |
| public-raid | zone | zone | 7 | 7 | 0 |
| quest | battle-royal-field | battle-royal-field | 2 | 0 | 2 |
| quest | dungeon | dungeon | 117 | 0 | 80 |
| quest | faction | faction | 106 | 0 | 2 |
| quest | faction-level | faction-level | 16 | 0 | 0 |
| quest | fielditem | fielditem | 5 | 0 | 4 |
| quest | item | item | 219 | 0 | 191 |
| quest | item-1 | item | 191 | 0 | 138 |
| quest | item-2 | item | 29 | 0 | 12 |
| quest | item-3 | item | 12 | 0 | 9 |
| quest | item-4 | item | 8 | 0 | 8 |
| quest | job-1 | job | 266 | 0 | 0 |
| quest | job-10 | job | 1 | 0 | 0 |
| quest | job-11 | job | 1 | 0 | 0 |
| quest | job-2 | job | 2 | 0 | 0 |
| quest | job-3 | job | 2 | 0 | 0 |
| quest | job-4 | job | 2 | 0 | 0 |
| quest | job-5 | job | 2 | 0 | 0 |
| quest | job-6 | job | 2 | 0 | 0 |
| quest | job-7 | job | 2 | 0 | 0 |
| quest | job-8 | job | 2 | 0 | 0 |
| quest | job-9 | job | 2 | 0 | 0 |
| quest | level | level | 1747 | 0 | 0 |
| quest | mastery-level | mastery-level | 23 | 0 | 0 |
| quest | quest | quest | 374 | 0 | 374 |
| quest | reward-1 | reward | 1624 | 0 | 0 |
| quest | reward-2 | reward | 409 | 0 | 0 |
| quest | skill | skill3 | 14 | 0 | 7 |
| quest | skill3 | skill3 | 29 | 0 | 14 |
| quest | summoned-preset | SummonedPreset | 15 | 0 | 0 |
| quest | zone | zone | 1269 | 0 | 1087 |
| quest | zone-1 | zone | 1872 | 0 | 1521 |
| quest | zone-2 | zone | 7 | 0 | 6 |
| raid-dungeon | boss-npc-1 | BossNpc | 15 | 0 | 0 |
| raid-dungeon | boss-npc-2 | BossNpc | 7 | 0 | 0 |
| raid-dungeon | boss-npc-3 | BossNpc | 5 | 0 | 0 |
| raid-dungeon | boss-npc-4 | BossNpc | 2 | 0 | 0 |
| raid-dungeon | zone-1 | zone | 15 | 0 | 15 |
| raid-dungeon | zone-2 | zone | 7 | 0 | 7 |
| raid-dungeon | zone-3 | zone | 5 | 0 | 5 |
| raid-dungeon | zone-4 | zone | 2 | 0 | 2 |
| random-store-item | item | item | 976 | 0 | 976 |
| random-store-item-group | random-store-item-1 | random-store-item | 122 | 0 | 122 |
| random-store-item-group | random-store-item-2 | random-store-item | 122 | 0 | 122 |
| random-store-item-group | random-store-item-3 | random-store-item | 122 | 0 | 122 |
| random-store-item-group | random-store-item-4 | random-store-item | 122 | 0 | 122 |
| random-store-item-group | random-store-item-5 | random-store-item | 122 | 0 | 122 |
| random-store-item-group | random-store-item-6 | random-store-item | 122 | 0 | 122 |
| random-store-item-group | random-store-item-7 | random-store-item | 122 | 0 | 122 |
| random-store-item-group | random-store-item-8 | random-store-item | 122 | 0 | 122 |
| script | cinematic | cinematic | 252 | 0 | 248 |
| script | effect | effect | 23520 | 0 | 22814 |
| script | field-item | fielditem | 47 | 0 | 47 |
| script | filter-1 | filter | 208 | 0 | 138 |
| script | filter-2 | filter | 61 | 0 | 61 |
| script | filter-3 | filter | 15 | 0 | 15 |
| script | item | item | 1 | 0 | 1 |
| script | npc | npc | 5099 | 0 | 0 |
| script | public-raid-event | public-raid-event | 2 | 0 | 2 |
| script | quest | quest | 3 | 0 | 3 |
| script | social | social | 10289 | 0 | 9572 |
| script | teleport | teleport | 99 | 0 | 92 |
| script | zone | zone | 342 | 0 | 332 |
| simple-field | zone | zone | 7 | 7 | 0 |
| skill-build-up | skill-modify-info-1 | skill-modify-info | 272 | 0 | 272 |
| skill-build-up | skill-modify-info-10 | skill-modify-info | 272 | 0 | 272 |
| skill-build-up | skill-modify-info-2 | skill-modify-info | 272 | 0 | 272 |
| skill-build-up | skill-modify-info-3 | skill-modify-info | 272 | 0 | 272 |
| skill-build-up | skill-modify-info-4 | skill-modify-info | 272 | 0 | 272 |
| skill-build-up | skill-modify-info-5 | skill-modify-info | 272 | 0 | 272 |
| skill-build-up | skill-modify-info-6 | skill-modify-info | 272 | 0 | 272 |
| skill-build-up | skill-modify-info-7 | skill-modify-info | 272 | 0 | 272 |
| skill-build-up | skill-modify-info-8 | skill-modify-info | 272 | 0 | 272 |
| skill-build-up | skill-modify-info-9 | skill-modify-info | 272 | 0 | 272 |
| skill-by-equipment | skill3-id-1 | skill3 | 205 | 204 | 0 |
| skill-by-equipment | skill3-id-2 | skill3 | 36 | 36 | 0 |
| skill-combo | skill-id | skill3 | 133 | 0 | 0 |
| skill-inheritance | skill-id | skill3 | 2674 | 2060 | 0 |
| skill-modify-info-group | job-style | job-style | 6980 | 0 | 0 |
| skill-modify-info-group | skill-modify-info-1 | skill-modify-info | 6509 | 0 | 6377 |
| skill-modify-info-group | skill-modify-info-2 | skill-modify-info | 1517 | 0 | 1463 |
| skill-modify-info-group | skill-modify-info-3 | skill-modify-info | 1076 | 0 | 1022 |
| skill-modify-info-group | skill-modify-info-4 | skill-modify-info | 992 | 0 | 932 |
| skill-train-category | job | job | 1226 | 0 | 0 |
| skill-training-room | zone | zone | 1 | 1 | 0 |
| skill-training-room-group | job | job | 96 | 0 | 0 |
| skill-training-room-group | job-style | job-style | 96 | 0 | 0 |
| skill-trait | job | job | 504 | 0 | 0 |
| skill-trait | job-style | job-style | 504 | 0 | 0 |
| skill3 | campfire | campfire | 1 | 0 | 1 |
| skill3 | skill-modify-limit | SkillModifyLimit | 6362 | 0 | 6078 |
| soul-npc-skill | skill3-1 | skill3 | 16 | 0 | 15 |
| soul-npc-skill | skill3-10 | skill3 | 16 | 0 | 15 |
| soul-npc-skill | skill3-2 | skill3 | 16 | 0 | 15 |
| soul-npc-skill | skill3-3 | skill3 | 16 | 0 | 15 |
| soul-npc-skill | skill3-4 | skill3 | 16 | 0 | 15 |
| soul-npc-skill | skill3-5 | skill3 | 16 | 0 | 15 |
| soul-npc-skill | skill3-6 | skill3 | 16 | 0 | 15 |
| soul-npc-skill | skill3-7 | skill3 | 16 | 0 | 15 |
| soul-npc-skill | skill3-8 | skill3 | 16 | 0 | 15 |
| soul-npc-skill | skill3-9 | skill3 | 16 | 0 | 15 |
| special-skill-gauge | job | job | 12 | 0 | 0 |
| stance | effect-1 | effect | 17 | 0 | 17 |
| stance | effect-2 | effect | 4 | 0 | 4 |
| stance | effect-3 | effect | 4 | 0 | 4 |
| stance | effect-4 | effect | 2 | 0 | 2 |
| store | faction | faction | 14 | 0 | 14 |
| store | item-1 | item | 150 | 0 | 150 |
| store | item-10 | item | 11 | 0 | 11 |
| store | item-11 | item | 10 | 0 | 10 |
| store | item-12 | item | 10 | 0 | 10 |
| store | item-13 | item | 9 | 0 | 9 |
| store | item-14 | item | 9 | 0 | 9 |
| store | item-15 | item | 5 | 0 | 5 |
| store | item-16 | item | 3 | 0 | 3 |
| store | item-17 | item | 2 | 0 | 2 |
| store | item-18 | item | 2 | 0 | 2 |
| store | item-19 | item | 2 | 0 | 2 |
| store | item-2 | item | 147 | 0 | 147 |
| store | item-20 | item | 2 | 0 | 2 |
| store | item-21 | item | 2 | 0 | 2 |
| store | item-22 | item | 2 | 0 | 2 |
| store | item-23 | item | 2 | 0 | 2 |
| store | item-24 | item | 2 | 0 | 2 |
| store | item-25 | item | 2 | 0 | 2 |
| store | item-26 | item | 2 | 0 | 2 |
| store | item-27 | item | 2 | 0 | 2 |
| store | item-3 | item | 145 | 0 | 145 |
| store | item-4 | item | 131 | 0 | 131 |
| store | item-5 | item | 127 | 0 | 127 |
| store | item-6 | item | 123 | 0 | 123 |
| store | item-7 | item | 122 | 0 | 122 |
| store | item-8 | item | 55 | 0 | 55 |
| store | item-9 | item | 17 | 0 | 17 |
| store2 | faction | faction | 30 | 0 | 30 |
| store2 | item-1 | item | 401 | 0 | 401 |
| store2 | item-10 | item | 183 | 0 | 183 |
| store2 | item-100 | item | 2 | 0 | 2 |
| store2 | item-101 | item | 2 | 0 | 2 |
| store2 | item-102 | item | 2 | 0 | 2 |
| store2 | item-103 | item | 2 | 0 | 2 |
| store2 | item-104 | item | 2 | 0 | 2 |
| store2 | item-105 | item | 2 | 0 | 2 |
| store2 | item-106 | item | 2 | 0 | 2 |
| store2 | item-107 | item | 2 | 0 | 2 |
| store2 | item-108 | item | 2 | 0 | 2 |
| store2 | item-109 | item | 1 | 0 | 1 |
| store2 | item-11 | item | 177 | 0 | 177 |
| store2 | item-110 | item | 1 | 0 | 1 |
| store2 | item-111 | item | 1 | 0 | 1 |
| store2 | item-12 | item | 165 | 0 | 162 |
| store2 | item-13 | item | 152 | 0 | 127 |
| store2 | item-14 | item | 149 | 0 | 149 |
| store2 | item-15 | item | 104 | 0 | 101 |
| store2 | item-16 | item | 103 | 0 | 103 |
| store2 | item-17 | item | 95 | 0 | 94 |
| store2 | item-18 | item | 88 | 0 | 88 |
| store2 | item-19 | item | 85 | 0 | 85 |
| store2 | item-2 | item | 327 | 0 | 327 |
| store2 | item-20 | item | 83 | 0 | 83 |
| store2 | item-21 | item | 80 | 0 | 80 |
| store2 | item-22 | item | 78 | 0 | 78 |
| store2 | item-23 | item | 77 | 0 | 76 |
| store2 | item-24 | item | 75 | 0 | 75 |
| store2 | item-25 | item | 73 | 0 | 46 |
| store2 | item-26 | item | 47 | 0 | 47 |
| store2 | item-27 | item | 45 | 0 | 45 |
| store2 | item-28 | item | 42 | 0 | 42 |
| store2 | item-29 | item | 39 | 0 | 38 |
| store2 | item-3 | item | 313 | 0 | 313 |
| store2 | item-30 | item | 37 | 0 | 37 |
| store2 | item-31 | item | 37 | 0 | 37 |
| store2 | item-32 | item | 37 | 0 | 37 |
| store2 | item-33 | item | 37 | 0 | 37 |
| store2 | item-34 | item | 36 | 0 | 36 |
| store2 | item-35 | item | 35 | 0 | 35 |
| store2 | item-36 | item | 32 | 0 | 32 |
| store2 | item-37 | item | 32 | 0 | 32 |
| store2 | item-38 | item | 29 | 0 | 29 |
| store2 | item-39 | item | 27 | 0 | 27 |
| store2 | item-4 | item | 308 | 0 | 308 |
| store2 | item-40 | item | 25 | 0 | 25 |
| store2 | item-41 | item | 25 | 0 | 25 |
| store2 | item-42 | item | 25 | 0 | 25 |
| store2 | item-43 | item | 22 | 0 | 22 |
| store2 | item-44 | item | 21 | 0 | 21 |
| store2 | item-45 | item | 20 | 0 | 20 |
| store2 | item-46 | item | 20 | 0 | 20 |
| store2 | item-47 | item | 20 | 0 | 20 |
| store2 | item-48 | item | 19 | 0 | 19 |
| store2 | item-49 | item | 17 | 0 | 17 |
| store2 | item-5 | item | 302 | 0 | 302 |
| store2 | item-50 | item | 17 | 0 | 17 |
| store2 | item-51 | item | 17 | 0 | 17 |
| store2 | item-52 | item | 17 | 0 | 17 |
| store2 | item-53 | item | 17 | 0 | 17 |
| store2 | item-54 | item | 16 | 0 | 16 |
| store2 | item-55 | item | 15 | 0 | 15 |
| store2 | item-56 | item | 15 | 0 | 15 |
| store2 | item-57 | item | 14 | 0 | 14 |
| store2 | item-58 | item | 14 | 0 | 14 |
| store2 | item-59 | item | 14 | 0 | 14 |
| store2 | item-6 | item | 299 | 0 | 299 |
| store2 | item-60 | item | 14 | 0 | 14 |
| store2 | item-61 | item | 14 | 0 | 14 |
| store2 | item-62 | item | 14 | 0 | 14 |
| store2 | item-63 | item | 14 | 0 | 14 |
| store2 | item-64 | item | 14 | 0 | 14 |
| store2 | item-65 | item | 14 | 0 | 14 |
| store2 | item-66 | item | 13 | 0 | 13 |
| store2 | item-67 | item | 13 | 0 | 13 |
| store2 | item-68 | item | 13 | 0 | 13 |
| store2 | item-69 | item | 13 | 0 | 13 |
| store2 | item-7 | item | 293 | 0 | 293 |
| store2 | item-70 | item | 13 | 0 | 13 |
| store2 | item-71 | item | 13 | 0 | 13 |
| store2 | item-72 | item | 13 | 0 | 13 |
| store2 | item-73 | item | 13 | 0 | 13 |
| store2 | item-74 | item | 12 | 0 | 12 |
| store2 | item-75 | item | 12 | 0 | 12 |
| store2 | item-76 | item | 12 | 0 | 12 |
| store2 | item-77 | item | 11 | 0 | 11 |
| store2 | item-78 | item | 11 | 0 | 11 |
| store2 | item-79 | item | 11 | 0 | 11 |
| store2 | item-8 | item | 200 | 0 | 200 |
| store2 | item-80 | item | 11 | 0 | 11 |
| store2 | item-81 | item | 11 | 0 | 11 |
| store2 | item-82 | item | 11 | 0 | 11 |
| store2 | item-83 | item | 11 | 0 | 11 |
| store2 | item-84 | item | 11 | 0 | 11 |
| store2 | item-85 | item | 11 | 0 | 11 |
| store2 | item-86 | item | 11 | 0 | 11 |
| store2 | item-87 | item | 11 | 0 | 11 |
| store2 | item-88 | item | 6 | 0 | 6 |
| store2 | item-89 | item | 5 | 0 | 5 |
| store2 | item-9 | item | 186 | 0 | 186 |
| store2 | item-90 | item | 4 | 0 | 4 |
| store2 | item-91 | item | 4 | 0 | 4 |
| store2 | item-92 | item | 4 | 0 | 4 |
| store2 | item-93 | item | 4 | 0 | 4 |
| store2 | item-94 | item | 4 | 0 | 4 |
| store2 | item-95 | item | 4 | 0 | 4 |
| store2 | item-96 | item | 4 | 0 | 4 |
| store2 | item-97 | item | 4 | 0 | 4 |
| store2 | item-98 | item | 3 | 0 | 3 |
| store2 | item-99 | item | 2 | 0 | 2 |
| summoned | job | job | 8 | 0 | 0 |
| summoned | stance | stance | 8 | 0 | 0 |
| summonedLevel | job | job | 1928 | 0 | 0 |
| summonedLevel | level | level | 1928 | 0 | 0 |
| unlocated-store | store2 | store | 7 | 0 | 1 |
| unlocated-store | store2 | store2 | 7 | 0 | 7 |
| zone | script | script | 1363 | 0 | 770 |
| zone | terrain | Terrain | 1533 | 1533 | 0 |
| zone-pc-random-spawn | zone | zone | 1 | 1 | 0 |
