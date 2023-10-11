# Event Appointments Service

This service is composed of three main parts:

- Web Server: Manages the main interface and API endpoints.
- Ticket CLI: Provides command-line functionalities for managing appointments.
- Shared Module: Contains utilities and functions that are commonly used across the Web Server and Ticket CLI.
---

# Web Server - Appointments & Invitations

Web server - service that managing appointments and invitations! This server provides a set of endpoints to manage appointments, generate QR invitations, and validate them.

## Prerequisites

- Rust and Cargo installed
- PostgreSQL Database
- Docker installed
- Proper configuration setup for email and QR services

## Setup & Installation

1. Clone the repository:

```bash
git clone [REPOSITORY_URL]
cd web-server
```

2. Setup the PostgreSQL database:
```bash
docker-compose up 
```

3. Update the configuration:
```bash
export APP_ENVIRONMENT = development | production | local
```

4. Build & Run:
```bash
cargo build && cargo run
```

## Configuration

### Email service (Based on Postmark service)

    email_client:
        base_url:               "https://api.postmarkapp.com"   
        sender_email:           "your@email.com"
        authorization_token:    "{POSTMARK_API_KEY}"
        timeout_milliseconds:   10000

### QR service (Based on Apilayer service)

    qr_client:
        api_url:                "https://api.apilayer.com/short_url/hash"
        api_key:                "{APILAYER_API_KEY}"
        base_url:               "{WEB_SERVER_BASE_URL}/api/validations"
        base_image_path:        "{PATH_TO_QR_CODES_STORE}"
        timeout_milliseconds:   10000

### Console client

    console_cli:
        web_url: "http://127.0.0.1:8000"    Url responsible for http communication between server and client

### Env variables

    Don't forget that all configuration can be overwritten via ENV variables:
    - prefix APP
    - prefix_separator "_"
    - separator("__")
    
    For example

    APP_CONSOLE_CLI__WEB_URL = "new domain"

## Usage

Once the server is up and running, you can use various endpoints to manage appointments, generate QR invitations, and validate them.

## Endpoints

### Appointments
    Create:             POST /api/appointment       
    Get by ID:          GET /api/appointment/{id}
    Delete by ID:       DELETE /api/appointment/{id}             
    Add Invitation:     POST /api/appointment/{id}/invitation`

### Invitations
    Get by ID:          GET /api/invitation/{id}
    Get QR Code:        GET /api/invitation/{id}/qr

### Validation
    Validate by ID      GET /api/validations/{id}

---

# Tickets CLI

`tickets_cli` is a command-line utility designed for managing appointments. It provides a range of functionalities including appointment creation, deletion, generation of QR invitations, and sending out email invitations. With a focus on appointments, it's the go-to solution for scheduling and organization.

## Features:

- **User Authentication**: A command specifically dedicated to handle user authentication.
- **Appointment Management**: Execute a range of appointment related tasks directly from your terminal.
    - Create new appointments
    - Delete existing appointments
    - Generate QR codes for specific appointments
    - Send email invitations containing QR codes

## Usage:

```bash
tickets_cli [OPTIONS] <SUBCOMMAND>
```

### Options:

- `--auth-token` : Optional authentication token to authorize certain operations.

### Commands:

- `auth`: Handles user authentication.
- `appointment`: Manages appointment interactions.

  #### Appointment Subcommands:

    - `create`: Creates a new appointment.
    - `delete`: Deletes a specified appointment. Provide the IDs of the appointments to delete using the `--uuids` flag.
    - `generate`: Generates QR codes for a specified appointment. Use the `--appt_id` flag to specify the appointment and `--count` to indicate the number of QR codes.
    - `send`: Sends emails containing QR codes for a specific appointment. Use the `--appt_id` flag to specify the appointment and `--email` to list the email addresses.
    
## Getting Started:

1. Create new `Appointment` command for appointment-related operations:

```bash
tickets_cli appointment Create
```

2. Generate new `Invitation` command for appointment-related operations:
```bash
tickets_cli appointment generate --appt_id xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx 
tickets_cli appointment generate --appt_id xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx --count N
```
As a result, you'll get path's to genetated QR images

3. To generate and send new `Invitation` by emails you can use:
```bash
tickets_cli appointment send --appt_id xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx 
```

4. To remove `Appointment` you can use:
```bash
tickets_cli appointment delete
or
tickets_cli appointment delete --uuids xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx --uuids xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx 
```

## Conclusion:

`tickets_cli` aims to simplify the process of appointment management. With its intuitive CLI commands, managing appointments has never been easier!

---