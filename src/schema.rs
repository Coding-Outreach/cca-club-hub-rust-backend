// @generated automatically by Diesel CLI.

diesel::table! {
    categories (id) {
        id -> Int4,
        category_name -> Varchar,
    }
}

diesel::table! {
    club_categories (id) {
        id -> Int4,
        club_id -> Int4,
        category_id -> Int4,
    }
}

diesel::table! {
    club_socials (id) {
        id -> Int4,
        club_id -> Int4,
        website -> Nullable<Varchar>,
        google_classroom -> Nullable<Varchar>,
        discord -> Nullable<Varchar>,
        instagram -> Nullable<Varchar>,
    }
}

diesel::table! {
    clubs (id) {
        id -> Int4,
        username -> Varchar,
        email -> Varchar,
        password_hash -> Varchar,
        club_name -> Varchar,
        description -> Nullable<Varchar>,
        meet_time -> Nullable<Varchar>,
        profile_picture_url -> Varchar,
        featured -> Bool,
    }
}

diesel::joinable!(club_categories -> categories (category_id));
diesel::joinable!(club_categories -> clubs (club_id));
diesel::joinable!(club_socials -> clubs (club_id));

diesel::allow_tables_to_appear_in_same_query!(categories, club_categories, club_socials, clubs,);
