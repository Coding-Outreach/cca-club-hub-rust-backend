-- Club Table (List of all the Clubs) --
CREATE TABLE IF NOT EXISTS clubs
(
    id            INT          NOT NULL PRIMARY KEY,
    username      VARCHAR(200) NOT NULL,
    email         VARCHAR(200) NOT NULL,
    password_hash VARCHAR(200) NOT NULL,
    club_name     VARCHAR(200) NOT NULL,
    description   VARCHAR(500),
    meet_time     VARCHAR(500),
    featured      BOOLEAN      NOT NULL
);
