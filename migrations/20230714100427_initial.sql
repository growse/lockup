-- Add migration script here
create table things
(
    id    integer primary key autoincrement,
    url   text                                                   not null,
    added integer                                                not null default CURRENT_TIMESTAMP,
    tags  text                                                   not null default "",
    type  text check (type in ('regular', 'youtube', 'podcast')) not null default 'regular'
);

CREATE UNIQUE INDEX idx_things_url ON things (url);