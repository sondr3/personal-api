CREATE TABLE IF NOT EXISTS contact
(
    id           uuid        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    name       text        not null,
    sender       text        not null,
    message      text        not null,
    contacted_at timestamptz NOT NULL             DEFAULT now()
);
