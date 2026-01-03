use chrono::{Datelike, Local, NaiveDate};
use clap::ArgMatches;

use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table};

use crate::Task;

pub fn construct_timestamp(arg_matches: &ArgMatches) -> NaiveDate {
    let mut timestamp = Local::now().date_naive();
    /*
     * reason of this year to day approach is only for day case
     * as no. of day will depend on the month
     *
     * Assume current date is some day (15th) in april month (support till 30th)
     * if user passes -d 31 -m 3 (31st march) -> valid
     *
     * in this case if we first modify day then will be modifing it to 31st april which is
     * invalid.
     * That's why we are first modifying month (if mentioned) so we will first convert it to
     * 15th march, then we will convert to 31st march
     *
     * */

    if let Some(year) = arg_matches.get_one::<u32>("year") {
        println!("Year no = {}", year);
        match timestamp.with_year(year.to_owned() as i32) {
            Some(date) => timestamp = date,
            None => {
                // invalid date
                panic!("Invalid date");
            }
        }
    }

    if let Some(month) = arg_matches.get_one::<u32>("month") {
        println!("Month no = {}", month);
        match timestamp.with_month(month.to_owned()) {
            Some(date) => timestamp = date,
            None => {
                // invalid date
                panic!("Invalid date");
            }
        }
    }

    if let Some(day) = arg_matches.get_one::<u32>("day") {
        println!("Day no = {}", day);
        match timestamp.with_day(day.to_owned()) {
            Some(date) => timestamp = date,
            None => {
                // invalid date
                panic!("Invalid date");
            }
        }
    }

    timestamp
}

pub fn iso_format_timestamp(timestamp: &NaiveDate) -> String {
    // iso date format by chrono
    // date + time
    // let formatted_timestamp = format!("{}", timestamp.format("%+"));

    // only date
    format!("{}", timestamp.format("%F"))
}

pub fn render_tasks_table(grouped_tasks: &Vec<(&String, &Vec<Task>)>, include_id: bool) {
    let mut tasks_table = Table::new();

    tasks_table
        .load_preset(comfy_table::presets::ASCII_FULL)
        .set_content_arrangement(ContentArrangement::DynamicFullWidth)
        .set_width(100);

    let header_cell = |title: &str| {
        Cell::new(title)
            .fg(Color::Rgb {
                r: 205,
                g: 214,
                b: 244,
            })
            .bg(Color::Rgb {
                r: 49,
                g: 50,
                b: 68,
            })
            .add_attribute(Attribute::Bold)
    };

    let mut headers = vec![
        header_cell(" Date "),
        header_cell(" Description "),
        header_cell(" Status "),
    ];

    if include_id {
        headers.push(header_cell(" ID "));
    } else {
        headers.push(header_cell(" Idx "));
    }

    tasks_table.set_header(headers);

    let mut last_used_date = "";
    for (date, tasks) in grouped_tasks.iter() {
        for (index, task) in tasks.iter().enumerate() {
            let display_date = if date.as_str() == last_used_date {
                ""
            } else {
                date
            };

            let mut cells = vec![
                Cell::new(display_date),
                Cell::new(&task.description).fg(Color::Red),
                Cell::new(&task.status),
            ];

            if include_id {
                cells.push(Cell::new(&task.id));
            } else {
                cells.push(Cell::new(index + 1));
            }

            tasks_table.add_row(cells);

            last_used_date = date;
        }
    }

    println!("{tasks_table}");
}
