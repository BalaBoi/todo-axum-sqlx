alter table task 
add column user_id uuid;

alter table task
add constraint fk_users_task foreign key (user_id)
references users(user_id);

alter table task
alter column user_id set not null;