CREATE TABLE quotes (
    id SERIAL PRIMARY KEY,
    quote TEXT NOT NULL,
    speaker TEXT NOT NULL,
    invoker TEXT,
    stream_category TEXT,
    stream_title TEXT
)
