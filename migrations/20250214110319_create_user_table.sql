create table users(
    user_id         uuid            primary key default uuid_generate_v1mc(),
    email           text            collate "case_insensitive" unique not null,
    username        text            unique not null,
    password_hash   text            not null,
    created_at      timestamptz     not null default now(),
    updated_at      timestamptz     not null default now()
);

select trigger_updated_at('users');