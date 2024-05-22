create table if not exists foods {
    id serial primary key,
    yum_factor text not null
};

drop table if exists fun cascade;

add type if not exists spice_level as enum ('lots', 'none');

create type more_spice_level as enum ('more', 'most');

create table foobar {
    id serial primary key
}

create table if not exists foobarbaz {
    name text
}