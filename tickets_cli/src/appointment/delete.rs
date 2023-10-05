use std::io::BufRead;
use anyhow::Error;
use tokio::{io, task};
use uuid::Uuid;
use shared::domain::RemoveAppointment;

pub async fn read_uuids() -> io::Result<Vec<Uuid>> {
    println!("Please list id's (Uuid) Appointment to delete (EOL format)':");
    task::spawn_blocking(|| {
        let stdin = std::io::stdin();
        let locked = stdin.lock();
        let mut uuids = Vec::new();

        for line in locked.lines() {
            let line = line?;
            if line.trim().is_empty() {
                break;
            }
            let id = match Uuid::parse_str(&line) {
                Ok(uuid) => uuid,
                Err(_) => return Err(io::Error::new(io::ErrorKind::InvalidData, "Failed to parse UUID")),
            };
            uuids.push(id);
        }

        Ok(uuids)
    })
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
}

pub async fn delete_appointment_handler(uuids: Vec<Uuid>) -> Result<bool, Error> {
    let mut payload = if uuids.is_empty() {
        let uuids = read_uuids().await?;
        RemoveAppointment { id: uuids }
    } else {
        RemoveAppointment { id: uuids }
    };
    payload.id.sort();
    payload.id.dedup();


    println!("{:?}", payload);
    Ok(true)
}