// @generated automatically by Diesel CLI.

diesel::table! {
    quotes (id) {
        id -> Int4,
        datetime -> Timestamptz,
        quote -> Text,
        sayer -> Text,
        invoker -> Nullable<Text>,
        stream_category -> Nullable<Text>,
        stream_title -> Nullable<Text>,
        stream_secs -> Nullable<Int8>,
    }
}
