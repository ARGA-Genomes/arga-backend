CREATE TABLE users (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name varchar(255) NOT NULL,
    email varchar(255) NOT NULL UNIQUE,
    user_role varchar(255) NOT NULL,
    password_hash varchar(255) NOT NULL,
    password_salt varchar(255) NOT NULL
);
