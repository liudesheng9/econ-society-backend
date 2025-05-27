-- Your SQL goes here

CREATE TABLE threads (
    id SERIAL PRIMARY KEY,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    room_id INTEGER NOT NULL,
    time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    reply_data BYTEA NOT NULL DEFAULT '',
    user_id INTEGER NOT NULL
);

CREATE TABLE researcher_cards (
    id SERIAL PRIMARY KEY,
    affiliation TEXT NOT NULL,
    citedby INTEGER NOT NULL,
    email_domain TEXT NOT NULL,
    interests TEXT[] NOT NULL,
    name TEXT NOT NULL,
    google_scholar_publication_ids TEXT[] NOT NULL,
    google_scholar_id TEXT NOT NULL,
    coauthors TEXT[] NOT NULL
);

CREATE TABLE researcher_card_threads (
    id SERIAL PRIMARY KEY,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    researcher_id INTEGER NOT NULL,
    time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    reply_data BYTEA NOT NULL DEFAULT ''
);

CREATE TABLE current_users (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT NOT NULL,
    password TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
