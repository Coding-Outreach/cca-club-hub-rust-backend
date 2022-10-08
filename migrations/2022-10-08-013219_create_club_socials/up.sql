-- Social Media Links (Club Instagram, Discord, Classroom, etc. links) --
CREATE TABLE IF NOT EXISTS club_socials
(
    id          INT          NOT NULL PRIMARY KEY,
    club_id     INT          NOT NULL,
    social_name VARCHAR(200) NOT NULL,
    social_link VARCHAR(200) NOT NULL,
    FOREIGN KEY (club_id) REFERENCES clubs (id)
);
