ALTER TABLE users ADD column age bigint not null;

alter table users
    add column if not exists city text,
    drop column user_name;

create table city {
    zip text
}