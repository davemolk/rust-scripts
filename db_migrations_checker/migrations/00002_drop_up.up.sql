drop table foods;

drop type IF EXISTS spice_level;

create table fun {
    id serial primary key,
    destination text not null
};
