use chrono::{NaiveDate, Utc};
use chrono::prelude::*;
use crate::requester;

#[derive(Debug)]
pub struct UserAttributes {
    pub uname: String,
    
    // users should modify here according to their
    // profiles. Apperantly, a user can change their
    // time format.
    // TODO: Fetch time format from user profile, if possible.

    pub date_format: String,
    pub date_format_backup: String
}

impl UserAttributes {
    pub fn new(uname: String) -> Self {
        UserAttributes { uname, date_format: String::new(), 
                         date_format_backup: String::new() }
    }

    pub fn set_date_format(&mut self, d_format: String) {
        match parse_i32(&d_format) {
            Ok(1) => {
                self.date_format = String::from("%d-%m-%Y");
                self.date_format_backup = String::from("%m-%d-%Y");
            },
            Ok(2) => {
                self.date_format = String::from("%m-%d-%Y");
                self.date_format_backup = String::from("%d-%m-%Y");
            },
            Ok(num) => {
                panic!("User entered number that is out of range [1, 2]: {}", num); 
            },
            Err(err) => {
                panic!("Error occured during parsing {} -> {:?}", d_format, err);
            }
        };
    }
}

#[derive(Debug, Clone)]
pub struct AnimeAttributes {
    pub status                  : i32,
    pub score                   : i32,
    pub id                      : i32,
    pub num_watched_episodes    : i32,
    pub num_episodes            : i32,
    current_day                 : i32,
    anime_airing_day            : i32,
    pub is_rewatching           : bool,
    pub is_airing               : bool,
    pub title                   : String,
    pub title_eng               : String,
    // day - month - year or month - day - year
    pub start_date              : String,
}

fn parse_i32(mut value: &str) -> Result<i32, std::num::ParseIntError> {
    value = if value.chars().last() == Some(',') {
            &value[1..value.len()-1]
        } else {
            value
    };

    value.parse::<i32>()
}

impl Default for AnimeAttributes {
    fn default() -> Self {
        AnimeAttributes { status: 0, score: 0, id: 0, num_watched_episodes: 0,
                          num_episodes: 0, is_rewatching: false, is_airing: false,
                          title: String::new(), title_eng: String::new(), start_date: String::new(), 
                          current_day: Utc::now().weekday().number_from_monday() as i32,
                          anime_airing_day: 0}
    }
}

impl AnimeAttributes {
    
    pub fn new() -> Self {
        Default::default()
    }

    pub fn register_attrib(&mut self, user: &UserAttributes, keyword: &str, 
                           value: &str, value_rec: &str) -> Result<usize, Box<dyn std::error::Error>> {
        let mut i_forward = 1;

//        println!("keyword: {} value: {}", keyword, value);

        match keyword {
            "status" => {
                self.status = parse_i32(value)?;
            },
            "score" => {
                self.score = parse_i32(value)?;
            },
            "anime_id" => {
                self.id = parse_i32(value)?; 
            },
            "num_watched_episodes" => {
                self.id = parse_i32(value)?;
            },
            "anime_num_episodes" => {
                self.num_episodes = parse_i32(value)?;
            },
            "is_rewatching" => {
                self.is_rewatching = parse_i32(value)? == 1;
            },
            "anime_airing_status" => {
                self.is_airing = parse_i32(value)? == 1;
            },
            "anime_title" => {
                self.title = String::from(value_rec);
                i_forward += 1;
            },
            "anime_title_eng" => {
                self.title_eng = String::from(value_rec);
                i_forward += 1;
            },
            "anime_start_date_string" => {
                self.start_date = String::from(value_rec);     
                self.anime_airing_day = match NaiveDate::parse_from_str(&self.start_date, &user.date_format) {
                    Ok(date_parsed) => {
                        date_parsed.weekday().number_from_monday() as i32
                    },
                    Err(_) => {
                        NaiveDate::parse_from_str(&self.start_date, &user.date_format_backup)?.weekday().num_days_from_monday() as i32
                    }
                };
                
                i_forward += 1;
            },
            _ => {
            
            }
        };
        Ok(i_forward)
    }

    /// returns True if there is a possibility that the anime might
    /// be airing today. This happens due to the time zone differences.
    pub fn should_get_precise_day(&self) -> bool {
        let day_diff = self.current_day - self.anime_airing_day;
        day_diff >= 0 && day_diff <= 1 
    }

    /// update the airing date of the anime by using the datetime
    /// information present in the anime page
    pub fn update_airing_day(&mut self, shifting_day: i32) {
        self.anime_airing_day += shifting_day;
    }

    /// Return true if the anime is finished or it is airing today.
    pub fn is_airing_today(&self) -> bool {
        self.anime_airing_day == self.current_day
    }

    pub fn is_finished(&self) -> bool {
        !self.is_airing
    }
}

