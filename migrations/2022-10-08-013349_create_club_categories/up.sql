CREATE TABLE IF NOT EXISTS categories
(
    id            INT          NOT NULL PRIMARY KEY,
    category_name VARCHAR(200) NOT NULL
);

-- Club Tags --
CREATE TABLE IF NOT EXISTS club_categories
(
    id          INT NOT NULL PRIMARY KEY,
    club_id     INT NOT NULL,
    category_id INT NOT NULL,
    FOREIGN KEY (club_id) REFERENCES clubs (id),
    FOREIGN KEY (category_id) REFERENCES categories (id)
);
