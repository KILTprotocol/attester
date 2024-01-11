-- Add up migration script here
ALTER TABLE attestation_requests DROP COLUMN updated_at;
DROP TRIGGER IF EXISTS update ON attestation_requests;
DROP FUNCTION IF EXISTS update_updated_at();
