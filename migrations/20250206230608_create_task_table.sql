create table task(
    task_id         uuid        primary key default uuid_generate_v1mc(),
    title           text        not null,
    description     text,
    completed       boolean     not null default false,
    created_at      timestamptz not null default now(),
    updated_at      timestamptz
);

select trigger_updated_at('task');