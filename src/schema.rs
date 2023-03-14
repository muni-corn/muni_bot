// @generated automatically by Diesel CLI.

diesel::table! {
    quotes (id) {
        id -> Int4,
        quote -> Text,
        speaker -> Text,
        invoker -> Nullable<Text>,
        stream_category -> Nullable<Text>,
        stream_title -> Nullable<Text>,
    }
}
