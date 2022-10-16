use crate::schema::*;
use diesel::prelude::*;

#[derive(Debug, Clone, Queryable, Insertable, Identifiable)]
pub struct Club {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub club_name: String,
    pub description: Option<String>,
    pub meet_time: Option<String>,
    pub profile_picture_url: String,
    pub featured: bool,
}

#[derive(Debug, Clone, Queryable, Identifiable, Associations)]
#[diesel(belongs_to(Club))]
pub struct ClubSocial {
    pub id: i32,
    pub club_id: i32,
    pub website: Option<String>,
    pub google_classroom: Option<String>,
    pub discord: Option<String>,
    pub instagram: Option<String>,
}

#[derive(Debug, Clone, Queryable, Identifiable)]
#[diesel(table_name = categories)]
pub struct Category {
    pub id: i32,
    pub category_name: String,
}

#[derive(Debug, Clone, Queryable, Identifiable, Associations, Insertable)]
#[diesel(belongs_to(Club))]
#[diesel(belongs_to(Category))]
#[diesel(table_name = club_categories)]
pub struct ClubCategory {
    pub id: i32,
    pub club_id: i32,
    pub category_id: i32,
}
