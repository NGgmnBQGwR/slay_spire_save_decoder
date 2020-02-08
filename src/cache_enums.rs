use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum CardRarity {
    BASIC,
    SPECIAL,
    COMMON,
    UNCOMMON,
    RARE,
    CURSE,
}
impl CardRarity {
    pub fn from_str(s: &str) -> Option<CardRarity> {
        match s {
            "BASIC" => Some(CardRarity::BASIC),
            "SPECIAL" => Some(CardRarity::SPECIAL),
            "COMMON" => Some(CardRarity::COMMON),
            "UNCOMMON" => Some(CardRarity::UNCOMMON),
            "RARE" => Some(CardRarity::RARE),
            "CURSE" => Some(CardRarity::CURSE),
            _ => None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum CardColor {
    RED,
    GREEN,
    BLUE,
    PURPLE,
    COLORLESS,
    CURSE,
}
impl CardColor {
    pub fn from_str(s: &str) -> Option<CardColor> {
        match s {
            "RED" => Some(CardColor::RED),
            "GREEN" => Some(CardColor::GREEN),
            "BLUE" => Some(CardColor::BLUE),
            "PURPLE" => Some(CardColor::PURPLE),
            "COLORLESS" => Some(CardColor::COLORLESS),
            "CURSE" => Some(CardColor::CURSE),
            _ => None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum CardType {
    ATTACK,
    SKILL,
    POWER,
    STATUS,
    CURSE,
}
impl CardType {
    pub fn from_str(s: &str) -> Option<CardType> {
        match s {
            "ATTACK" => Some(CardType::ATTACK),
            "SKILL" => Some(CardType::SKILL),
            "POWER" => Some(CardType::POWER),
            "STATUS" => Some(CardType::STATUS),
            "CURSE" => Some(CardType::CURSE),
            _ => None,
        }
    }
}
#[derive(Debug, Deserialize, Serialize)]
pub enum RelicTier {
    DEPRECATED,
    STARTER,
    COMMON,
    UNCOMMON,
    RARE,
    SPECIAL,
    BOSS,
    SHOP,
}
impl RelicTier {
    pub fn from_str(s: &str) -> Option<RelicTier> {
        match s {
            "DEPRECATED" => Some(RelicTier::DEPRECATED),
            "STARTER" => Some(RelicTier::STARTER),
            "COMMON" => Some(RelicTier::COMMON),
            "UNCOMMON" => Some(RelicTier::UNCOMMON),
            "RARE" => Some(RelicTier::RARE),
            "SPECIAL" => Some(RelicTier::SPECIAL),
            "BOSS" => Some(RelicTier::BOSS),
            "SHOP" => Some(RelicTier::SHOP),
            _ => None,
        }
    }
}
