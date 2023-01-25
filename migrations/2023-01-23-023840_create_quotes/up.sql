CREATE TABLE quotes (
    id SERIAL PRIMARY KEY,
    datetime TIMESTAMP WITH TIME ZONE NOT NULL,
    quote TEXT NOT NULL,
    sayer TEXT NOT NULL,
    invoker TEXT,
    stream_category TEXT,
    stream_title TEXT,
    stream_secs BIGINT
)
