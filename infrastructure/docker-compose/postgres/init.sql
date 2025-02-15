CREATE USER monitoring_user WITH PASSWORD 'secure_password';
GRANT SELECT ON pg_stat_activity TO monitoring_user;
GRANT pg_monitor TO monitoring_user;