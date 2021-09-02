create table if not exists repository
(
    name             text   not null primary key,
    name_with_owner  text   not null,
    license          text   not null,
    stars            int    not null,
    primary_language text   not null,
    languages        text[] not null,
    created_at       text   not null
)