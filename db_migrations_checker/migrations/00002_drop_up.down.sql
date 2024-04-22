create table if not exists foods {
    id serial primary key,
    yum_factor text not null
};

drop table if exists fun cascade;

add type if not exists spice_level as enum ('lots', 'none');