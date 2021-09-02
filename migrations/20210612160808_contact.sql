create table if not exists contact
(
    id           uuid        not null primary key default gen_random_uuid(),
    name         text        not null,
    sender       text        not null,
    message      text        not null,
    contacted_at timestamptz not null             default now()
);
