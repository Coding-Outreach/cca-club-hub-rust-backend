CREATE TABLE IF NOT EXISTS reset_password_requests
(
    id              INT          NOT NULL PRIMARY KEY,
    club_id         INT          NOT NULL,
    reset_code      VARCHAR(200) NOT NULL,
    expiration_date TIMESTAMP    NOT NULL,
    FOREIGN KEY (club_id) REFERENCES clubs (id)
);
