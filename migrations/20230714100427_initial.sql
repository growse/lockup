-- Add migration script here
 create table things (id integer primary key autoincrement, url text not null , added integer not null, tags text not null, type text check(type in ('regular','youtube', 'podcast')) not null default 'regular');
