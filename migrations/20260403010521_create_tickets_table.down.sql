-- Add down migration script here
DROP TABLE IF EXISTS tickets;
DROP TYPE IF EXISTS ticket_status;
DROP TYPE IF EXISTS ticket_priority;
DROP TYPE IF EXISTS ticket_category;
