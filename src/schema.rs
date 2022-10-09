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
        social_name -> Varchar,
        social_link -> Varchar,
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
        profile_picture_url -> Nullable<Varchar>,
        featured -> Bool,
    }
}

diesel::table! {
    posts (id) {
        id -> Int4,
        club_id -> Int4,
        title -> Varchar,
        text_content -> Nullable<Varchar>,
        media_url -> Nullable<Varchar>,
    }
}

diesel::table! {
    reset_password_requests (id) {
        id -> Int4,
        club_id -> Int4,
        reset_code -> Varchar,
        expiration_date -> Timestamp,
    }
}

diesel::joinable!(club_categories -> categories (category_id));
diesel::joinable!(club_categories -> clubs (club_id));
diesel::joinable!(club_socials -> clubs (club_id));
diesel::joinable!(posts -> clubs (club_id));
diesel::joinable!(reset_password_requests -> clubs (club_id));

diesel::allow_tables_to_appear_in_same_query!(
    categories,
    club_categories,
    club_socials,
    clubs,
    posts,
    reset_password_requests,
);
