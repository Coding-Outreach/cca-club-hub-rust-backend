-- Social Media Links (Club Instagram, Discord, Classroom, etc. links) --
CREATE TABLE IF NOT EXISTS club_socials
(
    id               INT          NOT NULL PRIMARY KEY,
    club_id          INT          NOT NULL,
    website          VARCHAR(200) NOT NULL,
    google_classroom VARCHAR(200) NOT NULL,
    discord          VARCHAR(200) NOT NULL,
    instagram        VARCHAR(200) NOT NULL,
    FOREIGN KEY (club_id) REFERENCES clubs (id)
);
