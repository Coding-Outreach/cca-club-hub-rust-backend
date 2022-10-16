-- Club Table (List of all the Clubs) --
CREATE TABLE clubs
(
    id                  SERIAL PRIMARY KEY,
    username            VARCHAR(200) NOT NULL UNIQUE,
    email               VARCHAR(200) NOT NULL,
    password_hash       VARCHAR(200) NOT NULL,
    club_name           VARCHAR(200) NOT NULL,
    description         VARCHAR(500),
    meet_time           VARCHAR(500),
    profile_picture_url VARCHAR(200) NOT NULL,
    featured            BOOLEAN      NOT NULL
);

-- Social Media Links (Club Instagram, Discord, Classroom, etc. links) --
CREATE TABLE club_socials
(
    id               SERIAL PRIMARY KEY,
    club_id          INTEGER NOT NULL UNIQUE REFERENCES clubs,
    website          VARCHAR(200),
    google_classroom VARCHAR(200),
    discord          VARCHAR(200),
    instagram        VARCHAR(200)
);

CREATE TABLE categories
(
    id            SERIAL PRIMARY KEY,
    category_name VARCHAR(200) NOT NULL UNIQUE
);

-- Club Tags --
CREATE TABLE club_categories
(
    id          SERIAL PRIMARY KEY,
    club_id     INTEGER NOT NULL REFERENCES clubs,
    category_id INTEGER NOT NULL REFERENCES categories
);
