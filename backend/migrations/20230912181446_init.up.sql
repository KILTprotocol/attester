
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
 
CREATE TABLE attestation_requests (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()  NOT NULL,
    approved BOOLEAN DEFAULT false  NOT NULL,
    created_at TIMESTAMP DEFAULT now()  NOT NULL,
    deleted_at TIMESTAMP,
    updated_at TIMESTAMP,
    ctype_hash VARCHAR(255)  NOT NULL,
    credential jsonb  NOT NULL,
    claimer VARCHAR(255)  NOT NULL,
    revoked BOOLEAN DEFAULT false NOT NULL
);


CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update
BEFORE UPDATE ON attestation_requests
FOR EACH ROW
EXECUTE FUNCTION update_updated_at();