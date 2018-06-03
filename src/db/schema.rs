table! {
    module (id) {
        id -> Int4,
        uuid -> Uuid,
        name -> Text,
    }
}

table! {
    pump_event (id) {
        id -> Int4,
        created -> Timestamptz,
        module_id -> Int4,
        pump_running -> Bool,
    }
}

table! {
    sample (id) {
        id -> Int4,
        created -> Timestamptz,
        module_id -> Int4,
        moisture -> Float8,
    }
}

joinable!(pump_event -> module (module_id));
joinable!(sample -> module (module_id));

allow_tables_to_appear_in_same_query!(module, pump_event, sample,);
