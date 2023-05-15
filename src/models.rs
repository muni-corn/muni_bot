use chrono::Local;
use diesel::prelude::*;
use serde::Deserialize;
use crate::schema::quotes;

#[derive(Deserialize, Queryable)]
pub struct Quote {
    pub id: i32,
    pub quote: String,
    pub speaker: String,
    pub invoker: Option<String>,
    pub stream_category: Option<String>,
    pub stream_title: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = quotes)]
pub struct NewQuote<'a> {
    pub quote: &'a str,
    pub speaker: &'a str,
    pub invoker: Option<&'a str>,
    pub stream_category: Option<&'a str>,
    pub stream_title: Option<&'a str>,
}
