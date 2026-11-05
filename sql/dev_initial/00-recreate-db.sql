-- DEV ONLY - Brute Force DROP DB (for local dev and unit test)
SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE
 usename = 'golion_user' OR datname = 'golion_db';
DROP DATABASE IF EXISTS golion_db;
DROP USER IF EXISTS golion_user;

-- DEV ONLY - Dev only password (for local dev and unit test).
CREATE USER golion_user PASSWORD 'devonlypassword';
CREATE DATABASE golion_db owner golion_user ENCODING = 'UTF-8';
