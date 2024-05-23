alter table if exists foo
    rename column foo to bar;

alter table if exists bar rename column bar to foo;