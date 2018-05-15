table! {
    module (id) {
        id -> Int4,
        uuid -> Uuid,
        name -> Text,
    }
}

table! {
    sample (id) {
        id -> Int4,
        created -> Timestamptz,
        module_id -> Int4,
        humidity -> Float8,
        temperature -> Float8,
    }
}

joinable!(sample -> module (module_id));

allow_tables_to_appear_in_same_query!(module, sample,);
