use clap::{arg, command, value_parser, ArgAction, Command};
use futures::stream::StreamExt;
use paris::{error, info};
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use std::collections::HashMap;
use std::fmt::Display;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

const TOTAL_BOYS: usize = 5000;
const TOTAL_GIRLS: usize = 5000;

const BUFFER_SIZE: usize = 1000;

#[tokio::main]
async fn main() {
    info!("Starting...");

    // parse cli
    let matches = command!()
        .subcommand(
            Command::new("generate-emotions")
                .arg(
                    arg!(--from <FILE> "From mapping file")
                        .required(true)
                        .value_parser(value_parser!(PathBuf)),
                )
                .arg(
                    arg!(--"input-dir" <DIR> "Input directory path")
                        .required(true)
                        .value_parser(value_parser!(PathBuf)),
                )
                .arg(
                    arg!(--"from-index" <INDEX> "From index")
                        .required(false)
                        .value_parser(value_parser!(usize)),
                ),
        )
        .arg(
            arg!(--"input-dir" <DIR> "Input directory path")
                .required(false)
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            arg!(--"output-dir" <DIR> "Output directory path")
                .required(false)
                .default_value("output")
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            arg!(--reset <BOOL> "Reset generation")
                .required(false)
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("generate-emotions", args)) => {
            if let Some(mapping_file) = args.get_one::<PathBuf>("from") {
                info!("Generate emotions from mapping file {mapping_file:?}");

                let input_dir = args
                    .get_one::<PathBuf>("input-dir")
                    .expect("Missing input dir");

                let mut output_dir = PathBuf::from("emotions_girl");
                if is_boy(&input_dir) {
                    output_dir = PathBuf::from("emotions_boy");
                }

                let from_index = args.get_one::<usize>("from-index").unwrap_or(&0);

                generate_emotions(mapping_file, input_dir, &output_dir, *from_index).await;
                return;
            }
        }
        _ => {}
    }

    if let Some(dir) = matches.get_one::<PathBuf>("input-dir") {
        info!("Working on directory {dir:?}");

        let output_dir = matches.get_one::<PathBuf>("output-dir").unwrap();
        info!("Output directory {:?}", output_dir);

        let is_reset = matches.get_one::<bool>("reset").unwrap_or(&true);

        if is_boy(&dir) {
            handle(&dir, &output_dir, Gender::Boy, TOTAL_BOYS, *is_reset).await;
        } else {
            handle(&dir, &output_dir, Gender::Girl, TOTAL_GIRLS, *is_reset).await;
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Gender {
    Boy,
    Girl,
}

impl Display for Gender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Gender::Boy => {
                write!(f, "boy")
            }
            Gender::Girl => {
                write!(f, "girl")
            }
        }
    }
}

trait RandomizedPart {
    fn random_part(gender: Gender) -> Option<&'static str>;
}

struct Background {}
impl RandomizedPart for Background {
    fn random_part(_gender: Gender) -> Option<&'static str> {
        let rarity = Rarity::pick_random_rarity();

        match rarity {
            Rarity::Common => Some(pick_random(&[
                "NFT_BG_1", "NFT_BG_2", "NFT_BG_3", "NFT_BG_4",
            ])),
            Rarity::Uncommon => None,
            Rarity::Rare => Some(pick_random(&["NFT_BG_5", "NFT_BG_6"])),
            Rarity::Epic => Some(pick_random(&["NFT_BG_7", "NFT_BG_9", "NFT_BG_10"])),
            Rarity::Legendary => Some(pick_random(&["NFT_BG_12"])),
            Rarity::Mythical => Some(pick_random(&["NFT_BG_8", "NFT_BG_11"])),
        }
    }
}

struct Hand {}
impl RandomizedPart for Hand {
    fn random_part(_gender: Gender) -> Option<&'static str> {
        let rarity = Rarity::pick_random_rarity();

        match rarity {
            Rarity::Common => Some(pick_random(&["NFT_Hand_1", "NFT_Hand_2"])),
            Rarity::Uncommon => Some(pick_random(&["NFT_Hand_3", "NFT_Hand_4"])),
            Rarity::Rare => Some(pick_random(&["NFT_Hand_5", "NFT_Hand_6"])),
            Rarity::Epic => Some(pick_random(&["NFT_Hand_7", "NFT_Hand_8"])),
            Rarity::Legendary => Some(pick_random(&["NFT_Hand_9", "NFT_Hand_10", "NFT_Hand_11"])),
            Rarity::Mythical => Some(pick_random(&["NFT_Hand_12", "NFT_Hand_13"])),
        }
    }
}

struct HairLong {}

impl HairLong {
    fn is_with_headphone(gender: Gender, variant: &str) -> bool {
        match gender {
            Gender::Boy => match variant {
                "NFT_B_Hair_Long_2" | "NFT_B_Hair_Long_3" | "NFT_B_Hair_Long_4"
                | "NFT_B_Hair_Long_7" | "NFT_B_Hair_Long_8" | "NFT_B_Hair_Long_9"
                | "NFT_B_Hair_Long_10" | "NFT_B_Hair_Long_1" | "NFT_B_Hair_Long_5"
                | "NFT_B_Hair_Long_6" => false,
                _ => true,
            },
            Gender::Girl => match variant {
                "NFT_G_Hair_Long_4" | "NFT_G_Hair_Long_5" | "NFT_G_Hair_Long_6"
                | "NFT_G_Hair_Long_7" | "NFT_G_Hair_Long_8" | "NFT_G_Hair_Long_9"
                | "NFT_G_Hair_Long_10" | "NFT_G_Hair_Long_13" | "NFT_G_Hair_Long_14"
                | "NFT_G_Hair_Long_15" | "NFT_G_Hair_Long_16" | "NFT_G_Hair_Long_1"
                | "NFT_G_Hair_Long_2" | "NFT_G_Hair_Long_3" | "NFT_G_Hair_Long_11"
                | "NFT_G_Hair_Long_12" => false,
                _ => true,
            },
        }
    }
    fn is_with_face_acc(variant: &str) -> bool {
        match variant {
            "NFT_B_Hair_Long_32" | "NFT_B_Hair_Long_33" | "NFT_B_Hair_Long_34" => false,
            _ => true,
        }
    }
}
impl RandomizedPart for HairLong {
    fn random_part(gender: Gender) -> Option<&'static str> {
        match gender {
            Gender::Boy => {
                let rarity = Rarity::pick_random_rarity();

                match rarity {
                    Rarity::Common => Some(pick_random(&[
                        "NFT_B_Hair_Long_11",
                        "NFT_B_Hair_Long_12",
                        "NFT_B_Hair_Long_13",
                        "NFT_B_Hair_Long_14",
                        "NFT_B_Hair_Long_15",
                        "NFT_B_Hair_Long_16",
                        "NFT_B_Hair_Long_17",
                        "NFT_B_Hair_Long_18",
                        "NFT_B_Hair_Long_19",
                        "NFT_B_Hair_Long_20",
                        "NFT_B_Hair_Long_21",
                        "NFT_B_Hair_Long_22",
                        "NFT_B_Hair_Long_23",
                        "NFT_B_Hair_Long_24",
                        "NFT_B_Hair_Long_25",
                        "NFT_B_Hair_Long_26",
                        "NFT_B_Hair_Long_27",
                        "NFT_B_Hair_Long_28",
                        "NFT_B_Hair_Long_29",
                        "NFT_B_Hair_Long_30",
                        "NFT_B_Hair_Long_31",
                    ])),
                    Rarity::Uncommon => Some(pick_random(&[
                        "NFT_B_Hair_Long_2",
                        "NFT_B_Hair_Long_3",
                        "NFT_B_Hair_Long_4",
                        "NFT_B_Hair_Long_7",
                        "NFT_B_Hair_Long_8",
                        "NFT_B_Hair_Long_9",
                        "NFT_B_Hair_Long_10",
                    ])),
                    Rarity::Rare => Some(pick_random(&[
                        "NFT_B_Hair_Long_32",
                        "NFT_B_Hair_Long_33",
                        "NFT_B_Hair_Long_34",
                        "NFT_B_Hair_Long_35",
                        "NFT_B_Hair_Long_36",
                        "NFT_B_Hair_Long_37",
                        "NFT_B_Hair_Long_38",
                    ])),
                    Rarity::Epic => Some(pick_random(&[
                        "NFT_B_Hair_Long_1",
                        "NFT_B_Hair_Long_5",
                        "NFT_B_Hair_Long_6",
                    ])),
                    Rarity::Legendary => {
                        Some(pick_random(&["NFT_B_Hair_Long_39", "NFT_B_Hair_Long_41"]))
                    }
                    Rarity::Mythical => {
                        Some(pick_random(&["NFT_B_Hair_Long_40", "NFT_B_Hair_Long_42"]))
                    }
                }
            }
            Gender::Girl => {
                let rarity = Rarity::pick_random_rarity();

                match rarity {
                    Rarity::Common => Some(pick_random(&[
                        "NFT_G_Hair_Long_17",
                        "NFT_G_Hair_Long_18",
                        "NFT_G_Hair_Long_19",
                        "NFT_G_Hair_Long_20",
                        "NFT_G_Hair_Long_21",
                        "NFT_G_Hair_Long_22",
                        "NFT_G_Hair_Long_23",
                        "NFT_G_Hair_Long_24",
                        "NFT_G_Hair_Long_25",
                        "NFT_G_Hair_Long_26",
                        "NFT_G_Hair_Long_27",
                        "NFT_G_Hair_Long_28",
                        "NFT_G_Hair_Long_29",
                        "NFT_G_Hair_Long_30",
                        "NFT_G_Hair_Long_31",
                    ])),
                    Rarity::Uncommon => Some(pick_random(&[
                        "NFT_G_Hair_Long_4",
                        "NFT_G_Hair_Long_5",
                        "NFT_G_Hair_Long_6",
                        "NFT_G_Hair_Long_7",
                        "NFT_G_Hair_Long_8",
                        "NFT_G_Hair_Long_9",
                        "NFT_G_Hair_Long_10",
                        "NFT_G_Hair_Long_13",
                        "NFT_G_Hair_Long_14",
                        "NFT_G_Hair_Long_15",
                        "NFT_G_Hair_Long_16",
                        "NFT_G_Hair_Long_32",
                        "NFT_G_Hair_Long_33",
                        "NFT_G_Hair_Long_37",
                        "NFT_G_Hair_Long_38",
                        "NFT_G_Hair_Long_39",
                        "NFT_G_Hair_Long_40",
                        "NFT_G_Hair_Long_41",
                        "NFT_G_Hair_Long_42",
                    ])),
                    Rarity::Rare => Some(pick_random(&[
                        "NFT_G_Hair_Long_34",
                        "NFT_G_Hair_Long_35",
                        "NFT_G_Hair_Long_36",
                    ])),
                    Rarity::Epic => Some(pick_random(&[
                        "NFT_G_Hair_Long_1",
                        "NFT_G_Hair_Long_2",
                        "NFT_G_Hair_Long_3",
                        "NFT_G_Hair_Long_11",
                        "NFT_G_Hair_Long_12",
                    ])),
                    Rarity::Legendary => {
                        Some(pick_random(&["NFT_G_Hair_Long_43", "NFT_G_Hair_Long_45"]))
                    }
                    Rarity::Mythical => {
                        Some(pick_random(&["NFT_G_Hair_Long_44", "NFT_G_Hair_Long_46"]))
                    }
                }
            }
        }
    }
}

struct Body {}
impl RandomizedPart for Body {
    fn random_part(_gender: Gender) -> Option<&'static str> {
        let rarity = Rarity::pick_random_rarity();

        match rarity {
            Rarity::Common => Some(pick_random(&["NFT_Body_1", "NFT_Body_2", "NFT_Body_3"])),
            Rarity::Uncommon => None,
            Rarity::Rare => None,
            Rarity::Epic => Some(pick_random(&["NFT_Body_4"])),
            Rarity::Legendary => None,
            Rarity::Mythical => None,
        }
    }
}

struct Clothes {}
impl RandomizedPart for Clothes {
    fn random_part(gender: Gender) -> Option<&'static str> {
        match gender {
            Gender::Boy => {
                let rarity = Rarity::pick_random_rarity();

                match rarity {
                    Rarity::Common => Some(pick_random(&[
                        "NFT_B_Clothes_1",
                        "NFT_B_Clothes_2",
                        "NFT_B_Clothes_3",
                        "NFT_B_Clothes_4",
                        "NFT_B_Clothes_5",
                        "NFT_B_Clothes_6",
                    ])),
                    Rarity::Uncommon => Some(pick_random(&[
                        "NFT_B_Clothes_7",
                        "NFT_B_Clothes_8",
                        "NFT_B_Clothes_9",
                    ])),
                    Rarity::Rare => Some(pick_random(&[
                        "NFT_B_Clothes_10",
                        "NFT_B_Clothes_12",
                        "NFT_B_Clothes_17",
                    ])),
                    Rarity::Epic => Some(pick_random(&[
                        "NFT_B_Clothes_11",
                        "NFT_B_Clothes_13",
                        "NFT_B_Clothes_14",
                        "NFT_B_Clothes_15",
                        "NFT_B_Clothes_16",
                    ])),
                    Rarity::Legendary => {
                        Some(pick_random(&["NFT_B_Clothes_18", "NFT_B_Clothes_19"]))
                    }
                    Rarity::Mythical => {
                        Some(pick_random(&["NFT_B_Clothes_20", "NFT_B_Clothes_21"]))
                    }
                }
            }
            Gender::Girl => {
                let rarity = Rarity::pick_random_rarity();

                match rarity {
                    Rarity::Common => Some(pick_random(&[
                        "NFT_G_Clothes_1",
                        "NFT_G_Clothes_2",
                        "NFT_G_Clothes_3",
                        "NFT_G_Clothes_4",
                        "NFT_G_Clothes_5",
                        "NFT_G_Clothes_6",
                    ])),
                    Rarity::Uncommon => Some(pick_random(&[
                        "NFT_G_Clothes_7",
                        "NFT_G_Clothes_8",
                        "NFT_G_Clothes_12",
                        "NFT_G_Clothes_13",
                        "NFT_G_Clothes_14",
                    ])),
                    Rarity::Rare => Some(pick_random(&["NFT_G_Clothes_9", "NFT_G_Clothes_10"])),
                    Rarity::Epic => Some(pick_random(&[
                        "NFT_G_Clothes_11",
                        "NFT_G_Clothes_15",
                        "NFT_G_Clothes_16",
                        "NFT_G_Clothes_17",
                    ])),
                    Rarity::Legendary => {
                        Some(pick_random(&["NFT_G_Clothes_18", "NFT_G_Clothes_19"]))
                    }
                    Rarity::Mythical => {
                        Some(pick_random(&["NFT_G_Clothes_20", "NFT_G_Clothes_21"]))
                    }
                }
            }
        }
    }
}

struct Face {}

impl RandomizedPart for Face {
    fn random_part(gender: Gender) -> Option<&'static str> {
        match gender {
            Gender::Boy => {
                let rarity = Rarity::pick_random_rarity();

                match rarity {
                    Rarity::Common => Some(pick_random(&[
                        "NFT_B_Face_1",
                        "NFT_B_Face_2",
                        "NFT_B_Face_3",
                        "NFT_B_Face_4",
                        "NFT_B_Face_13",
                        "NFT_B_Face_14",
                    ])),
                    Rarity::Uncommon => Some(pick_random(&[
                        "NFT_B_Face_5",
                        "NFT_B_Face_6",
                        "NFT_B_Face_7",
                        "NFT_B_Face_8",
                        "NFT_B_Face_15",
                    ])),
                    Rarity::Rare => Some(pick_random(&[
                        "NFT_B_Face_9",
                        "NFT_B_Face_10",
                        "NFT_B_Face_11",
                        "NFT_B_Face_12",
                    ])),
                    Rarity::Epic => Some(pick_random(&[
                        "NFT_B_Face_16",
                        "NFT_B_Face_17",
                        "NFT_B_Face_18",
                        "NFT_B_Face_19",
                        "NFT_B_Face_23",
                        "NFT_B_Face_24",
                    ])),
                    Rarity::Legendary => Some(pick_random(&[
                        "NFT_B_Face_20",
                        "NFT_B_Face_21",
                        "NFT_B_Face_22",
                        "NFT_B_Face_25",
                        "NFT_B_Face_27",
                    ])),
                    Rarity::Mythical => Some(pick_random(&["NFT_B_Face_26", "NFT_B_Face_28"])),
                }
            }
            Gender::Girl => {
                let rarity = Rarity::pick_random_rarity();

                match rarity {
                    Rarity::Common => Some(pick_random(&[
                        "NFT_G_Face_6",
                        "NFT_G_Face_7",
                        "NFT_G_Face_8",
                        "NFT_G_Face_9",
                        "NFT_G_Face_14",
                    ])),
                    Rarity::Uncommon => Some(pick_random(&[
                        "NFT_G_Face_1",
                        "NFT_G_Face_2",
                        "NFT_G_Face_3",
                        "NFT_G_Face_4",
                        "NFT_G_Face_5",
                    ])),
                    Rarity::Rare => Some(pick_random(&[
                        "NFT_G_Face_10",
                        "NFT_G_Face_11",
                        "NFT_G_Face_12",
                        "NFT_G_Face_13",
                    ])),
                    Rarity::Epic => Some(pick_random(&[
                        "NFT_G_Face_15",
                        "NFT_G_Face_16",
                        "NFT_G_Face_17",
                    ])),
                    Rarity::Legendary => Some(pick_random(&[
                        "NFT_G_Face_18",
                        "NFT_G_Face_19",
                        "NFT_G_Face_21",
                        "NFT_G_Face_22",
                    ])),
                    Rarity::Mythical => Some(pick_random(&["NFT_G_Face_20", "NFT_G_Face_23"])),
                }
            }
        }
    }
}

struct FaceAcc {}

impl RandomizedPart for FaceAcc {
    fn random_part(_gender: Gender) -> Option<&'static str> {
        let rarity = Rarity::pick_random_rarity();

        match rarity {
            Rarity::Common => None,
            Rarity::Uncommon => Some(pick_random(&["NFT_Face_Acc_1", "NFT_Face_Acc_2"])),
            Rarity::Rare => Some(pick_random(&["NFT_Face_Acc_5", "NFT_Face_Acc_6"])),
            Rarity::Epic => Some(pick_random(&["NFT_Face_Acc_3", "NFT_Face_Acc_4"])),
            Rarity::Legendary => Some(pick_random(&["NFT_Face_Acc_7", "NFT_Face_Acc_8"])),
            Rarity::Mythical => Some(pick_random(&["NFT_Face_Acc_9"])),
        }
    }
}

struct Hair {}

impl Hair {
    fn from_hair_long(hair_long: Option<&str>) -> Option<&'static str> {
        if let Some(hair_long) = hair_long {
            let mut arr: Vec<String> = hair_long
                .to_string()
                .split("_")
                .map(|item| item.to_string())
                .collect();
            arr.remove(arr.len() - 2);

            let s = arr.join("_");
            let result = Box::leak(s.into_boxed_str());
            Some(result)
        } else {
            None
        }
    }
}
impl RandomizedPart for Hair {
    fn random_part(gender: Gender) -> Option<&'static str> {
        match gender {
            Gender::Boy => {
                let rarity = Rarity::pick_random_rarity();

                match rarity {
                    Rarity::Common => Some(pick_random(&[
                        "NFT_B_Hair_11",
                        "NFT_B_Hair_12",
                        "NFT_B_Hair_13",
                        "NFT_B_Hair_14",
                        "NFT_B_Hair_15",
                        "NFT_B_Hair_16",
                        "NFT_B_Hair_17",
                        "NFT_B_Hair_18",
                        "NFT_B_Hair_19",
                        "NFT_B_Hair_20",
                        "NFT_B_Hair_21",
                        "NFT_B_Hair_22",
                        "NFT_B_Hair_23",
                        "NFT_B_Hair_24",
                        "NFT_B_Hair_25",
                        "NFT_B_Hair_26",
                        "NFT_B_Hair_27",
                        "NFT_B_Hair_28",
                        "NFT_B_Hair_29",
                        "NFT_B_Hair_30",
                        "NFT_B_Hair_31",
                    ])),
                    Rarity::Uncommon => Some(pick_random(&[
                        "NFT_B_Hair_2",
                        "NFT_B_Hair_3",
                        "NFT_B_Hair_4",
                        "NFT_B_Hair_7",
                        "NFT_B_Hair_8",
                        "NFT_B_Hair_9",
                        "NFT_B_Hair_10",
                    ])),
                    Rarity::Rare => Some(pick_random(&[
                        "NFT_B_Hair_32",
                        "NFT_B_Hair_33",
                        "NFT_B_Hair_34",
                        "NFT_B_Hair_35",
                        "NFT_B_Hair_36",
                        "NFT_B_Hair_37",
                        "NFT_B_Hair_38",
                    ])),
                    Rarity::Epic => Some(pick_random(&[
                        "NFT_B_Hair_1",
                        "NFT_B_Hair_5",
                        "NFT_B_Hair_6",
                    ])),
                    Rarity::Legendary => Some(pick_random(&["NFT_B_Hair_39", "NFT_B_Hair_41"])),
                    Rarity::Mythical => Some(pick_random(&["NFT_B_Hair_40", "NFT_B_Hair_42"])),
                }
            }
            Gender::Girl => {
                let rarity = Rarity::pick_random_rarity();

                match rarity {
                    Rarity::Common => Some(pick_random(&[
                        "NFT_G_Hair_17",
                        "NFT_G_Hair_18",
                        "NFT_G_Hair_19",
                        "NFT_G_Hair_20",
                        "NFT_G_Hair_21",
                        "NFT_G_Hair_22",
                        "NFT_G_Hair_23",
                        "NFT_G_Hair_24",
                        "NFT_G_Hair_25",
                        "NFT_G_Hair_26",
                        "NFT_G_Hair_27",
                        "NFT_G_Hair_28",
                        "NFT_G_Hair_29",
                        "NFT_G_Hair_30",
                        "NFT_G_Hair_31",
                    ])),
                    Rarity::Uncommon => Some(pick_random(&[
                        "NFT_G_Hair_4",
                        "NFT_G_Hair_5",
                        "NFT_G_Hair_6",
                        "NFT_G_Hair_7",
                        "NFT_G_Hair_8",
                        "NFT_G_Hair_9",
                        "NFT_G_Hair_10",
                        "NFT_G_Hair_13",
                        "NFT_G_Hair_14",
                        "NFT_G_Hair_15",
                        "NFT_G_Hair_16",
                        "NFT_G_Hair_32",
                        "NFT_G_Hair_33",
                        "NFT_G_Hair_37",
                        "NFT_G_Hair_38",
                        "NFT_G_Hair_39",
                        "NFT_G_Hair_40",
                        "NFT_G_Hair_41",
                        "NFT_G_Hair_42",
                    ])),
                    Rarity::Rare => Some(pick_random(&[
                        "NFT_G_Hair_34",
                        "NFT_G_Hair_35",
                        "NFT_G_Hair_36",
                    ])),
                    Rarity::Epic => Some(pick_random(&[
                        "NFT_G_Hair_1",
                        "NFT_G_Hair_2",
                        "NFT_G_Hair_3",
                        "NFT_G_Hair_11",
                        "NFT_G_Hair_12",
                    ])),
                    Rarity::Legendary => Some(pick_random(&["NFT_G_Hair_43", "NFT_G_Hair_45"])),
                    Rarity::Mythical => Some(pick_random(&["NFT_G_Hair_44", "NFT_G_Hair_46"])),
                }
            }
        }
    }
}

struct HeadPhone {}

impl RandomizedPart for HeadPhone {
    fn random_part(_gender: Gender) -> Option<&'static str> {
        let rarity = Rarity::pick_random_rarity();

        match rarity {
            Rarity::Common => None,
            Rarity::Uncommon => Some(pick_random(&["NFT_Head_Phone_1"])),
            Rarity::Rare => Some(pick_random(&["NFT_Head_Phone_2"])),
            Rarity::Epic => Some(pick_random(&["NFT_Head_Phone_3"])),
            Rarity::Legendary => Some(pick_random(&["NFT_Head_Phone_4"])),
            Rarity::Mythical => Some(pick_random(&["NFT_Head_Phone_5"])),
        }
    }
}

fn pick_random<'a>(choices: &[&'a str]) -> &'a str {
    let weights = choices.iter().map(|_| 1).collect::<Vec<i64>>();

    let dist = WeightedIndex::new(&weights).expect("Error parsing weights");
    let mut rng = thread_rng();

    choices[dist.sample(&mut rng)]
}

enum Parts {
    Background(Background),
    Hand(Hand),
    HairLong(HairLong),
    Body(Body),
    Clothes(Clothes),
    Face(Face),
    FaceAcc(FaceAcc),
    Hair(Hair),
    HeadPhone(HeadPhone),
}

#[derive(Debug, Clone, Copy)]
enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
    Mythical,
}

impl Rarity {
    fn probability(&self) -> f64 {
        match self {
            Rarity::Common => 0.5,
            Rarity::Uncommon => 0.3,
            Rarity::Rare => 0.1,
            Rarity::Epic => 0.06,
            Rarity::Legendary => 0.03,
            Rarity::Mythical => 0.01,
        }
    }

    fn pick_random_rarity() -> Self {
        let choices = [
            Rarity::Common,
            Rarity::Uncommon,
            Rarity::Rare,
            Rarity::Epic,
            Rarity::Legendary,
            Rarity::Mythical,
        ];
        let weights: Vec<f64> = choices.iter().map(|item| item.probability()).collect();
        let dist = WeightedIndex::new(&weights).expect("Error parsing weights");
        let mut rng = thread_rng();

        choices[dist.sample(&mut rng)]
    }
}

fn get_parts_order() -> [Parts; 9] {
    [
        Parts::Background(Background {}),
        Parts::Hand(Hand {}),
        Parts::HairLong(HairLong {}),
        Parts::Body(Body {}),
        Parts::Clothes(Clothes {}),
        Parts::Face(Face {}),
        Parts::FaceAcc(FaceAcc {}),
        Parts::Hair(Hair {}),
        Parts::HeadPhone(HeadPhone {}),
    ]
}

fn is_boy(dir_path: &PathBuf) -> bool {
    let dir_str = dir_path.to_str().unwrap();
    dir_str.contains("NFT_B")
}

async fn handle(
    input_dir: &PathBuf,
    output_dir: &PathBuf,
    gender: Gender,
    total: usize,
    is_reset: bool,
) {
    std::fs::create_dir_all(output_dir).expect("Failed to create output directory");

    let mapping = (1..=total)
        .into_iter()
        .map(|i| {
            let metronion_parts = generate_random_metronion(gender);
            info!("Metronion {i:?} with parts {:?}", metronion_parts);
            metronion_parts
        })
        .collect::<Vec<Vec<String>>>();

    let tasks = mapping.iter().enumerate().map(|(i, metronion_parts)| {
        tokio::spawn({
            let input_dir = input_dir.clone();
            let output_dir = output_dir.clone();
            let metronion_parts = metronion_parts.clone();
            async move { magick_metronion(i, metronion_parts, input_dir, output_dir) }
        })
    });
    info!("Number of metronions = {:?}", mapping.len());

    // write mapping to file
    let mapping_filepath = PathBuf::from(format!("mapping_{}.txt", gender));
    if is_reset {
        fs_extra::file::remove(mapping_filepath.clone()).expect("Failed to reset mapping file");
    }
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(mapping_filepath.clone())
        .expect("Failed to open the file.");
    for (i, item) in mapping.iter().enumerate() {
        writeln!(file, "{},{:?}", i + 1, item).expect("Failed to write");
    }
    info!("Write metronion mappings to file {mapping_filepath:?}");

    let mut stream = futures::stream::iter(tasks).buffered(BUFFER_SIZE);

    while let Some(result) = stream.next().await {}
}

async fn generate_emotions(
    mapping_file: &PathBuf,
    input_dir: &PathBuf,
    output_dir: &PathBuf,
    from_index: usize,
) {
    let file = File::open(mapping_file).expect("Failed to open the mapping file.");
    let reader = BufReader::new(file);

    info!("From index {from_index:?}");

    let metronion_parts = reader
        .lines()
        .into_iter()
        .filter_map(|line| {
            let mut line = line.unwrap();
            line = line
                .replace(" ", "")
                .replace("\"", "")
                .replace("[", "")
                .replace("]", "");

            let mut line = line
                .split(",")
                .map(|item| item.to_string())
                .collect::<Vec<String>>();

            line.remove(0);
            // remove Face and FaceAcc
            line.retain(|item| !item.contains("Face"));
            Some(line)
        })
        .collect::<Vec<Vec<String>>>();

    let extended_metronion_parts = metronion_parts
        .iter()
        .enumerate()
        .filter_map(|(index, item)| {
            if index < from_index + 1 {
                return None;
            }
            // add emotions
            let mut result: Vec<Vec<String>> = vec![];
            for i in 1..=10 {
                if let Some(index) = item.iter().position(|item| item.contains("Clothes")) {
                    let mut res = item.clone();
                    res.insert(index + 1, format!("NFT_Emo_{i}"));
                    result.push(res);
                }
            }
            Some((index, result))
        })
        .collect::<Vec<(usize, Vec<Vec<String>>)>>();

    // for metronion in metronion_parts {
    //     info!("{:?}", metronion);
    // }

    let tasks = extended_metronion_parts
        .into_iter()
        .map(|(i, metronion_parts)| {
            tokio::spawn({
                let input_dir = input_dir.clone();
                let output_dir = output_dir.clone();
                let metronion_parts = metronion_parts.clone();
                async move {
                    for parts in metronion_parts {
                        magick_emotions(i, parts, input_dir.clone(), output_dir.clone());
                    }
                }
            })
        });
    info!("Number of metronions = {:?}", metronion_parts.len());

    let mut stream = futures::stream::iter(tasks).buffered(BUFFER_SIZE);

    while let Some(result) = stream.next().await {}
}

fn split_at_first<'a>(input: &'a str, pattern: &'a str) -> (&'a str, &'a str) {
    if let Some(pos) = input.find(pattern) {
        let (first, second) = input.split_at(pos);
        let second = &second[pattern.len()..];
        (first, second)
    } else {
        (input, "")
    }
}

fn magick_metronion(index: usize, parts: Vec<String>, input_dir: PathBuf, output_dir: PathBuf) {
    let output_file = output_dir.join(format!("{index:}.png"));
    let output_file_path = output_file.to_str().unwrap();

    let inputs_path = parts
        .into_iter()
        .map(|part| {
            let p = input_dir.join(format!("{part:}.png"));
            p.to_str().unwrap().to_string()
        })
        .collect::<Vec<String>>();

    let output = std::process::Command::new("magick")
        .args(&["convert"])
        .args(inputs_path)
        .args(&["-background", "none", "-flatten", output_file_path])
        .output()
        .expect("Failed to execute command magic k");

    if output.status.success() {
        info!("Generate metronion {index:?} successfully!");
    } else {
        error!("Command failed with exit code: {}", output.status);
    }
}

fn magick_emotions(index: usize, parts: Vec<String>, input_dir: PathBuf, output_dir: PathBuf) {
    let output_dir = output_dir.join(format!("{index:}"));
    std::fs::create_dir_all(&output_dir).expect("Failed to create output directory");

    let emo_part_index = parts
        .iter()
        .position(|item| item.contains("NFT_Emo"))
        .unwrap();
    let emo_part = &parts[emo_part_index].to_owned();

    let output_file = output_dir.join(format!("{index:}_{emo_part:}.png"));
    let output_file_path = output_file.to_str().unwrap();

    let inputs_path = parts
        .into_iter()
        .map(|part| {
            let p = input_dir.join(format!("{part:}.png"));
            p.to_str().unwrap().to_string()
        })
        .collect::<Vec<String>>();

    let output = std::process::Command::new("magick")
        .args(&["convert"])
        .args(inputs_path)
        .args(&["-background", "none", "-flatten", output_file_path])
        .output()
        .expect("Failed to execute command magic k");

    if output.status.success() {
        info!(
            "Generate metronion {index:?} with emotions {:?} successfully!",
            emo_part
        );
    } else {
        error!("Command failed with exit code: {}", output.status);
    }
}

fn generate_random_metronion(gender: Gender) -> Vec<String> {
    let mut hair_long_part_str: Option<&str> = None;
    let mut metronion_parts: Vec<String> = vec![];

    for part in get_parts_order() {
        let part_str = match part {
            Parts::Background(_) => ensure_part(|| Background::random_part(gender)),
            Parts::Hand(_) => ensure_part(|| Hand::random_part(gender)),
            Parts::HairLong(_) => {
                hair_long_part_str = ensure_part(|| HairLong::random_part(gender));
                hair_long_part_str
            }
            Parts::Body(_) => ensure_part(|| Body::random_part(gender)),
            Parts::Clothes(_) => ensure_part(|| Clothes::random_part(gender)),
            Parts::Face(_) => ensure_part(|| Face::random_part(gender)),
            Parts::FaceAcc(_) => {
                if matches!(gender, Gender::Boy)
                    && hair_long_part_str.is_some()
                    && !HairLong::is_with_face_acc(hair_long_part_str.unwrap())
                {
                    None
                } else {
                    FaceAcc::random_part(gender)
                }
            }
            Parts::Hair(_) => Hair::from_hair_long(hair_long_part_str),
            Parts::HeadPhone(_) => {
                if hair_long_part_str.is_some()
                    && !HairLong::is_with_headphone(gender, hair_long_part_str.unwrap())
                {
                    None
                } else {
                    HeadPhone::random_part(gender)
                }
            }
        };

        if let Some(part_str) = part_str {
            metronion_parts.push(part_str.to_string());
        }
    }

    metronion_parts
}

// always return Some(str)
fn ensure_part<F>(f: F) -> Option<&'static str>
where
    F: Fn() -> Option<&'static str>,
{
    loop {
        if let Some(result) = f() {
            return Some(result);
        }
    }
}
