// @generated automatically by Diesel CLI.

diesel::table! {
    threads (id) {
        id -> Int4,
        title -> Text,
        content -> Text,
        reply_data -> Bytea,
        room_id -> Int4,
        time -> Timestamp,
        user_id -> Int4,
    }
}

diesel::table! {
    researcher_cards (id) {
        id -> Int4,
        affiliation -> Text,
        citedby -> Int4,
        email_domain -> Text,
        interests -> Array<Text>,
        name -> Text,
        google_scholar_publication_ids -> Array<Text>,
        google_scholar_id -> Text,
        coauthors -> Array<Text>,
    }
}

diesel::table! {
    researcher_card_threads (id) {
        id -> Int4,
        title -> Text,
        content -> Text,
        time -> Timestamp,
        reply_data -> Bytea,
        researcher_id -> Int4,
    }
}

diesel::table! {
    current_users (id) {
        id -> Int4,
        name -> Text,
        email -> Text,
        password -> Text,
        created_at -> Timestamp,
    }
}
