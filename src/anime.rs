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

    // a day can shift based on the airing hour.
    // we need to calculate the time difference between
    // JST to local time zone. Thus, shifting day can be
    // 0, 1 or -1 and affects the anime's airing day.
    shifting_day                : i32,

    pub is_rewatching           : bool,
    pub airing_status           : bool,
    pub is_airing_today         : bool,
    pub title                   : String,
    // day - month - year or month - day - year
    pub start_date              : String,
}

fn parse_i32(value: &str) -> Result<i32, Box<dyn std::error::Error>> {
    if value.len() > 1 {
        let val_range  = 1..value.len() - 1;
        Ok(value[val_range].parse::<i32>()?)
    } else {
        Ok(value.parse::<i32>()?)
    }
}

impl Default for AnimeAttributes {
    fn default() -> Self {
        AnimeAttributes { status: 0, score: 0, id: 0, num_watched_episodes: 0,
                          num_episodes: 0, shifting_day: 0, 
                          is_rewatching: false, airing_status: false,
                          is_airing_today: false, title: String::new(),
                          start_date: String::new(), 
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
                self.airing_status = parse_i32(value)? == 1;
            },
            "anime_title" => {
                self.title = String::from(value_rec);
                i_forward += 1;
            },
            "anime_start_date_string" => {
                // println!("***\nvalue: {}, value_rec: {}", value, value_rec);
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

    pub fn update_airing_date(&mut self) {
        if (self.current_day - self.anime_airing_day).abs() <= 1 {
            self.shifting_day = requester::get_animehour_diff(self.id).unwrap();
            self.anime_airing_day += self.shifting_day;
        }
        self.is_airing_today = self.current_day == self.anime_airing_day;
    }
}

