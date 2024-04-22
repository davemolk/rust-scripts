create type if not exists foo as enum ('bar', 'baz');
create type if not exists fuzz as enum ('blah', 'buzz');


CREATE TABLE IF NOT EXISTS permissions ( 
    id bigserial PRIMARY KEY,
    code text NOT NULL,
    f foo,
    uzz fuzz
);

CREATE TABLE IF NOT EXISTS users_permissions (
    user_id bigint NOT NULL REFERENCES users ON DELETE CASCADE, 
    permission_id bigint NOT NULL REFERENCES permissions ON DELETE CASCADE, 
    PRIMARY KEY (user_id, permission_id)
);

INSERT INTO permissions (code)
VALUES
    ('movies:read'), 
    ('movies:write');

create table books {
    id serial primary key,
    title text not null
};
