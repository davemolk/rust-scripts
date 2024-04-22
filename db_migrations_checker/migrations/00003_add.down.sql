ALTER TABLE users 
    DROP column IF EXISTS age,
    drop column city,
    add column user_name text;

drop table city;