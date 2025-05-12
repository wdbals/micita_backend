-- 1. Insertar usuarios (veterinarios, asistentes y admin)
INSERT INTO
    users (
        email,
        password_hash,
        name,
        role,
        license_number,
        is_active
    )
VALUES
    (
        'dr.smith@vetclinic.com',
        '$2a$10$xJwL5v5zL3h7U3vYQ1qZNu',
        'Dr. John Smith',
        'veterinarian',
        'VET-12345',
        TRUE
    ),
    (
        'dr.jones@vetclinic.com',
        '$2a$10$xJwL5v5zL3h7U3vYQ1qZNu',
        'Dr. Sarah Jones',
        'veterinarian',
        'VET-54321',
        TRUE
    ),
    (
        'assistant1@vetclinic.com',
        '$2a$10$xJwL5v5zL3h7U3vYQ1qZNu',
        'Maria Garcia',
        'assistant',
        NULL,
        TRUE
    ),
    (
        'admin@vetclinic.com',
        '$2a$10$xJwL5v5zL3h7U3vYQ1qZNu',
        'Admin User',
        'admin',
        NULL,
        TRUE
    );

-- 2. Insertar razas de animales
INSERT INTO
    breeds (species, name)
VALUES
    ('dog', 'Labrador Retriever'),
    ('dog', 'German Shepherd'),
    ('dog', 'Beagle'),
    ('cat', 'Siamese'),
    ('cat', 'Persian'),
    ('cat', 'Domestic Shorthair'),
    ('bird', 'Parrot'),
    ('bird', 'Canary'),
    ('rodent', 'Hamster'),
    ('rabbit', 'Dutch Rabbit');

-- 3. Insertar clientes
INSERT INTO
    clients (name, email, phone, address, notes, assigned_to)
VALUES
    (
        'Robert Johnson',
        'robert.johnson@email.com',
        '555-0101',
        '123 Main St, Anytown',
        'Prefiere comunicación por email',
        1
    ),
    (
        'Emily Davis',
        'emily.davis@email.com',
        '555-0102',
        '456 Oak Ave, Somewhere',
        'Tiene 3 mascotas',
        2
    ),
    (
        'Michael Brown',
        'michael.brown@email.com',
        '555-0103',
        '789 Pine Rd, Nowhere',
        NULL,
        1
    ),
    (
        'Jessica Wilson',
        'jessica.wilson@email.com',
        '555-0104',
        '321 Elm St, Anywhere',
        'Alergico a la penicilina',
        2
    );

-- 4. Insertar pacientes (mascotas)
INSERT INTO
    patients (
        name,
        species,
        breed,
        birth_date,
        gender,
        weight_kg,
        client_id
    )
VALUES
    ('Max', 'dog', 1, '2018-05-15', 'male', 28.5, 1),
    (
        'Bella',
        'dog',
        2,
        '2019-11-03',
        'female',
        32.0,
        1
    ),
    ('Luna', 'cat', 5, '2020-02-20', 'female', 4.2, 2),
    (
        'Charlie',
        'dog',
        3,
        '2017-08-10',
        'male',
        12.3,
        3
    ),
    ('Lucy', 'cat', 6, '2021-01-12', 'female', 3.8, 4),
    ('Tweety', 'bird', 7, '2020-07-04', 'male', 0.5, 2);

-- 5. Insertar procedimientos
INSERT INTO
    procedures (name, type, description, duration_minutes)
VALUES
    (
        'Rabies Vaccine',
        'vaccine',
        'Annual rabies vaccination',
        15
    ),
    (
        'Distemper Vaccine',
        'vaccine',
        'Canine distemper vaccine',
        15
    ),
    (
        'Spay/Neuter',
        'surgery',
        'Standard spay/neuter procedure',
        60
    ),
    (
        'Dental Cleaning',
        'surgery',
        'Professional dental cleaning',
        45
    ),
    (
        'Flea Treatment',
        'deworming',
        'Topical flea treatment',
        10
    ),
    ('Blood Test', 'test', 'Complete blood count', 20),
    (
        'Nail Trim',
        'grooming',
        'Basic nail trimming',
        10
    );

-- 6. Insertar historial médico
INSERT INTO
    medical_records (
        patient_id,
        veterinarian_id,
        diagnosis,
        treatment,
        notes,
        weight_at_visit
    )
VALUES
    (
        1,
        1,
        'Healthy annual checkup',
        'Rabies vaccine administered',
        'Patient in good health',
        28.3
    ),
    (
        1,
        2,
        'Skin irritation',
        'Prescribed medicated shampoo',
        'Possible allergy, monitor',
        28.5
    ),
    (
        2,
        1,
        'Vaccination update',
        'Distemper and rabies vaccines',
        NULL,
        31.8
    ),
    (
        3,
        2,
        'Spay surgery',
        'Performed spay surgery',
        'Recovering well',
        4.1
    ),
    (
        4,
        1,
        'Dental issues',
        'Dental cleaning performed',
        'Two teeth extracted',
        12.5
    ),
    (
        6,
        2,
        'Feather plucking',
        'Behavioral consultation',
        'Environmental changes recommended',
        0.5
    );

-- 7. Insertar procedimientos de pacientes
INSERT INTO
    patient_procedures (
        patient_id,
        procedure_id,
        veterinarian_id,
        date,
        next_due_date,
        notes
    )
VALUES
    (
        1,
        1,
        1,
        '2023-01-15',
        '2024-01-15',
        'Annual rabies vaccine'
    ),
    (
        1,
        2,
        1,
        '2023-01-15',
        '2024-01-15',
        'Distemper vaccine'
    ),
    (
        2,
        1,
        2,
        '2023-02-10',
        '2024-02-10',
        'Rabies vaccine'
    ),
    (3, 3, 2, '2023-03-05', NULL, 'Spay surgery'),
    (
        4,
        4,
        1,
        '2023-03-20',
        '2024-03-20',
        'Dental cleaning'
    ),
    (
        5,
        5,
        2,
        '2023-04-12',
        '2023-07-12',
        'Flea treatment'
    );

-- 8. Insertar citas
INSERT INTO
    appointments (
        patient_id,
        client_id,
        veterinarian_id,
        start_time,
        end_time,
        status,
        reason
    )
VALUES
    (
        1,
        1,
        1,
        '2023-06-01 09:00:00+00',
        '2023-06-01 09:30:00+00',
        'completed',
        'Annual checkup'
    ),
    (
        2,
        1,
        2,
        '2023-06-01 10:00:00+00',
        '2023-06-01 10:45:00+00',
        'completed',
        'Vaccination'
    ),
    (
        3,
        2,
        1,
        '2023-06-02 11:00:00+00',
        '2023-06-02 11:30:00+00',
        'scheduled',
        'Follow-up'
    ),
    (
        4,
        3,
        2,
        '2023-06-02 14:00:00+00',
        '2023-06-02 14:30:00+00',
        'scheduled',
        'Dental check'
    ),
    (
        5,
        4,
        1,
        '2023-06-03 15:00:00+00',
        '2023-06-03 15:15:00+00',
        'canceled',
        'Nail trim'
    ),
    (
        6,
        2,
        2,
        '2023-06-03 16:00:00+00',
        '2023-06-03 16:30:00+00',
        'scheduled',
        'Behavioral consult'
    );
