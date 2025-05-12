# MiCita Backend

Proyecto universitario, un backend desarrollado en **Rust** utilizando el framework **Actix-Web**. Está diseñado para gestionar las operaciones de una clínica veterinaria, incluyendo la gestión de usuarios, clientes, pacientes, citas, procedimientos médicos y registros médicos.

## Propósito

El propósito de este proyecto es proporcionar una API robusta y segura para gestionar las operaciones de una clínica veterinaria. Esto incluye:

- Gestión de usuarios (veterinarios, asistentes y administradores).
- Gestión de clientes (dueños de mascotas).
- Gestión de pacientes (mascotas).
- Gestión de citas.
- Gestión de procedimientos médicos.
- Gestión de registros médicos.
- Estadísticas y reportes.

## Características principales

- **Autenticación y autorización**: Utiliza JWT para autenticar usuarios y proteger los endpoints.
- **Validaciones robustas**: Validaciones en los modelos y en la base de datos para garantizar la integridad de los datos.
- **Filtros avanzados y paginación**: Soporte para búsquedas avanzadas y paginación en la mayoría de los endpoints.
- **Seguridad**: Protección de la API mediante una clave API y manejo seguro de contraseñas con Argon2.
- **Base de datos**: Integración con PostgreSQL utilizando SQLx.

## Requisitos previos

- **Rust**: Asegúrate de tener instalado Rust en tu sistema. Puedes instalarlo desde [aquí](https://www.rust-lang.org/tools/install).
- **PostgreSQL**: Una base de datos PostgreSQL configurada y accesible.
- **Cargo**: Administrador de paquetes de Rust.

## Configuración

1. Clona este repositorio:
```bash
git clone git@github.com:wdbals/micita_backend.git
cd veterinaria_backend
```

2. Configura las variables de entorno en un archivo `.env`:
```bash
DATABASE_URL=postgres://usuario:contraseña@localhost:5432/nombre_db
API_KEY=api_key_fuerte
ALLOWED_ORIGIN=localhost
JWT_SECRET=token_magico
```

3.Crea la base de datos y ejecuta los scripts de inicialización en `init.sql`:
```bash
psql -U usuario -d nombre_db -f init.sql/0.sql
psql -U usuario -d nombre_db -f init.sql/1.sql
```

4. Instala las dependencias del proyecto:
```bash
cargo build
```

5. Inicia el servidor:
```bash
cargo run
```

> El servidor estará disponible en `http://localhost:8080`.


## Endpoints

### Usuarios

- **GET /api/users**: Lista usuarios con filtros avanzados.
- **GET /api/users/{id}**: Obtiene un usuario por ID.
- **POST /api/users**: Crea un nuevo usuario.
- **PUT /api/users/{id}**: Actualiza un usuario existente.
- **DELETE /api/users/{id}**: Elimina un usuario (borrado lógico).
- **POST /api/users/login**: Inicia sesión y obtiene un token JWT.

### Clientes

- **GET /api/clients**: Lista clientes con filtros avanzados.
- **GET /api/clients/{id}**: Obtiene un cliente por ID.
- **POST /api/clients**: Crea un nuevo cliente.
- **PUT /api/clients/{id}**: Actualiza un cliente existente.
- **DELETE /api/clients/{id}**: Elimina un cliente.

### Pacientes

- **GET /api/patients**: Lista pacientes con filtros avanzados.
- **GET /api/patients/{id}**: Obtiene un paciente por ID.
- **POST /api/patients**: Crea un nuevo paciente.
- **PUT /api/patients/{id}**: Actualiza un paciente existente.
- **DELETE /api/patients/{id}**: Elimina un paciente.

### Citas

- **GET /api/appointments**: Lista citas con filtros avanzados.
- **GET /api/appointments/{id}**: Obtiene una cita por ID.
- **POST /api/appointments**: Crea una nueva cita.
- **PUT /api/appointments/{id}**: Actualiza una cita existente.
- **DELETE /api/appointments/{id}**: Elimina una cita.

### Procedimientos

- **GET /api/procedures**: Lista procedimientos.
- **POST /api/procedures**: Crea un nuevo procedimiento.
- **PUT /api/procedures/{id}**: Actualiza un procedimiento existente.
- **DELETE /api/procedures/{id}**: Elimina un procedimiento.

### Registros médicos

- **GET /api/medical_records**: Lista registros médicos con filtros avanzados.
- **GET /api/medical_records/{id}**: Obtiene un registro médico por ID.
- **POST /api/medical_records**: Crea un nuevo registro médico.
- **PUT /api/medical_records/{id}**: Actualiza un registro médico existente.
- **DELETE /api/medical_records/{id}**: Elimina un registro médico.

### Estadísticas

- **GET /api/stats**: Obtiene estadísticas generales del sistema.

## Estructura del proyecto

- **src/auth.rs**: Funciones relacionadas con autenticación y manejo de JWT.
- **src/db.rs**: Conexión a la base de datos.
- **src/errors.rs**: Manejo de errores personalizados.
- **src/handlers/**: Controladores para cada recurso (usuarios, clientes, pacientes, etc.).
- **src/models/**: Modelos de datos y validaciones.
- **src/middleware.rs**: Middleware para validación de API Key.
- **src/routes.rs**: Configuración de rutas.
