use anyhow::Result as AnyResult;
use base64;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use std::io::Write;
use std::path::PathBuf;

use crate::cache::{Card, STSCache};
use crate::cache_enums::CardColor;

const ENCODING_KEY: &[u8] = b"key";

#[derive(Serialize, Deserialize, Debug)]
struct JsonCard {
    id: String,
    misc: u32,
    upgrades: u32,
}

fn encode(data: &[u8], key: &[u8]) -> Vec<u8> {
    let mut result = Vec::from(data);
    let key_len = key.len();
    result
        .iter_mut()
        .enumerate()
        .for_each(|(i, v)| *v ^= key[i % key_len]);
    result
}

pub fn get_save_file_path(folder: &PathBuf) -> Option<PathBuf> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() <= 1 {
        let mut save_file_names = Vec::with_capacity(5);
        for entry_result in std::fs::read_dir(&folder).ok()? {
            if let Ok(entry) = entry_result {
                if let Ok(file_data) = entry.metadata() {
                    if file_data.is_file() {
                        let filename = entry.path();
                        if let Some(extension) = filename.extension() {
                            if extension == "autosave" {
                                save_file_names.push(filename);
                            }
                        }
                    }
                }
            };
        }
        match save_file_names.len() {
            0 => None,
            1 => Some(folder.join(save_file_names.pop().unwrap())),
            _ => {
                println!("Found several save files, select one:");
                let mut buffer = String::with_capacity(5);
                let choice = loop {
                    buffer.clear();
                    for (index, file) in save_file_names.iter().enumerate() {
                        println!("{}: {:?}", index + 1, file);
                    }
                    std::io::stdin().read_line(&mut buffer).ok()?;
                    if let Ok(value) = usize::from_str_radix(buffer.trim(), 10) {
                        if value > 0 && value <= save_file_names.len() {
                            break value - 1;
                        }
                    }
                };
                Some(folder.join(save_file_names[choice].clone()))
            }
        }
    } else {
        let arg = PathBuf::from(args.last().unwrap());
        if arg.exists() {
            Some(arg)
        } else {
            let arg_filename = folder.join(arg);
            if arg_filename.exists() {
                Some(arg_filename)
            } else {
                None
            }
        }
    }
}

fn unpack_file(savefile_path: &PathBuf) -> AnyResult<JsonValue> {
    let contents = std::fs::read(&savefile_path)?;
    let config = base64::Config::new(base64::CharacterSet::Standard, true);
    let unbased_encoded = base64::decode_config(&contents, config)?;
    let unbased_decoded = encode(&unbased_encoded, ENCODING_KEY);
    let json_dict: JsonValue = serde_json::from_slice(&unbased_decoded)?;
    Ok(json_dict)
}

fn pack_file(value: JsonValue, filepath: &PathBuf) -> AnyResult<()> {
    let config = base64::Config::new(base64::CharacterSet::Standard, true);
    let json_string = value.to_string();
    let unbased_encoded = encode(&json_string.as_bytes(), ENCODING_KEY);
    let based_encoded = base64::encode_config(&unbased_encoded, config);
    std::fs::write(&filepath, based_encoded).map_err(anyhow::Error::msg)
}

fn print_status(json: &JsonValue) {
    println!("Gold: {} ({} gained)", json["gold"], json["gold_gained"]);

    let cards = serde_json::from_value::<Vec<JsonCard>>(json["cards"].clone()).unwrap();
    let cards_ids: Vec<String> = cards.into_iter().map(|x| x.id).collect();
    println!("Cards ({}): {:?}", cards_ids.len(), cards_ids);

    let relics = serde_json::from_value::<Vec<String>>(json["relics"].clone()).unwrap();
    println!("Relics: {:?}", relics);

    println!("Select action ('q' to quit):");
    println!("g - Give 100 gold");
    println!("z - Remove all cards");
    println!("x - Give 10 random cards");
    println!("c - Give 5 Colorless cards");
    println!("v/b/n/m - Give 5 Red/Green/Blue/Purple cards");
    println!("f - Give card by name");
    println!("r - Remove card by name");
}

fn get_random_cards(
    cache: &STSCache,
    json_dict: &JsonValue,
    rng: &mut rand::rngs::ThreadRng,
    amount: u32,
    filter: impl Fn(&Card) -> bool,
) -> JsonValue {
    let mut current_cards =
        serde_json::from_value::<Vec<JsonCard>>(json_dict["cards"].clone()).unwrap();
    for _ in 0..amount {
        let random_card = loop {
            let random_card_id = rng.gen_range(0, cache.cards.len());
            let random_card_data = &cache.cards[random_card_id];
            if !filter(random_card_data) {
                continue;
            } else {
                break JsonCard {
                    id: random_card_data.id.clone(),
                    misc: random_card_data.misc,
                    upgrades: 0,
                };
            }
        };
        current_cards.push(random_card);
    }
    serde_json::to_value(current_cards).unwrap()
}

fn add_specific_card(cache: &STSCache, json_dict: &JsonValue, card_name: &str) -> JsonValue {
    let mut current_cards =
        serde_json::from_value::<Vec<JsonCard>>(json_dict["cards"].clone()).unwrap();
    for card in &cache.cards {
        if card.id == card_name {
            let new_card = JsonCard {
                id: card.id.clone(),
                misc: card.misc,
                upgrades: 0,
            };
            current_cards.push(new_card);
        }
    }
    serde_json::to_value(current_cards).unwrap()
}

fn remove_specific_card(json_dict: &JsonValue, card_name: &str) -> JsonValue {
    let mut current_cards =
        serde_json::from_value::<Vec<JsonCard>>(json_dict["cards"].clone()).unwrap();
    current_cards.retain(|x| x.id != card_name);
    serde_json::to_value(current_cards).unwrap()
}

fn get_card_name_from_user(possible_cards: &[String]) -> Option<String> {
    let mut buffer = String::with_capacity(10);
    let mut results: Vec<_> = Vec::with_capacity(10);
    loop {
        print!("Enter the name of card (or nothing to leave): ");
        std::io::stdout().flush().expect("Failed to flush stdout.");
        buffer.clear();
        results.clear();
        std::io::stdin()
            .read_line(&mut buffer)
            .expect("Failed to read input into buffer in get_card_name_from_user.");
        let lower_buffer = buffer.to_lowercase();
        let needle = lower_buffer.trim();
        if needle.is_empty() {
            break None;
        }

        for choice in possible_cards {
            if choice.to_lowercase().contains(needle) {
                results.push(choice.clone());
            }
        }
        match results.len() {
            0 => continue,
            1 => break Some(results.pop().expect("Tried to pop from empty results.")),
            _ => println!("Found several matches: {:?}", results),
        }
    }
}

pub fn process_file(save_file: &PathBuf, cache: &STSCache) -> AnyResult<()> {
    let mut json_dict = unpack_file(save_file)?;
    let mut buffer = String::with_capacity(5);
    let mut rng = rand::thread_rng();

    let all_cache_card_ids: Vec<_> = cache.cards.iter().map(|x| x.id.clone()).collect();
    loop {
        print_status(&json_dict);
        buffer.clear();
        std::io::stdin().read_line(&mut buffer)?;
        match buffer.trim() {
            "g" => {
                let g1 = serde_json::from_value::<u32>(json_dict["gold"].clone()).unwrap() + 100;
                json_dict["gold"] = JsonValue::from(g1);
                let g2 =
                    serde_json::from_value::<u32>(json_dict["gold_gained"].clone()).unwrap() + 100;
                json_dict["gold_gained"] = JsonValue::from(g2);
            }
            "f" => {
                if let Some(card_name) = get_card_name_from_user(&all_cache_card_ids) {
                    json_dict["cards"] = add_specific_card(&cache, &json_dict, &card_name);
                }
            }
            "r" => {
                if let Some(card_name) = get_card_name_from_user(&all_cache_card_ids) {
                    json_dict["cards"] = remove_specific_card(&json_dict, &card_name);
                }
            }
            "z" => {
                json_dict["cards"] = serde_json::to_value(Vec::<JsonCard>::new()).unwrap();
            }
            "x" => {
                json_dict["cards"] = get_random_cards(&cache, &json_dict, &mut rng, 10, |_| true);
            }
            "v" => {
                json_dict["cards"] = get_random_cards(&cache, &json_dict, &mut rng, 5, |x| {
                    x.color == CardColor::RED
                });
            }
            "b" => {
                json_dict["cards"] = get_random_cards(&cache, &json_dict, &mut rng, 5, |x| {
                    x.color == CardColor::GREEN
                });
            }
            "n" => {
                json_dict["cards"] = get_random_cards(&cache, &json_dict, &mut rng, 5, |x| {
                    x.color == CardColor::BLUE
                });
            }
            "m" => {
                json_dict["cards"] = get_random_cards(&cache, &json_dict, &mut rng, 5, |x| {
                    x.color == CardColor::PURPLE
                });
            }
            "c" => {
                json_dict["cards"] = get_random_cards(&cache, &json_dict, &mut rng, 5, |x| {
                    x.color == CardColor::COLORLESS
                });
            }
            "q" => break,
            _ => continue,
        }
    }
    pack_file(json_dict, save_file)
}
