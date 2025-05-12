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
  "role": "Veterinarian", // Opciones: "Veterinarian", "Assistant", "Admin"
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

#### **POST /api/users/login**: Inicia sesión y obtiene un token JWT.
```json
// Request
{
  "email": "nuevo.usuario@ejemplo.com",
  "password": "contraseñaSegura123",
}

//Response
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


### Pacientes

#### PatientResponse  (Estructura de respuesta)

```json
{
  "id": 1,
  "name": "Max",
  "species": "Dog",
  "breed": "Golden Retriever", // Nombre de la raza (opcional)
  "breed_id": 3,              // ID de la raza (opcional)
  "birth_date": "2020-05-15", // Fecha de nacimiento (opcional)
  "gender": "Male",           // Género (opcional)
  "weight_kg": 12.5,          // Peso en kg (opcional)
  "client_id": 1,             // ID del cliente dueño
  "photo_url": "https://example.com/max.jpg " // URL de la foto (opcional)
}
```

#### **GET /api/patients**: Lista pacientes con filtros avanzados.

#### **GET /api/patients/{id}**: Obtiene un paciente por ID.

#### **POST /api/patients**: Crea un nuevo paciente.
```json
{
  "name": "Max",
  "species": "dog", // Opciones: "dog", "cat", "bird", "reptile", "rodent", "rabbit", "other"
  "breed_id": 1, // Opcional, ID de la raza
  "birth_date": "2020-05-15", // Opcional
  "gender": "male", // Opciones: "male", "female", "unknown"
  "weight_kg": 12.5, // Opcional
  "client_id": 1, // ID del dueño
  "photo_url": "https://example.com/max.jpg" // Opcional
}
```
#### **PUT /api/patients/{id}**: Actualiza un paciente existente.

#### **DELETE /api/patients/{id}**: Elimina un paciente.


### Citas

#### **GET /api/appointments**: Lista citas con filtros avanzados.

#### **GET /api/appointments/{id}**: Obtiene una cita por ID.

#### **POST /api/appointments**: Crea una nueva cita.
```json
{
  "patient_id": 1, // Opcional
  "client_id": 2, // Opcional
  "veterinarian_id": 3,
  "start_time": "2023-11-01T10:00:00Z",
  "end_time": "2023-11-01T11:00:00Z",
  "reason": "Consulta de rutina"
}
```

#### **PUT /api/appointments/{id}**: Actualiza una cita existente.

#### **DELETE /api/appointments/{id}**: Elimina una cita.


### Razas

#### **GET /api/breeds**: Lista las razas.

#### **POST /api/breeds**
```json
{
  "species": "dog", // Opciones: "dog", "cat", "bird", "reptile", "rodent", "rabbit", "other"
  "name": "Labrador Retriever"
}
```


### Procedimientos

#### **GET /api/procedures**: Lista procedimientos.

#### **POST /api/procedures**: Crea un nuevo procedimiento.
```json
{
  "name": "Vacuna contra la rabia",
  "procedure_type": "vaccine", // Opciones: "vaccine", "surgery", "deworming", "test", "grooming", "other"
  "description": "Vacuna anual contra la rabia", // Opcional
  "duration_minutes": 15 // Opcional
}
```

#### **PUT /api/procedures/{id}**: Actualiza un procedimiento existente.

#### **DELETE /api/procedures/{id}**: Elimina un procedimiento.


## Registros médicos

#### **GET /api/medical_records**: Lista registros médicos con filtros avanzados.

#### **GET /api/medical_records/{id}**: Obtiene un registro médico por ID.

#### **POST /api/medical_records**: Crea un nuevo registro médico.
```json
{
  "patient_id": 1,
  "veterinarian_id": 3,
  "diagnosis": "Infección en la oreja",
  "treatment": "Antibióticos", // Opcional
  "notes": "Seguimiento en una semana", // Opcional
  "weight_at_visit": 12.5 // Opcional
}
```

#### **PUT /api/medical_records/{id}**: Actualiza un registro médico existente.

#### **DELETE /api/medical_records/{id}**: Elimina un registro médico.


### Estadísticas

#### **GET /api/stats**: Obtiene estadísticas generales del sistema.
