table! {
    posts (id) {
        id -> Uuid,
        title -> Varchar,
        text -> Text,
        author -> Uuid,
        datetime -> Timestamp,
    }
}

table! {
    users (id) {
        id -> Uuid,
        name -> Varchar,
        privileges -> Int2,
        password -> Varchar,
    }
}

joinable!(posts -> users (author));

allow_tables_to_appear_in_same_query!(
    posts,
    users,
);
