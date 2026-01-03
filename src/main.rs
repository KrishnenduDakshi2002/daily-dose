use std::{collections::HashMap, str::FromStr};

use chrono::{Datelike, Local};
use rusqlite::{
    named_params,
    types::{FromSql, ToSqlOutput},
    Error, ToSql,
};
use strum::{Display, EnumString};

use crate::{
    cmd_handler::{
        construct_cmd_args, handle_cmd_add, handle_cmd_delete, handle_cmd_list, handle_cmd_mark,
        handle_cmd_show, handle_cmd_unmark, handle_cmd_update,
    },
    database::{create_task_table, get_tasks_by_date, insert_task, open_db_connection},
    utils::{construct_timestamp, iso_format_timestamp, render_tasks_table},
};

mod cmd_handler;
mod database;
mod utils;

#[derive(Display, EnumString, Debug)]
#[strum(serialize_all = "snake_case")]
enum Status {
    Todo,
    InProgress,
    Done,
    Blocked,
}

#[derive(Debug)]
struct Task {
    id: String,
    description: String,
    status: Status,
    date: String,
}

impl ToSql for Status {
    /*
     * i received 'trait bound not satisfied error'
     * then in the rusqlite documentation searched for the asked trait
     * found ToSql trait and it's various implementations
     * checked the 'impl ToSql for String' as it look closed to the above enum
     * then visited the source to check how they have implemented that, so i can do the same for
     * this enum
     *
     * https://docs.rs/rusqlite/latest/src/rusqlite/types/to_sql.rs.html#257-262
     * */
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.to_string()))
    }
}

impl FromSql for Status {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        value.as_str().map(|s| Status::from_str(s).unwrap())
    }
}

fn main() -> Result<(), Box<Error>> {
    let db_conn = open_db_connection().expect("Failed open storage connection");

    create_task_table(&db_conn).expect("Failed to create table");

    let cmd_matches = construct_cmd_args().get_matches();

    if let Some(arg_matches) = cmd_matches.subcommand_matches("list") {
        handle_cmd_list(arg_matches, &db_conn);
    }

    if let Some(arg_matches) = cmd_matches.subcommand_matches("show") {
        handle_cmd_show(arg_matches, &db_conn);
    }

    if let Some(arg_matches) = cmd_matches.subcommand_matches("add") {
        handle_cmd_add(arg_matches, &db_conn);
    }

    if let Some(arg_matches) = cmd_matches.subcommand_matches("update") {
        handle_cmd_update(arg_matches, &db_conn);
    }

    if let Some(arg_matches) = cmd_matches.subcommand_matches("mark") {
        handle_cmd_mark(arg_matches, &db_conn);
    }

    if let Some(arg_matches) = cmd_matches.subcommand_matches("unmark") {
        handle_cmd_unmark(arg_matches, &db_conn);
    }

    if let Some(arg_matches) = cmd_matches.subcommand_matches("delete") {
        handle_cmd_delete(arg_matches, &db_conn);
    }

    Ok(())
}
