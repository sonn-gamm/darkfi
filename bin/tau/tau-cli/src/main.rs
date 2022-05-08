use log::error;
use serde_json::json;
use simplelog::{ColorChoice, TermLogger, TerminalMode};
use structopt_toml::StructOptToml;

use darkfi::{
    util::{
        cli::{log_config, spawn_config},
        path::get_config_path,
    },
    Result,
};

mod cli;
mod filter;
mod jsonrpc;
mod primitives;
mod util;
mod view;

use cli::CliTauSubCommands;
use primitives::TaskInfo;
use util::{desc_in_editor, CONFIG_FILE, CONFIG_FILE_CONTENTS};
use view::{comments_as_string, events_as_string, print_list_of_task, print_task_info};

async fn start(mut options: cli::CliTau) -> Result<()> {
    let rpc_addr = &format!("tcp://{}", &options.rpc_listen.clone());

    let states: Vec<String> = vec!["stop".into(), "open".into(), "pause".into()];

    match options.id {
        Some(id) if id.len() < 4 && id.parse::<u64>().is_ok() => {
            let task = jsonrpc::get_task_by_id(rpc_addr, id.parse::<u64>().unwrap()).await?;
            let taskinfo: TaskInfo = serde_json::from_value(task.clone())?;
            print_task_info(taskinfo)?;
            return Ok(())
        }
        Some(id) => options.filters.push(id),
        None => {}
    }

    match options.command {
        Some(CliTauSubCommands::Add { values }) => {
            let mut task = cli::task_from_cli_values(values)?;
            if task.title.is_empty() {
                error!("Provide a title for the task");
                return Ok(())
            };

            if task.desc.is_none() {
                task.desc = desc_in_editor()?;
            };

            jsonrpc::add(rpc_addr, json!([task])).await?;
        }

        Some(CliTauSubCommands::Update { id, values }) => {
            let task = cli::task_from_cli_values(values)?;
            jsonrpc::update(rpc_addr, id, json!([task])).await?;
        }

        Some(CliTauSubCommands::State { id, state }) => match state {
            Some(state) => {
                let state = state.trim().to_lowercase();
                if states.contains(&state) {
                    jsonrpc::set_state(rpc_addr, id, &state).await?;
                } else {
                    error!("Task state could only be one of three states: open, pause or stop");
                }
            }
            None => {
                let task = jsonrpc::get_task_by_id(rpc_addr, id).await?;
                let taskinfo: TaskInfo = serde_json::from_value(task.clone())?;
                let state = events_as_string(taskinfo.events);
                println!("Task {}: {}", id, state);
            }
        },

        Some(CliTauSubCommands::Comment { id, content }) => match content {
            Some(content) => {
                jsonrpc::set_comment(rpc_addr, id, content.trim()).await?;
            }
            None => {
                let task = jsonrpc::get_task_by_id(rpc_addr, id).await?;
                let taskinfo: TaskInfo = serde_json::from_value(task.clone())?;
                let comments = comments_as_string(taskinfo.comments);
                println!("Comments {}:\n{}", id, comments);
            }
        },

        Some(CliTauSubCommands::List {}) | None => {
            let tasks = jsonrpc::list(rpc_addr, json!([])).await?;
            let mut tasks: Vec<TaskInfo> = serde_json::from_value(tasks)?;
            print_list_of_task(&mut tasks, options.filters)?;
        }
    }

    Ok(())
}

#[async_std::main]
async fn main() -> Result<()> {
    let args = cli::CliTau::from_args_with_toml("").unwrap();
    let cfg_path = get_config_path(args.config, CONFIG_FILE)?;
    spawn_config(&cfg_path, CONFIG_FILE_CONTENTS.as_bytes())?;
    let args = cli::CliTau::from_args_with_toml(&std::fs::read_to_string(cfg_path)?).unwrap();

    let (lvl, conf) = log_config(args.verbose.into())?;
    TermLogger::init(lvl, conf, TerminalMode::Mixed, ColorChoice::Auto)?;

    start(args).await
}
