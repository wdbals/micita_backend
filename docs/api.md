# Documentación de API

## Enums del Sistema

> **Nota:** Todos los valores de enum deben ser enviados/recibidos comenzando con mayúscula (ej: `Veterinarian`, `Scheduled`, `Dog`).

### <a id="user_role">UserRole</a>
**Tipo en BD:** `user_role`

**Valores disponibles:**
- `Veterinarian`
- `Assistant`
- `Admin`

### <a id="animal_species">AnimalSpecies</a>
**Tipo en BD:** `animal_species`

**Valores disponibles:**
- `Dog`
- `Cat`
- `Bird`
- `Reptile`
- `Rodent`
- `Rabbit`
- `Other`

### <a id="animal_gender">AnimalGender</a>
**Tipo en BD:** `animal_gender`

**Valores disponibles:**
- `Male`
- `Female`
- `Unknown`

### <a id="appointment_status">AppointmentStatus</a>
**Tipo en BD:** `appointment_status`

**Valores disponibles:**
- `Scheduled`
- `Completed`
- `Canceled`
- `NoShow`

### <a id="procedure_type">ProcedureType</a>
**Tipo en BD:** `procedure_type`

**Valores disponibles:**
- `Vaccine`
- `Surgery`
- `Deworming`
- `Test`
- `Grooming`
- `Other`

## Endpoints


### Usuarios

#### UserResponse (Estructura de respuesta)
```json
{
  "id": 123,
  "email": "usuario@ejemplo.com",
  "name": "Nombre Usuario",
  "role": "Veterinarian",
  "license_number": "LIC-12345",
  "is_active": true,
  "created_at": "2023-01-15T10:30:00Z"
}
```

#### **GET /api/users**: Lista usuarios con filtros avanzados.

##### Parametros opcionales
| Parámetro        | Tipo      | Descripción                                                                 | Ejemplo                          |
|------------------|-----------|-----------------------------------------------------------------------------|----------------------------------|
| patient_id       | número    | Filtra por ID de mascota                                                    | patient_id=123                   |
| client_id        | número    | Filtra por ID del dueño                                                     | client_id=456                    |
| veterinarian_id  | número    | Filtra por ID del veterinario                                               | veterinarian_id=789              |
| status           | string    | Estado de la cita ( [AppointmentStatus](#appointment_status) ) | status=Completed |
| start_date       | ISO 8601  | Fecha inicial (inclusive) en UTC                                            | start_date=2023-11-20T08:00:00Z |
| end_date         | ISO 8601  | Fecha final (inclusive) en UTC                                              | end_date=2023-11-25T17:00:00Z   |
| reason_contains  | string    | Busca texto en el campo "reason" (case-insensitive)                         | reason_contains=emergency        |
| limit            | número    | Cantidad máxima de resultados (para paginación)                             | limit=10                         |
| offset           | número    | Número de resultados a saltar (para paginación)                             | offset=20                        |

> **Nota:** Los valores para `status` corresponden al enum [`AppointmentStatus`](#appointment_status) y deben enviarse comenzando con mayúscula.

#### **GET /api/users/{id}**: Obtiene un usuario por ID.

#### **POST /api/users**: Crea un nuevo usuario.
```json
{
  "email": "nuevo.usuario@ejemplo.com",
  "password": "contraseñaSegura123",
  "name": "Nuevo Usuario",
  "role": "Veterinarian",
  "license_number": "VET-12345" // Opcional, solo para veterinarios
}
```
#### **PUT /api/users/{id}**: Actualiza un usuario existente.

```json
{
  "email": "nuevo_email@ejemplo.com",
  "password": "nuevaContraseña123",
  "name": "Nuevo Nombre",
  "role": "Admin",
  "license_number": "LIC-54321",
  "is_active": false
}
```

#### **DELETE /api/users/{id}**: Elimina un usuario (borrado lógico).

```http
HTTP/1.1 204 No Content
```

#### **POST /api/users/login**: Inicia sesión y obtiene un token JWT.

##### Solicitud

```json
{
  "email": "nuevo.usuario@ejemplo.com",
  "password": "contraseñaSegura123",
}
```

##### Respuesta

```json
{
  "token": "jwt.token.here",
  "user": {
    "id": 123,
    "email": "usuario@ejemplo.com",
    // ... resto de campos de UserResponse
  }
}
```


### Clientes

#### ClientResponse  (Estructura de respuesta)
```json
{
  "id": 1,
  "name": "Juan Pérez",
  "email": "juan.perez@example.com",
  "phone": "+56912345678",
  "assigned_to": 12
}
```

#### **GET /api/clients**: Lista clientes con filtros avanzados.

##### Parametros

| Parámetro    | Tipo    | Descripción                                                                 | Ejemplo               |
|--------------|---------|-----------------------------------------------------------------------------|-----------------------|
| name         | string  | Filtra por nombre del cliente (búsqueda case-insensitive)                   | `name=Juan`           |
| phone        | string  | Filtra por número de teléfono (coincidencia exacta)                        | `phone=+56912345678`  |
| assigned_to  | número  | Filtra por ID del usuario asignado (veterinario/asistente)                 | `assigned_to=12`      |
| limit        | número  | Cantidad máxima de resultados (para paginación, default: 50)               | `limit=10`            |
| offset       | número  | Número de resultados a saltar (para paginación, default: 0)                | `offset=20`           |

#### **GET /api/clients/{id}**: Obtiene un cliente por ID.

#### **POST /api/clients**: Crea un nuevo cliente.
```json
{
  "name": "María López",
  "email": "maria.lopez@example.com", // Opcional
  "phone": "+56987654321",
  "address": "Calle Falsa 123",       // Opcional
  "notes": "Cliente frecuente",       // Opcional
  "assigned_to": 15
}
```
#### **PUT /api/clients/{id}**: Actualiza un cliente existente.

```json
{
  "name": "María López Gómez", // Opcional
  "email": "maria.gomez@example.com", // Opcional
  "phone": "+56987654321",     // Opcional
  "address": null,             // Opcional.
  "notes": "Cliente VIP",      // Opcional.
  "assigned_to": null          // Opcional.
}
```
> `null` no actualiza el valor actual

#### **DELETE /api/clients/{id}**: Elimina un cliente.

```http
HTTP/1.1 204 No Content
```

### Pacientes

#### PatientResponse  (Estructura de respuesta)

```json
{
  "id": 1,
  "name": "Max",
  "species": "Dog",
  "breed": "Golden Retriever",
  "breed_id": 3,
  "birth_date": "2020-05-15", // Fecha de nacimiento (opcional)
  "gender": "Male",           // Género (opcional)
  "weight_kg": 12.5,          // Peso en kg (opcional)
  "client_id": 1,             // ID del cliente dueño
  "photo_url": "https://example.com/max.jpg " // URL de la foto (opcional)
}
```

#### **GET /api/patients**: Lista pacientes con filtros avanzados.

##### Parametros

| Parámetro  | Tipo    | Descripción                                                                 | Ejemplo          |
|------------|---------|-----------------------------------------------------------------------------|------------------|
| name       | string  | Filtra por nombre (búsqueda parcial insensible a mayúsculas/minúsculas)    | `name=Max`       |
| species    | string  | Filtra por especie ([AnimalSpecies](#animal_species))                                    | `species=Dog`    |
| breed_id   | número  | Filtra por ID de raza                                                      | `breed_id=3`     |
| client_id  | número  | Filtra por ID del cliente dueño                                            | `client_id=1`    |
| gender     | string  | Filtra por género ([AnimalGender](#animal_gender))                                 | `gender=Male`    |
| limit      | número  | Máximo de resultados (default: 50, máximo: 400)                            | `limit=20`       |
| offset     | número  | Desplazamiento (default: 0)                                                | `offset=10`      |

##### Respuesta

```json
[
  {
    "id": 1,
    "name": "Max",
    "species": "Dog",
    "breed": "Golden Retriever",
    "breed_id": 3,
    "birth_date": "2020-05-15",
    "gender": "Male",
    "weight_kg": 12.5,
    "client_id": 1,
    "photo_url": "https://example.com/max.jpg "
  },
  {
    "id": 2,
    "name": "Bella",
    "species": "Cat",
    "breed": "Siamese",
    "breed_id": 5,
    "birth_date": "2018-08-20",
    "gender": "Female",
    "weight_kg": 4.2,
    "client_id": 2,
    "photo_url": null
  }
]
```

#### **GET /api/patients/{id}**: Obtiene un paciente por ID.

##### Respuesta

```json
{
  "id": 1,
  "name": "Max",
  "species": "Dog",
  "breed": "Golden Retriever",
  "breed_id": 3,
  "birth_date": "2020-05-15",
  "gender": "Male",
  "weight_kg": 12.5,
  "client_id": 1,
  "photo_url": "https://example.com/max.jpg "
}
```

#### **POST /api/patients**: Crea un nuevo paciente.

##### Solicitud

```json
{
  "name": "Max",
  "species": "dog",
  "breed_id": 1, // ID de la raza
  "birth_date": "2020-05-15",
  "gender": "male",
  "weight_kg": 12.5, // Opcional
  "client_id": 1, // ID del dueño
  "photo_url": "https://example.com/max.jpg" // Opcional
}
```

##### Respuesta

```json
{
  "id": 1,
  "name": "Max",
  "species": "Dog",
  "breed": "Golden Retriever",
  "breed_id": 3,
  "birth_date": "2020-05-15",
  "gender": "Male",
  "weight_kg": 12.5,
  "client_id": 1,
  "photo_url": "https://example.com/max.jpg "
}
```

#### **PUT /api/patients/{id}**: Actualiza un paciente existente.

##### Solicitud

```json
{
  "name": "Max Updated", // Opcional
  "species": "Dog", // Opcional
  "breed_id": null, // Opcional.
  "birth_date": null, // Opcional.
  "gender": null, // Opcional.
  "weight_kg": 13.0, // Opcional
  "client_id": 2, // Opcional
  "photo_url": null // Opcional.
}
```

> Si se envía un null, se omite el campo correspondiente.

##### Respuesta

```json
{
  "id": 1,
  "name": "Max Updated",
  "species": "Dog",
  "breed": null,
  "breed_id": null,
  "birth_date": null,
  "gender": null,
  "weight_kg": 13.0,
  "client_id": 2,
  "photo_url": null
}
```

#### **DELETE /api/patients/{id}**: Elimina un paciente.

```http
HTTP/1.1 204 No Content
```

### Citas

#### AppointmentResponse  (Estructura de respuesta)

```json
{
  "id": 1,
  "patient_id": 5,
  "patient_name": "Max",
  "client_id": 2,
  "client_name": "Juan Pérez",
  "veterinarian_id": 3,
  "veterinarian_name": "Dr. López",
  "start_time": "2023-11-01T10:00:00Z",
  "end_time": "2023-11-01T11:00:00Z",
  "status": "Scheduled",
  "reason": "Consulta de rutina",
  "duration_minutes": 60
}
```

#### **GET /api/appointments**: Lista citas con filtros avanzados.

##### Parametros

| Parámetro        | Tipo            | Descripción                                                                 | Ejemplo                          |
|------------------|-----------------|-----------------------------------------------------------------------------|----------------------------------|
| `patient_id`     | número          | Filtrar por ID del paciente                                                 | `patient_id=5`                   |
| `client_id`      | número          | Filtrar por ID del cliente                                                  | `client_id=2`                    |
| `veterinarian_id`| número          | Filtrar por ID del veterinario                                              | `veterinarian_id=3`              |
| `status`         | string          | Filtrar por estado ([`AppointmentStatus`](#appointment_status))                         | `status=Scheduled`               |
| `start_date`     | fecha/hora ISO  | Citas que comienzan después de esta fecha/hora (inclusive)                  | `start_date=2023-11-01T00:00:00Z`|
| `end_date`       | fecha/hora ISO  | Citas que terminan antes de esta fecha/hora (inclusive)                     | `end_date=2023-11-30T23:59:59Z`  |
| `reason_contains`| string          | Filtrar por citas cuya razón contenga este texto (case-insensitive)         | `reason_contains=rutina`         |
| `limit`          | número          | Máximo de resultados (default: 50, máximo permitido: 400)                   | `limit=20`                       |
| `offset`         | número          | Desplazamiento para paginación (default: 0)                                 | `offset=10`                      |

**Notas importantes:**
- Formato de fechas: **ISO 8601** (UTC)
- Valores válidos para [`AppointmentStatus`](#appointment_status)
- Para búsquedas de texto (`reason_contains`), se ignoran mayúsculas/minúsculas

##### Respuesta

```json
[
  {
    "id": 1,
    "patient_id": 5,
    "patient_name": "Max",
    "client_id": 2,
    "client_name": "Juan Pérez",
    "veterinarian_id": 3,
    "veterinarian_name": "Dr. López",
    "start_time": "2023-11-01T10:00:00Z",
    "end_time": "2023-11-01T11:00:00Z",
    "status": "Scheduled",
    "reason": "Consulta de rutina",
    "duration_minutes": 60
  },
  {
    "id": 2,
    "patient_id": 7,
    "patient_name": "Bella",
    "client_id": 4,
    "client_name": "María Gómez",
    "veterinarian_id": 3,
    "veterinarian_name": "Dr. López",
    "start_time": "2023-11-02T14:00:00Z",
    "end_time": "2023-11-02T15:00:00Z",
    "status": "Completed",
    "reason": "Vacunación anual",
    "duration_minutes": 60
  }
]
```

#### **GET /api/appointments/{id}**: Obtiene una cita por ID.

##### Respuesta

```json
{
  "id": 1,
  "patient_id": 5,
  "patient_name": "Max",
  "client_id": 2,
  "client_name": "Juan Pérez",
  "veterinarian_id": 3,
  "veterinarian_name": "Dr. López",
  "start_time": "2023-11-01T10:00:00Z",
  "end_time": "2023-11-01T11:00:00Z",
  "status": "Scheduled",
  "reason": "Consulta de rutina",
  "duration_minutes": 60
}
```

#### **POST /api/appointments**: Crea una nueva cita.

##### Solicitud

```json
{
  "patient_id": 5,
  "client_id": 2,
  "veterinarian_id": 3,
  "start_time": "2023-11-01T10:00:00Z",
  "end_time": "2023-11-01T11:00:00Z",
  "reason": "Consulta de rutina"
}
```

##### Respuesta

```json
{
  "id": 1,
  "patient_id": 5,
  "patient_name": "Max",
  "client_id": 2,
  "client_name": "Juan Pérez",
  "veterinarian_id": 3,
  "veterinarian_name": "Dr. López",
  "start_time": "2023-11-01T10:00:00Z",
  "end_time": "2023-11-01T11:00:00Z",
  "status": "Scheduled",
  "reason": "Consulta de rutina",
  "duration_minutes": 60
}
```

#### **PUT /api/appointments/{id}**: Actualiza una cita existente.

##### Solicitud

```json
{
  "patient_id": 1,
  "client_id": 2,
  "veterinarian_id": 5,
  "start_time": "2023-11-02T14:00:00Z",
  "end_time": "2023-11-02T15:00:00Z",
  "status": "Completed",
  "reason": "Consulta de seguimiento"
}
```

##### Respuesta

```json
{
  "id": 1,
  "patient_id": 5,
  "patient_name": "Max",
  "client_id": 2,
  "client_name": "Juan Pérez",
  "veterinarian_id": 3,
  "veterinarian_name": "Dr. López",
  "start_time": "2023-11-01T14:00:00Z",
  "end_time": "2023-11-01T15:00:00Z",
  "status": "Scheduled",
  "reason": "Consulta de seguimiento",
  "duration_minutes": 60
}
```

#### **DELETE /api/appointments/{id}**: Elimina una cita.

```http
HTTP/1.1 204 No Content
```

### Razas

#### BreedResponse  (Estructura de respuesta)

```json
{
  "id": 1,
  "species": "Dog",
  "name": "Labrador Retriever"
}
```

#### **GET /api/breeds**: Lista las razas.

##### Parametros

| Parámetro | Tipo   | Descripción                              | Valores por defecto | Ejemplo   |
|-----------|--------|------------------------------------------|---------------------|-----------|
| `limit`   | número | Límite de resultados (máximo permitido: 400) | 50                  | `limit=10` |
| `offset`  | número | Desplazamiento para paginación           | 0                   | `offset=20` |

##### Respuesta

```json
[
  {
    "id": 1,
    "species": "Dog",
    "name": "Labrador Retriever"
  },
  {
    "id": 2,
    "species": "Cat",
    "name": "Siamese"
  }
]
```

#### **GET /api/breeds/{id}**: Obtiene una raza por ID.

##### Respuesta

```json
{
  "id": 1,
  "species": "Dog",
  "name": "Labrador Retriever"
}
```

#### **POST /api/breeds**: : Crea una nueva raza.

##### Solicitud

```json
{
  "species": "dog",
  "name": "Labrador Retriever"
}
```
> Los valores para `species` corresponden al enum [AnimalSpecies](#animal_species) y deben enviarse comenzando con mayúscula.

#### **PUT /api/breeds/{id}**: Actualiza una raza existente.

##### Solicitud

```json
{
  "species": "Dog",
  "name": "Retriever Dorado"
}
```

##### Respuesta

```json
{
  "id": 3,
  "species": "Dog",
  "name": "Golden Retriever"
}
```

#### **DELETE /api/breeds/{id}**: Elimina una raza existente.
> Nota:  No se puede eliminar una raza si tiene mascotas registradas asociadas.

##### Respuesta
```http
HTTP/1.1 204 No Content
```

### Procedimientos

#### **GET /api/procedures**: Lista procedimientos.

#### **POST /api/procedures**: Crea un nuevo procedimiento.
```json
{
  "name": "Vacuna contra la rabia",
  "procedure_type": "vaccine",
  "description": "Vacuna anual contra la rabia", // Opcional
  "duration_minutes": 15
}
```

> Los valores para `procedure_type` corresponden al enum [ProcedureType](#procedure_type) y deben enviarse comenzando con mayúscula.


#### **PUT /api/procedures/{id}**: Actualiza un procedimiento existente.

#### **DELETE /api/procedures/{id}**: Elimina un procedimiento.

```http
HTTP/1.1 204 No Content
```

### Paciente-Procedimiento



### Registros médicos

#### **GET /api/medical_records**: Lista registros médicos con filtros avanzados.

#### **GET /api/medical_records/{id}**: Obtiene un registro médico por ID.

#### **POST /api/medical_records**: Crea un nuevo registro médico.
```json
{
  "patient_id": 1,
  "veterinarian_id": 3,
  "diagnosis": "Infección en la oreja",
  "treatment": "Antibióticos",
  "notes": "Seguimiento en una semana",
  "weight_at_visit": 12.5
}
```

#### **PUT /api/medical_records/{id}**: Actualiza un registro médico existente.

#### **DELETE /api/medical_records/{id}**: Elimina un registro médico.

```http
HTTP/1.1 204 No Content
```


### Estadísticas

#### **GET /api/stats**: Obtiene estadísticas generales del sistema.

##### Parametros

| Parámetro   | Tipo            | Descripción                                                                 | Ejemplo                     |
|-------------|-----------------|-----------------------------------------------------------------------------|-----------------------------|
| role        | string          | Rol del usuario (`admin` o `veterinarian`)                                  | `role=admin`                |
| user_id     | número          | ID del veterinario (solo relevante si `role=veterinarian`)                  | `user_id=123`               |
| start_date  | fecha (ISO 8601)| Fecha inicial para filtrar datos                                            | `start_date=2023-01-01`     |
| end_date    | fecha (ISO 8601)| Fecha final para filtrar datos                                              | `end_date=2023-12-31`       |
| type_       | string          | Tipo de estadística a obtener (`appointments`, `users`, `procedures`, etc.) | `type_=appointments`        |

```json
// Respuesta
{
  "appointments_by_month": [
    {
      "month": "2023-01",
      "count": 45
    },
    {
      "month": "2023-02",
      "count": 50
    }
  ],
  "user_counts": {
    "total_users": 100,
    "veterinarians": 30,
    "assistants": 50,
    "admins": 20
  },
  "procedures_by_type": [
    {
      "procedure_type": "Vacunación",
      "count": 120
    },
    {
      "procedure_type": "Cirugía",
      "count": 30
    }
  ],
  "patients_by_species": [
    {
      "species": "Dog",
      "count": 75
    },
    {
      "species": "Cat",
      "count": 45
    }
  ],
  "veterinarian_stats": {
    "appointments_by_status": [
      {
        "status": "Completed",
        "count": 30
      },
      {
        "status": "Scheduled",
        "count": 15
      }
    ],
    "procedures_performed": [
      {
        "procedure_type": "Vacunación",
        "count": 50
      },
      {
        "procedure_type": "Cirugía",
        "count": 10
      }
    ],
    "medical_records_created": 40,
    "patients_attended": [
      {
        "species": "Dog",
        "count": 30
      },
      {
        "species": "Cat",
        "count": 10
      }
    ]
  }
}
```
