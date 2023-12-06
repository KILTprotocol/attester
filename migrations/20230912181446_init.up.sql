CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
 
CREATE TYPE tx_states AS ENUM ('Succeeded', 'Failed', 'Pending', 'InFlight' ); 

CREATE TABLE IF NOT EXISTS attestation_requests (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()  NOT NULL,
    approved BOOLEAN DEFAULT false  NOT NULL,
    created_at TIMESTAMP DEFAULT now()  NOT NULL,
    deleted_at TIMESTAMP,
    updated_at TIMESTAMP,
    approved_at TIMESTAMP,
    revoked_at TIMESTAMP,
    ctype_hash VARCHAR(255)  NOT NULL,
    credential jsonb  NOT NULL,
    claimer VARCHAR(255)  NOT NULL,
    revoked BOOLEAN DEFAULT false NOT NULL,
    tx_state tx_states DEFAULT 'Pending'
);

CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.triggers WHERE event_object_table = 'attestation_requests' AND trigger_name = 'update') THEN
        CREATE TRIGGER update
        BEFORE UPDATE ON attestation_requests
        FOR EACH ROW
        EXECUTE FUNCTION update_updated_at();
    END IF;
END $$;

CREATE TABLE IF NOT EXISTS session_request (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4() NOT NULL
);
