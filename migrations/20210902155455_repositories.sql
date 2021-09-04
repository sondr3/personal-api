create table if not exists repository
(
    owner            text    not null,
    name             text    not null,
    license          text    not null,
    stars            int     not null,
    primary_language text    not null,
    languages        text [] not null,
    created_at       text    not null,
    primary key (owner, name)
)