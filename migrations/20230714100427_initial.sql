create table things
(
    id    integer primary key autoincrement,
    url   text                                                                  not null,
    added datetime                                                              not null default CURRENT_TIMESTAMP,
    type  text check (type in ('article', 'youtube', 'podcast', 'rss', 'file')) not null default 'article'
);

CREATE UNIQUE INDEX idx_things_url ON things (url);

CREATE TABLE tags
(
    id  integer primary key autoincrement,
    tag text not null
);

CREATE UNIQUE INDEX idx_tags_tag ON tags (tag);

CREATE TABLE thing_tags
(
    thing integer not null,
    tag   integer not null,
    foreign key (thing) references things (id),
    foreign key (tag) references tags (id)
);

CREATE UNIQUE INDEX idx_thing_tags ON thing_tags (thing, tag);
