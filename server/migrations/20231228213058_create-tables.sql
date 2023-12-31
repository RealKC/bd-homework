CREATE TABLE Authors(
    author_id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    date_of_birth INTEGER NOT NULL,
    date_of_death INTEGER,
    description TEXT NOT NULL,
    CHECK (date_of_death IS NULL OR date_of_death > date_of_birth)
) STRICT;

CREATE TABLE Books(
    book_id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    author_id INTEGER NOT NULL,
    publish_date INTEGER NOT NULL,
    publisher TEXT NOT NULL,
    count INTEGER NOT NULL CHECK (count >= 0),
    synopsis TEXT NOT NULL,
    language TEXT NOT NULL CHECK (language IN ('ro', 'en')),
    FOREIGN KEY (author_id) REFERENCES Authors(author_id)
) STRICT;

CREATE TABLE Users(
    user_id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    type INTEGER NOT NULL CHECK (type = 1 OR type = 2), -- regular user = 1, librarian = 2
    email TEXT UNIQUE NOT NULL,
    password TEXT NOT NULL
) STRICT;

CREATE TABLE Borrows(
    borrow_id INTEGER PRIMARY KEY AUTOINCREMENT,
    book_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    FOREIGN KEY (book_id) REFERENCES Books(book_id),
    FOREIGN KEY (user_id) REFERENCES Users(user_id)
) STRICT;

CREATE TABLE BorrowData(
    borrow_id INTEGER NOT NULL UNIQUE,
    valid_until INTEGER NOT NULL, -- this is actually a date
    chapters_read INTEGER NOT NULL,
    FOREIGN KEY (borrow_id) REFERENCES Borrows(borrow_id)
) STRICT;
