-- Add migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
DROP TYPE IF EXISTS format CASCADE;
CREATE TYPE format AS ENUM (
    'ONLINE',
    'OFFLINE'
);

CREATE TABLE Appointment (
    id uuid DEFAULT uuid_generate_v4() PRIMARY KEY,
    title VARCHAR(255),
    description TEXT,
    format format NOT NULL,
    address VARCHAR(255) DEFAULT NULL,
    link VARCHAR(255) DEFAULT NULL,
    date TIMESTAMP NOT NULL,
    duration BIGINT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE Invitation (
    id uuid PRIMARY KEY,
    appointment_id UUID REFERENCES Appointment(id) ON DELETE CASCADE,
    used BOOLEAN DEFAULT false,
    short_url VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE Emails (
    id SERIAL PRIMARY KEY,
    invitation_id UUID REFERENCES Invitation(id) ON DELETE CASCADE,
    receiver VARCHAR(255),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

