table! {
    history (id) {
        id -> Text,
        login -> Text,
        request -> Text,
        tms -> Timestamp,
    }
}

table! {
    users (login) {
        login -> Text,
        password -> Text,
        token -> Text,
    }
}

allow_tables_to_appear_in_same_query!(history, users,);
