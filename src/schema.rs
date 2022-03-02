table! {
    history (num) {
        num -> BigInt,
        login -> Text,
        request -> Text,
        timestamp -> Timestamp,
    }
}

table! {
    users (login) {
        login -> Text,
        password -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    history,
    users,
);
