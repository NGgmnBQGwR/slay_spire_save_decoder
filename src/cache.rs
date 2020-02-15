use anyhow::{anyhow, Result as AnyResult};
use bincode::serialize;
use regex::Regex;
use serde::{Deserialize, Serialize};

use std::collections::HashSet;
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;

use crate::cache_enums::{CardColor, CardRarity, CardType, RelicTier};

#[derive(Debug, Deserialize, Serialize)]
pub struct Card {
    pub rarity: CardRarity,
    pub color: CardColor,
    pub type_: CardType,
    pub id: String,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct Relic {
    pub tier: RelicTier,
    pub id: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct STSCache {
    pub cards: Vec<Card>,
    pub relics: Vec<Relic>,
}

fn parse_card(contents: &str) -> Card {
    let id_regex = Regex::new(r#"ID[ ]*=[ ]*"(.+)""#).expect("Failed to compile id regex.");
    let cap_regex = Regex::new(r"([A-Z]{2,})").expect("Failed to compile capital regex.");
    let id_match = id_regex
        .captures(&contents)
        .expect("Failed to get create regex capture groups.");
    let id = id_match.get(1).expect("Failed to get id regex group.");

    let cap_matches: Vec<_> = cap_regex
        .find_iter(&contents)
        .map(|mat| mat.as_str())
        .collect();

    let mut rarity: Vec<_> = cap_matches
        .iter()
        .filter_map(|x| CardRarity::from_str(x))
        .collect();
    let mut color: Vec<_> = cap_matches
        .iter()
        .filter_map(|x| CardColor::from_str(x))
        .collect();
    let mut type_: Vec<_> = cap_matches
        .iter()
        .filter_map(|x| CardType::from_str(x))
        .collect();
    Card {
        id: id.as_str().to_owned(),
        rarity: rarity
            .pop()
            .expect("Expected 1 rarity regex result, got 0."),
        color: color.pop().expect("Expected 1 color regex result, got 0."),
        type_: type_.pop().expect("Expected 1 type regex result, got 0."),
    }
}

fn parse_relic(contents: &str) -> Option<Relic> {
    let id_regex = Regex::new(r#"ID[ ]*=[ ]*"(.+)""#).unwrap();
    let cap_regex = Regex::new(r"([A-Z]{2,})").unwrap();
    let id_match = id_regex.captures(&contents).unwrap();
    let id = id_match.get(1)?;

    let cap_matches: Vec<_> = cap_regex
        .find_iter(&contents)
        .map(|mat| mat.as_str())
        .collect();

    let mut rarity: Vec<_> = cap_matches
        .iter()
        .filter_map(|x| RelicTier::from_str(x))
        .collect();
    if rarity.is_empty() {
        None
    } else {
        Some(Relic {
            id: id.as_str().to_owned(),
            tier: rarity.pop().unwrap(),
        })
    }
}

impl std::fmt::Display for STSCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cache ({} cards, {} relics)",
            self.cards.len(),
            self.relics.len()
        )
    }
}

impl STSCache {
    const CACHE_MAGIC_WORD: [u8; 4] = [0x5, 0xE, 0xE, 0x5];
    const CACHE_VERSION: u32 = 1;
    const CACHE_FILENAME: &'static str = "_cache.stsc";

    fn walk_dir(
        start_dir: PathBuf,
        folder_filter: impl Fn(&PathBuf) -> bool,
        file_filter: impl Fn(&PathBuf) -> bool,
    ) -> AnyResult<Vec<String>> {
        let mut results = Vec::with_capacity(100);
        let mut dir_queue = Vec::with_capacity(50);
        dir_queue.push(start_dir);

        while let Some(next_dir) = dir_queue.pop() {
            for dir_contents in std::fs::read_dir(&next_dir)? {
                if let Ok(entry) = dir_contents {
                    if let Ok(file_data) = entry.file_type() {
                        let filepath = entry.path();
                        if file_data.is_file() {
                            if !file_filter(&filepath) {
                                continue;
                            }
                            if let Some(extension) = filepath.extension() {
                                if extension.to_string_lossy().to_lowercase() == "java" {
                                    if let Ok(file_contents) = std::fs::read_to_string(&filepath) {
                                        results.push(file_contents);
                                    }
                                }
                            }
                        } else {
                            if !folder_filter(&filepath) {
                                continue;
                            }
                            dir_queue.push(filepath);
                        }
                    }
                };
            }
        }
        Ok(results)
    }

    fn create_cache_from_src_folder(folder: &PathBuf) -> AnyResult<Self> {
        let src_folder = folder.join("sts_src");
        if src_folder.exists() {
            let cards = {
                let card_folder = src_folder
                    .join("com")
                    .join("megacrit")
                    .join("cardcrawl")
                    .join("cards");
                if !card_folder.exists() {
                    return Err(anyhow!("Unable to find cards folder '{:?}'", card_folder));
                }
                let folders_to_skip = {
                    let mut temp = HashSet::with_capacity(20);
                    temp.insert(std::ffi::OsStr::new("curses"));
                    temp.insert(std::ffi::OsStr::new("deprecated"));
                    temp.insert(std::ffi::OsStr::new("optionCards"));
                    temp.insert(std::ffi::OsStr::new("status"));
                    temp.insert(std::ffi::OsStr::new("tempCards"));
                    temp
                };

                let files_to_skip = {
                    let mut temp = HashSet::with_capacity(20);
                    temp.insert(std::ffi::OsStr::new("AbstractCard.java"));
                    temp.insert(std::ffi::OsStr::new("CardGroup.java"));
                    temp.insert(std::ffi::OsStr::new("CardModUNUSED.java"));
                    temp.insert(std::ffi::OsStr::new("CardQueueItem.java"));
                    temp.insert(std::ffi::OsStr::new("CardSave.java"));
                    temp.insert(std::ffi::OsStr::new("DamageInfo.java"));
                    temp.insert(std::ffi::OsStr::new("DescriptionLine.java"));
                    temp.insert(std::ffi::OsStr::new("Soul.java"));
                    temp.insert(std::ffi::OsStr::new("SoulGroup.java"));
                    temp
                };

                let folder_filter = |folder: &PathBuf| {
                    let filename = folder.file_name();
                    filename.is_some() && !folders_to_skip.contains(filename.unwrap())
                };

                let file_filter = |file: &PathBuf| {
                    let filename = file.file_name();
                    filename.is_some() && !files_to_skip.contains(filename.unwrap())
                };

                STSCache::walk_dir(card_folder, folder_filter, file_filter)?
                    .into_iter()
                    .map(|x| parse_card(&x))
                    .collect()
            };

            let relics = {
                let relic_folder = src_folder
                    .join("com")
                    .join("megacrit")
                    .join("cardcrawl")
                    .join("relics");
                if !relic_folder.exists() {
                    return Err(anyhow!("Unable to find relics folder '{:?}'", relic_folder));
                }
                let folders_to_skip = {
                    let mut temp = HashSet::with_capacity(20);
                    temp.insert(std::ffi::OsStr::new("deprecated"));
                    temp
                };

                let files_to_skip = {
                    let mut temp = HashSet::with_capacity(20);
                    temp.insert(std::ffi::OsStr::new("AbstractRelic.java"));
                    temp.insert(std::ffi::OsStr::new("Test1.java"));
                    temp.insert(std::ffi::OsStr::new("Test3.java"));
                    temp.insert(std::ffi::OsStr::new("Test4.java"));
                    temp.insert(std::ffi::OsStr::new("Test5.java"));
                    temp.insert(std::ffi::OsStr::new("Test6.java"));
                    temp
                };

                let folder_filter = |folder: &PathBuf| {
                    let filename = folder.file_name();
                    filename.is_some() && !folders_to_skip.contains(filename.unwrap())
                };

                let file_filter = |file: &PathBuf| {
                    let filename = file.file_name();
                    filename.is_some() && !files_to_skip.contains(filename.unwrap())
                };

                STSCache::walk_dir(relic_folder, folder_filter, file_filter)?
                    .into_iter()
                    .filter_map(|x| parse_relic(&x))
                    .collect()
            };
            let cache = STSCache { cards, relics };
            cache.save(&folder);
            Ok(cache)
        } else {
            Err(anyhow!("Unable to find src folder '{:?}'", src_folder))
        }
    }

    fn load_cache(cache_filepath: &PathBuf) -> AnyResult<Self> {
        let mut cache_file = BufReader::new(std::fs::File::open(cache_filepath)?);

        let magic_word = bincode::config().deserialize_from::<_, [u8; 4]>(&mut cache_file)?;
        if magic_word != STSCache::CACHE_MAGIC_WORD {
            return Err(anyhow!(
                "Expected magic word {:?}, got {:?}",
                STSCache::CACHE_MAGIC_WORD,
                magic_word
            ));
        }

        let version = bincode::config().deserialize_from::<_, u32>(&mut cache_file)?;
        if version != STSCache::CACHE_VERSION {
            return Err(anyhow!(
                "Expected version {:?}, got {:?}",
                STSCache::CACHE_VERSION,
                version
            ));
        }

        Ok(bincode::config().deserialize_from::<_, STSCache>(&mut cache_file)?)
    }

    fn save(&self, folder: &PathBuf) {
        let cache_filepath = folder.join(STSCache::CACHE_FILENAME);
        let mut cache_file = BufWriter::new(std::fs::File::create(cache_filepath).unwrap());
        if let Ok(serialized_data) = serialize(&STSCache::CACHE_MAGIC_WORD) {
            cache_file.write_all(&serialized_data).unwrap();
        }
        if let Ok(serialized_data) = serialize(&STSCache::CACHE_VERSION) {
            cache_file.write_all(&serialized_data).unwrap();
        }
        if let Ok(serialized_data) = serialize(self) {
            cache_file.write_all(&serialized_data).unwrap();
        }
    }

    pub fn load_or_create_from_file_in_folder(folder: &PathBuf) -> AnyResult<Self> {
        if folder.join(STSCache::CACHE_FILENAME).exists() {
            STSCache::load_cache(&folder.join(STSCache::CACHE_FILENAME))
        } else {
            STSCache::create_cache_from_src_folder(&folder)
        }
    }
}
