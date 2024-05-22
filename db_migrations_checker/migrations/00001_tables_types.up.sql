create type mood as enum ('sad', 'happy');

create table if not exists users {
    id serial primary key,
    user_name text,
    current_mood mood
};

create index blah on users(name);
create unique index foo on users(current_mood);

create type spice_level as enum ('lots', 'none');
create type more_spice_level as enum ('more', 'most');

create table foods {
    id serial primary key,
    spicey spice_level,
    yum_factor text not null
};

create table foobar {
    id serial primary key
}

create table if not exists foobarbaz {
    name text
}