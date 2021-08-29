use std::io::prelude::*;

mod trie;
mod anime;
mod requester;
mod config;
mod dns;

use trie::Trie;
use anime::{UserAttributes, AnimeAttributes};


fn main() {
    // there should be an infinite loop that accepts username
    // and constructs their watching animelist.

    // create a trie object and register key words that are valid for
    // this operation.
    let registered_words = vec!["status", "score", "is_rewatching", 
                                "anime_airing_status", "anime_id", "anime_title",
                                "anime_start_date_string", "anime_num_episodes"];
    let registered_trie = Trie::new(Some(&registered_words)); 
    
    let mut u_name = String::new();
    let mut date_format = String::new();

    loop {
        u_name = String::new();
        date_format = String::new();

        println!("Enter User Name: ");
        match std::io::stdin().read_line(&mut u_name) {
            Ok(_) => {
                u_name = String::from(u_name.trim());
            },
            Err(err) => {
                println!("user did not enter a valid input");
                println!("Following error occured: {}", err);
                continue;
            }
        };
        let mut user_attrib = UserAttributes::new(u_name.clone());
        println!("\nSelect a time format from following options:\n");
        println!("1 -> Day - Month - Year \t 2 -> Month - Day - Year");
        println!("Example input for Day - Month - Year: 1");
        match std::io::stdin().read_line(&mut date_format) {
            Ok(size) => {
                date_format = String::from(date_format.trim());
                if date_format.len() != 1 {
                    println!("User did not enter a valid input");
                    continue;
                }
                user_attrib.set_date_format(date_format.clone());
            },
            Err(err) => {
                println!("user did not enter a valid input");
                println!("Following error occured: {}", err);
                continue;
            }
        };
        match requester::get_animelist(&user_attrib, &registered_trie) {
            Ok(anime_list) => {
                for anime_entry in anime_list {
                    if anime_entry.is_airing_today {
                        println!("***Anime {} is airing TODAY!***", anime_entry.title);
                        println!("{:?}", anime_entry);
                    }
                }
            },
            Err(err) => {
                println!("Error: {:?}", err);
            }
        }
    }
}
