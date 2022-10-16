use crate::schema::*;
use chrono::{DateTime, Utc};
use diesel::prelude::*;

#[derive(Queryable, Insertable, Identifiable)]
pub struct Club {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub club_name: String,
    pub description: Option<String>,
    pub meet_time: Option<String>,
    pub profile_picture_url: Option<String>,
    pub featured: bool,
}

#[derive(Queryable, Identifiable, Associations)]
#[diesel(belongs_to(Club))]
pub struct ClubSocial {
    pub id: i32,
    pub club_id: i32,
    pub social_name: String,
    pub social_link: String,
}

#[derive(Queryable, Identifiable)]
#[diesel(table_name = categories)]
pub struct Category {
    pub id: i32,
    pub category_name: String,
}

#[derive(Queryable, Identifiable, Associations, Insertable)]
#[diesel(belongs_to(Club))]
#[diesel(belongs_to(Category))]
#[diesel(table_name = club_categories)]
pub struct ClubCategory {
    pub id: i32,
    pub club_id: i32,
    pub category_id: i32,
}

#[derive(Queryable, Identifiable, Associations)]
#[diesel(belongs_to(Club))]
pub struct Post {
    pub id: i32,
    pub club_id: i32,
    pub title: String,
    pub text_content: String,
    pub media_url: String,
}

#[derive(Queryable, Identifiable, Associations)]
#[diesel(belongs_to(Club))]
pub struct ResetPasswordRequest {
    pub id: i32,
    pub club_id: i32,
    pub reset_code: String,
    pub expiration_date: DateTime<Utc>,
}
