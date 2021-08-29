// write a parser here, and also get the request functions in this
// module. So that we can call them from trie.

use std::io::prelude::*;
use std::net::{TcpStream};
use std::sync::{Mutex, Arc};
use std::thread;
use crate::trie::Trie;
use crate::anime::{AnimeAttributes, UserAttributes};
use crate::config::{TIME_DIFF_TO_JST, HOUR_IDENTIFIER};

extern crate reqwest;


fn parse_animepage_body(body: String) -> i32 {
    // we are only interested in the airing hour of the anime, so that we
    // can deduce the airing day correctly. Hence, I do not provide a proper
    // html parser. Despite that, the operations performed here are too
    // expensive. TODO: Improve HTML PARSER. 
    let tokenized_body: Vec<&str> = body.split('\n').collect();
    let mut index = 0;
    let mut shifting = 0;
    while index < tokenized_body.len() {
        let line = tokenized_body[index];
        let length_line = line.len();
        if length_line  >= 44 && length_line < 48 && line[28..].contains(HOUR_IDENTIFIER) {
            let date_tokenized: Vec<&str> = tokenized_body[index + 1]
                                                    .trim().split(' ').collect();
            let hour_min_tokenized: Vec<&str> = date_tokenized[2].split(':').collect();
            let hour = hour_min_tokenized[0].parse::<i32>().unwrap();
            let min  = hour_min_tokenized[1].parse::<i32>().unwrap();

            if TIME_DIFF_TO_JST < 0  && hour + TIME_DIFF_TO_JST  >= 24 {
                shifting = 1; 
            } else if TIME_DIFF_TO_JST > 0 && hour - TIME_DIFF_TO_JST < 0 {
                shifting = -1;
            }
            break;

        }
        index += 1;
    }
    shifting
}

pub fn get_animehour_diff(anime_id: i32) -> Result<i32, Box<dyn std::error::Error>> {
    // this function performs a GET request using reqwest to the 
    // myanimelist page of given anime_id. Once obtained, it parses the
    // page and extracts the start date.
    let url = format!("https://myanimelist.net/anime/{}/", anime_id);
    let mut res = reqwest::blocking::get(url)?;
    let body = res.text()?;
    Ok(parse_animepage_body(body))
}

fn parse_animelist_body(body: String, user_attrib: &UserAttributes, 
            registered_words: &Trie) -> Vec<Arc<Mutex<Vec<AnimeAttributes>>>> {
    // split the body of the html file by end of line
    // character. Then, traverse through the vector
    // and seek for <table class="list-table" data-items="[
    // Once found, shrink the line via: s[..-3]
    let mut tokenized_body: Vec<&str> = body.split('\n').collect();

    let target = "<table class=\"list-table\" data-items=\"[";
    let t_len = target.len();
    let mut raw_anime_list = "";
    for tk in tokenized_body.iter() {
        let trimmed_tk = tk.trim();
        let tk_len = trimmed_tk.len();
        if tk_len >= t_len {
            if &trimmed_tk[..t_len] == target {
                raw_anime_list = &trimmed_tk[t_len..tk_len-3];
                break;
            }
        }
    }
    tokenized_body = raw_anime_list.split("&quot;").collect(); 
    let mut index = 0;
    let mut anime_list: Vec<Arc<Mutex<Vec<AnimeAttributes>>>> = Vec::new();
    let mut current_anime_entry = AnimeAttributes::new();
    let mut current_chunk: Vec<AnimeAttributes> = Vec::new();
    let mut ignore_enabled = false;

    let num_threads: usize = 4;
    let chunk_size: usize = std::cmp::max(1, anime_list.len() / num_threads);

    while index < tokenized_body.len() {
        let token = tokenized_body[index]; 
        
        if !ignore_enabled && (token == "}" || token == "},{") {
            // this concludes an anime entry.
            current_chunk.push(current_anime_entry);
            if (index + 1) % chunk_size == 0 {
                anime_list.push(Arc::new(Mutex::new(current_chunk)));
                current_chunk = Vec::new();
            }
            current_anime_entry = AnimeAttributes::new();
        } else if token == ":[{" || token == ":{" {
            ignore_enabled = true;
        } else if token == "}]," || token == "}," {
            ignore_enabled = false;
        } else if !ignore_enabled && registered_words.contains_word(token){
            // check if this word is registered.
            match current_anime_entry.register_attrib(user_attrib, token, 
                        tokenized_body[index + 1], tokenized_body[index + 2]) {
                Ok(i_forward) => {
                    index += i_forward;
                },
                Err(err) => {
                    println!("Error occured: {}", err);
                    panic!("Error while inserting following token: {}, index:{} len:{} ", 
                            token, index, tokenized_body.len());
                }
            }
            index += 1;
        }
        
        index += 1;
    }
    anime_list 
}


pub fn get_animelist(user_attrib: &UserAttributes, 
        registered_words: &Trie) -> Result<Vec<AnimeAttributes>, 
                                           Box<dyn std::error::Error>> {
    // let res = fetch_page("https://myanimelist.net:8080");
    let url = format!("https://myanimelist.net/animelist/{}?status=1", &user_attrib.uname);
    let mut res = reqwest::blocking::get(url)?;
    let body = res.text()?;
    let mut anime_list = parse_animelist_body(body, 
                                              user_attrib, registered_words);

    let mut result: Vec<AnimeAttributes> = Vec::new();
    if anime_list.len() == 0 {
        return Ok(result);
    }
     
    let mut threads = Vec::new();
    for chunk in anime_list.iter_mut() {
        let cloned_chunk = Arc::clone(&chunk);
        let handle = thread::spawn(move || {
            for anime in cloned_chunk.lock().unwrap().iter_mut() {
                anime.update_airing_date();
            }
        });
        threads.push(handle);
    }
    for th in threads {
        th.join().unwrap();
    }
    for chunk in anime_list.iter() {
        for anime in chunk.lock().unwrap().iter() {
            result.push(anime.clone());
        }
    }
    Ok(result)
}

