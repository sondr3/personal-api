create table if not exists repository
(
    name             text      not null primary key,
    repository       text      not null,
    license          text      not null,
    stars            int       not null,
    primary_language text      not null,
    languages        text[]    not null,
    created_at       timestamp not null
)