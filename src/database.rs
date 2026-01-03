use std::fs;

use rusqlite::{named_params, Connection, Error};
use ulid::Ulid;

use crate::{Status, Task};

pub fn get_db_path() -> String {
    let mut data_dir = dirs::data_dir().expect("Could not find data directory in OS");

    data_dir.push("daily-dose");

    fs::create_dir_all(&data_dir).expect("Failed to create directory");

    data_dir.push("storage.db");

    let db_path = data_dir.to_str().expect("Path conversion failed to str");

    db_path.to_string()
}

pub fn open_db_connection() -> Result<Connection, Error> {
    let path = get_db_path();
    let connection = Connection::open(path)?;
    Ok(connection)
}

pub fn create_task_table(conn: &Connection) -> Result<(), Error> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks (
            id   TEXT PRIMARY KEY,
            description TEXT NOT NULL,
            status TEXT NOT NULL,
            date TEXT DEFAULT CURRENT_TIMESTAMP
        )",
        (), // empty list of parameters.
    )?;

    Ok(())
}

pub fn insert_task(
    db_conn: &Connection,
    desc: &str,
    status: Status,
    timestamp: &str,
) -> Result<String, Error> {
    let uid = Ulid::new();

    let doc_id = uid.to_string();

    db_conn.execute(
        "INSERT INTO tasks (id, description, status, date) VALUES (?1, ?2, ?3, ?4)",
        (&doc_id, desc, status, timestamp),
    )?;

    Ok(doc_id)
}

pub fn get_tasks_by_date(
    db_conn: &Connection,
    start_date: &str,
    end_date: Option<&str>,
) -> Result<Vec<Task>, Error> {
    // https://docs.rs/rusqlite/latest/rusqlite/struct.Statement.html#use-with-positional-parameters-1
    let (query, params) = match end_date {
        Some(end_date) => {
            ("SELECT id, description, status, date FROM tasks WHERE date BETWEEN :start_date AND :end_date ORDER BY id", named_params! {
                ":start_date": start_date,
                ":end_date": end_date.to_string(),
            })
        },
        None => ("SELECT id, description, status, date FROM tasks WHERE date = :start_date ORDER BY id", named_params! {
                ":start_date": start_date,
        }),
    };

    let mut stmt = db_conn.prepare(query)?;

    let rows = stmt.query_map(params, |row| {
        Ok(Task {
            id: row.get(0)?,
            description: row.get(1)?,
            status: row.get(2)?,
            date: row.get(3)?,
        })
    })?;

    let mut tasks: Vec<Task> = vec![];

    for task in rows.flatten() {
        tasks.push(task);
    }

    Ok(tasks)
}

pub fn update_task_description(
    db_conn: &Connection,
    task_id: &str,
    desc: &str,
) -> Result<(), Error> {
    db_conn.execute(
        "UPDATE tasks SET description = :description WHERE id = :id",
        named_params! {
            ":description": desc,
            ":id":task_id
        },
    )?;

    Ok(())
}
pub fn update_task_status(
    db_conn: &Connection,
    task_id: &str,
    status: Status,
) -> Result<(), Error> {
    db_conn.execute(
        "UPDATE tasks SET status = :status WHERE id = :id",
        named_params! {
            ":status": status,
            ":id":task_id
        },
    )?;

    Ok(())
}

pub fn delete_task(db_conn: &Connection, task_id: &str) -> Result<(), Error> {
    db_conn.execute(
        "delete from tasks where id = :id",
        named_params! {
            ":id":task_id
        },
    )?;

    Ok(())
}
