-- Posts List --
CREATE TABLE IF NOT EXISTS posts
(
    id           INT          NOT NULL PRIMARY KEY,
    club_id      INT          NOT NULL,
    title        VARCHAR(200) NOT NULL,
    text_content VARCHAR(500),
    media_url    VARCHAR(500),
    FOREIGN KEY (club_id) REFERENCES clubs (id)
);
