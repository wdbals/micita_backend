-- Tipos enumerados para estandarizar opciones
CREATE TYPE user_role AS ENUM ('veterinarian', 'assistant', 'admin');

CREATE TYPE animal_species AS ENUM (
    'dog',
    'cat',
    'bird',
    'reptile',
    'rodent',
    'rabbit',
    'other'
);

CREATE TYPE animal_gender AS ENUM ('male', 'female', 'unknown');

CREATE TYPE appointment_status AS ENUM ('scheduled', 'completed', 'canceled', 'no_show');

CREATE TYPE procedure_type AS ENUM (
    'vaccine',
    'surgery',
    'deworming',
    'test',
    'grooming',
    'other'
);

-- Veterinarios y Staff
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    name VARCHAR(100) NOT NULL,
    role user_role NOT NULL,
    license_number VARCHAR(50),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
);

-- Dueños de Mascotas
CREATE TABLE clients (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    email VARCHAR(255) UNIQUE,
    phone VARCHAR(20) NOT NULL,
    address TEXT,
    notes TEXT,
    assigned_to INTEGER REFERENCES users(id) ON DELETE SET NULL
);

-- Función de validación CORREGIDA
CREATE OR REPLACE FUNCTION validate_assigned_role()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.assigned_to IS NOT NULL AND NOT EXISTS (
        SELECT 1 FROM users
        WHERE id = NEW.assigned_to
        AND role IN ('veterinarian', 'assistant')
    ) THEN
        RAISE EXCEPTION 'El usuario asignado (ID: %) no es veterinario ni asistente', NEW.assigned_to;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger
CREATE TRIGGER trg_validate_assigned_role
BEFORE INSERT OR UPDATE ON clients
FOR EACH ROW EXECUTE FUNCTION validate_assigned_role();

-- Animales
CREATE TABLE breeds (
    id SERIAL PRIMARY KEY,
    species animal_species NOT NULL,
    name VARCHAR(50) NOT NULL,
    UNIQUE (species, name)
);

CREATE TABLE patients (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    species animal_species NOT NULL,
    breed INTEGER REFERENCES breeds(id) ON DELETE SET NULL,
    birth_date DATE,
    gender animal_gender,
    weight_kg DECIMAL(5, 2),
    client_id INTEGER REFERENCES clients(id) ON DELETE CASCADE,
    photo_url TEXT
);

-- Función de validación para especie-raza
CREATE OR REPLACE FUNCTION validate_breed_species()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.breed IS NOT NULL AND NOT EXISTS (
        SELECT 1 FROM breeds
        WHERE id = NEW.breed
        AND species = NEW.species
    ) THEN
        RAISE EXCEPTION 'La raza (ID: %) no corresponde a la especie (%)', NEW.breed, NEW.species;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger
CREATE TRIGGER trg_validate_breed_species
BEFORE INSERT OR UPDATE ON patients
FOR EACH ROW EXECUTE FUNCTION validate_breed_species();

-- Historial Médico
CREATE TABLE medical_records (
    id SERIAL PRIMARY KEY,
    patient_id INTEGER REFERENCES patients (id) ON DELETE CASCADE,
    veterinarian_id INTEGER REFERENCES users (id),
    date TIMESTAMPTZ DEFAULT NOW (),
    diagnosis TEXT NOT NULL,
    treatment TEXT,
    notes TEXT,
    weight_at_visit DECIMAL(5, 2)
);

-- Vacunas/Procedimientos
CREATE TABLE procedures (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    type procedure_type NOT NULL, -- Enum
    description TEXT,
    duration_minutes INTEGER
);

-- Tabla de relación Paciente-Procedimiento
CREATE TABLE patient_procedures (
    id SERIAL PRIMARY KEY,
    patient_id INTEGER REFERENCES patients (id) ON DELETE CASCADE,
    procedure_id INTEGER REFERENCES procedures (id),
    veterinarian_id INTEGER REFERENCES users (id),
    date DATE NOT NULL,
    next_due_date DATE,
    notes TEXT,
    CONSTRAINT chk_next_date CHECK (
        next_due_date IS NULL
        OR next_due_date >= date
    )
);

-- Primero creamos la tabla sin el CHECK que contiene subconsulta
CREATE TABLE appointments (
    id SERIAL PRIMARY KEY,
    patient_id INTEGER REFERENCES patients(id) ON DELETE SET NULL,
    client_id INTEGER REFERENCES clients(id) ON DELETE SET NULL,
    veterinarian_id INTEGER REFERENCES users(id) NOT NULL,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ,
    status appointment_status DEFAULT 'scheduled',
    reason TEXT NOT NULL,
    -- Fechas coherentes
    CONSTRAINT chk_valid_times CHECK (start_time < end_time)
);

-- Función de validación para el rol de veterinario
CREATE OR REPLACE FUNCTION validate_veterinarian_role()
RETURNS TRIGGER AS $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM users
        WHERE id = NEW.veterinarian_id
        AND role = 'veterinarian'
    ) THEN
        RAISE EXCEPTION 'El usuario (ID: %) no es un veterinario válido', NEW.veterinarian_id;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger que ejecuta la validación
CREATE TRIGGER trg_validate_veterinarian_role
BEFORE INSERT OR UPDATE ON appointments
FOR EACH ROW EXECUTE FUNCTION validate_veterinarian_role();

-- Indices
-- Para búsquedas frecuentes
CREATE INDEX idx_patient_client ON patients (client_id);

CREATE INDEX idx_medical_patient ON medical_records (patient_id);

CREATE INDEX idx_appointment_vet ON appointments (veterinarian_id);

CREATE INDEX idx_appointment_status ON appointments (status);

-- Para campos únicos adicionales
CREATE UNIQUE INDEX idx_client_phone ON clients (phone)
WHERE
    phone IS NOT NULL;
