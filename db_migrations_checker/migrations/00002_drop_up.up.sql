drop table foods;

drop type IF EXISTS spice_level cascade;
drop type if exists more_spice_level;

create table fun {
    id serial primary key,
    destination text not null
};

drop table foobar cascade;
drop table foobarbaz;