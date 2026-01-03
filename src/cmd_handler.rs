use std::collections::HashMap;

use chrono::{Datelike, Local};
use clap::{arg, builder, value_parser, Arg, ArgMatches, Command};
use rusqlite::Connection;

use crate::{
    database::{get_tasks_by_date, insert_task, update_task_status},
    render_tasks_table,
    utils::{construct_timestamp, iso_format_timestamp},
    Status, Task,
};

pub fn construct_cmd_args() -> Command {
    Command::new("Daily Dose")
        .version("1.0.0")
        .about("Record your daily dose of pain")
        .subcommands([
            Command::new("list")
                .about("List multiple standups based on timeline")
                .args([
                    arg!(-m --month <MONTH_NO> "List standups for specified month (eg. 1, 2, 3)")
                        .value_parser(value_parser!(u32).range(1..=12))
                        .required(false),
                    arg!(-l --limit <LIMIT> "Limit no. of standups in result")
                        .value_parser(value_parser!(u32).range(1..))
                        .required(false),
                    arg!(-s --search <QUERY> "Search query(task) for searching")
                        .value_parser(builder::NonEmptyStringValueParser::new())
                        .required(false),
                    Arg::new("include-id")
                        .long("include-id")
                        .action(clap::ArgAction::SetTrue),
                ]),
            Command::new("show")
                .about("Show tasks for any specific date")
                .args([
                    arg!(-d --day <DAY_NO> "Day for which fetching standup")
                        .value_parser(value_parser!(u32).range(1..=31))
                        .required(false),
                    arg!(-m --month <MONTH_NO> "Month for which fetching standup")
                        .value_parser(value_parser!(u32).range(1..=12))
                        .required(false),
                    arg!(-y --year <YEAR_NO> "Year for which fetching standup")
                        .value_parser(value_parser!(u32).range(1978..))
                        .required(false),
                    Arg::new("include-id")
                        .long("include-id")
                        .action(clap::ArgAction::SetTrue),
                ]),
            Command::new("add")
                .about("Add a task to current or specific date's standup task list")
                .args([
                    arg!([TASK] "Task description")
                        .value_parser(builder::NonEmptyStringValueParser::new())
                        .required(true),
                    arg!(-d --day <DAY_NO> "Day for which fetching standup")
                        .value_parser(value_parser!(u32).range(1..=31))
                        .required(false),
                    arg!(-m --month <MONTH_NO> "Month for which fetching standup")
                        .value_parser(value_parser!(u32).range(1..=12))
                        .required(false),
                    arg!(-y --year <YEAR_NO> "Year for which fetching standup")
                        .value_parser(value_parser!(u32).range(1978..))
                        .required(false),
                ]),
            Command::new("update")
                .about("Update a task based on task id")
                .args([
                    arg!([TASK] "Task description")
                        .value_parser(builder::NonEmptyStringValueParser::new())
                        .required(true),
                    arg!(--id <TASK_ID> "Task ID to update on")
                        .value_parser(builder::NonEmptyStringValueParser::new()),
                ]),
            Command::new("mark")
                .about("Mark today's specific task as done")
                .arg(
                    arg!([TASK_INDEX] "Mark current date's task based on task index")
                        .value_parser(value_parser!(u8).range(1..=100))
                        .required(true),
                ),
            Command::new("unmark")
                .about("Unmark today's specific task as todo")
                .arg(
                    arg!([TASK_INDEX] "Mark current date's task based on task index")
                        .value_parser(value_parser!(u8).range(1..=100))
                        .required(true),
                ),
            Command::new("delete")
                .about("Delete a task based on task id")
                .arg(
                    arg!(--id <TASK_ID> "Task ID to delete")
                        .value_parser(builder::NonEmptyStringValueParser::new()),
                ),
        ])
}

pub fn handle_cmd_list(arg_matches: &ArgMatches, db_conn: &Connection) {
    let mut now = Local::now().date_naive();

    let get_include_id_flag = arg_matches.get_flag("include-id");

    if let Some(month_no) = arg_matches.get_one::<u32>("month") {
        now = now.with_month(*month_no).expect("Invalid month");
    }

    let start_date = iso_format_timestamp(&now.with_day(1).expect("Internal Error: Invalid day"));
    let end_date = iso_format_timestamp(&now);

    match get_tasks_by_date(&db_conn, &start_date, Some(&end_date)) {
        Ok(tasks) => {
            let mut date_tasks_map: HashMap<String, Vec<Task>> = HashMap::new();
            for task in tasks {
                if date_tasks_map.contains_key(&task.date) {
                    let task_list = date_tasks_map.get_mut(&task.date);

                    match task_list {
                        Some(list) => {
                            list.push(task);
                        }
                        None => {
                            date_tasks_map.insert(task.date.clone(), vec![task]);
                        }
                    }
                } else {
                    date_tasks_map.insert(task.date.clone(), vec![task]);
                }
            }

            let mut task_grouped_by_date: Vec<(&String, &Vec<Task>)> =
                date_tasks_map.iter().collect();

            // sorting by date
            task_grouped_by_date.sort_by(|a, b| b.0.cmp(a.0));

            render_tasks_table(&task_grouped_by_date, get_include_id_flag);
        }
        Err(error) => println!("Error fetching tasks = {error}"),
    }
}

pub fn handle_cmd_show(arg_matches: &ArgMatches, db_conn: &Connection) {
    let timestamp = construct_timestamp(arg_matches);

    let get_include_id_flag = arg_matches.get_flag("include-id");

    let start_date = iso_format_timestamp(&timestamp);

    match get_tasks_by_date(db_conn, &start_date, None) {
        Ok(tasks) => render_tasks_table(&vec![(&start_date, &tasks)], get_include_id_flag),
        Err(error) => println!("Error getting tasks for date = {error}"),
    }
}

pub fn handle_cmd_add(arg_matches: &ArgMatches, db_conn: &Connection) {
    let task_description = arg_matches
        .get_one::<String>("TASK")
        .expect("Task description is required for add");
    println!("Task description = {}", task_description);

    let timestamp = construct_timestamp(arg_matches);

    let iso_timestamp = iso_format_timestamp(&timestamp);

    if let Err(error) = insert_task(db_conn, task_description, Status::Todo, &iso_timestamp) {
        println!("Error inserting new task = {:?}", error);
    }
}

pub fn handle_cmd_update(arg_matches: &ArgMatches, _db_conn: &Connection) {
    println!("Update sub command matches = {:?}", arg_matches);
}

pub fn handle_cmd_delete(arg_matches: &ArgMatches, _db_conn: &Connection) {
    println!("Deleted sub command matches = {:?}", arg_matches);
}

pub fn handle_cmd_mark(arg_matches: &ArgMatches, db_conn: &Connection) {
    let now = Local::now().date_naive();

    let task_index = arg_matches
        .get_one::<u8>("TASK_INDEX")
        .expect("Tasks Index is required");

    let start_date = iso_format_timestamp(&now);

    let tasks = get_tasks_by_date(db_conn, &start_date, None).expect("Failed to fetch tasks");

    let selected_row = tasks
        .get(*task_index as usize - 1)
        .expect("Error: Index outbound");

    update_task_status(db_conn, &selected_row.id, Status::Done).expect("Failed to update task");
}

pub fn handle_cmd_unmark(arg_matches: &ArgMatches, db_conn: &Connection) {
    let now = Local::now().date_naive();

    let task_index = arg_matches
        .get_one::<u8>("TASK_INDEX")
        .expect("Tasks Index is required");

    let start_date = iso_format_timestamp(&now);

    let tasks = get_tasks_by_date(db_conn, &start_date, None).expect("Failed to fetch tasks");

    let selected_row = tasks
        .get(*task_index as usize - 1)
        .expect("Error: Index outbound");

    update_task_status(db_conn, &selected_row.id, Status::Todo).expect("Failed to update task");
}
