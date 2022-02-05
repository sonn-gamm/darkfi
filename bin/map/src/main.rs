use darkfi::{
    error::{Error, Result},
    rpc::{jsonrpc, jsonrpc::JsonResult},
    util::async_util,
};

use async_std::sync::{Arc, Mutex};
use easy_parallel::Parallel;
use log::debug;
use serde_json::{json, Value};
use smol::Executor;
use std::{io, io::Read};
use termion::{async_stdin, event::Key, input::TermRead, raw::IntoRawMode};
use tui::{
    backend::{Backend, TermionBackend},
    Terminal,
};

use map::{id_list::IdList, info_list::InfoList, node_info::NodeInfo, ui, Model};

struct Map {
    url: String,
}

impl Map {
    pub fn new(url: String) -> Self {
        Self { url }
    }

    async fn request(&self, r: jsonrpc::JsonRequest) -> Result<Value> {
        let reply: JsonResult = match jsonrpc::send_request(&self.url, json!(r)).await {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        match reply {
            JsonResult::Resp(r) => {
                debug!(target: "RPC", "<-- {}", serde_json::to_string(&r)?);
                Ok(r.result)
            }

            JsonResult::Err(e) => {
                debug!(target: "RPC", "<-- {}", serde_json::to_string(&e)?);
                Err(Error::JsonRpcError(e.error.message.to_string()))
            }

            JsonResult::Notif(n) => {
                debug!(target: "RPC", "<-- {}", serde_json::to_string(&n)?);
                Err(Error::JsonRpcError("Unexpected reply".to_string()))
            }
        }
    }

    // --> {"jsonrpc": "2.0", "method": "say_hello", "params": [], "id": 42}
    // <-- {"jsonrpc": "2.0", "result": "hello world", "id": 42}
    async fn _say_hello(&self) -> Result<Value> {
        let req = jsonrpc::request(json!("say_hello"), json!([]));
        Ok(self.request(req).await?)
    }

    //--> {"jsonrpc": "2.0", "method": "poll", "params": [], "id": 42}
    // <-- {"jsonrpc": "2.0", "result": {"nodeID": [], "nodeinfo" [], "id": 42}
    async fn get_info(&self) -> Result<Value> {
        let req = jsonrpc::request(json!("get_info"), json!([]));
        Ok(self.request(req).await?)
    }
}

#[async_std::main]
async fn main() -> Result<()> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;

    let infos = vec![
        NodeInfo {
            id: "0385048034sodisofjhosd1111q3434".to_string(),
            connections: 10,
            is_active: true,
            last_message: "hey how are you?".to_string(),
        },
        NodeInfo {
            id: "09w30we9wsnfksdfkdjflsjkdfjdfsd".to_string(),
            connections: 5,
            is_active: false,
            last_message: "lmao".to_string(),
        },
        NodeInfo {
            id: "038043325alsdlasjfrsdfsdfsdjsdf".to_string(),
            connections: 7,
            is_active: true,
            last_message: "gm".to_string(),
        },
        NodeInfo {
            id: "04985034953ldflsdjflsdjflsdjfii".to_string(),
            connections: 2,
            is_active: true,
            last_message: "hihi".to_string(),
        },
        NodeInfo {
            id: "09850249352asdjapsdikalskasdkas".to_string(),
            connections: 10,
            is_active: true,
            last_message: "wtf".to_string(),
        },
    ];

    let info_list = InfoList::new(infos.clone());

    let ids = vec![
        infos[0].id.clone(),
        infos[1].id.clone(),
        infos[2].id.clone(),
        infos[3].id.clone(),
        infos[4].id.clone(),
    ];

    let id_list = IdList::new(ids);

    //let app = Arc::new(Model::new(id_list, info_list));
    let app = Model::new(id_list, info_list);

    let nthreads = num_cpus::get();
    let (signal, shutdown) = async_channel::unbounded::<()>();

    let ex = Arc::new(Executor::new());
    let ex2 = ex.clone();

    let (_, result) = Parallel::new()
        .each(0..nthreads, |_| smol::future::block_on(ex.run(shutdown.recv())))
        // Run the main future on the current thread.
        .finish(|| {
            smol::future::block_on(async move {
                run_rpc(ex2.clone(), app.clone()).await?;
                run_app(&mut terminal, app.clone()).await?;
                drop(signal);
                Ok::<(), darkfi::Error>(())
            })
        });

    result
}

async fn run_rpc(ex: Arc<Executor<'_>>, app: Model) -> Result<()> {
    let client = Map::new("tcp://127.0.0.1:8000".to_string());

    ex.spawn(poll(client, app)).detach();

    Ok(())
}

async fn poll(client: Map, _app: Model) -> Result<()> {
    loop {
        let reply = client.get_info().await?;

        if reply.as_object().is_some() && !reply.as_object().unwrap().is_empty() {
            let nodes = reply.as_object().unwrap().get("nodes").unwrap();

            let node1 = &nodes[0];
            let node2 = &nodes[1];
            let node3 = &nodes[2];

            let _infos = vec![NodeInfo {
                id: node1["id"].to_string(),
                connections: node1["connections"].as_u64().unwrap() as usize,
                is_active: node2["is_active"].as_bool().unwrap(),
                last_message: node3["message"].to_string(),
            }];

            //app.lock().await.update(infos).await;
        } else {
            // TODO: error handling
            println!("Reply is an error");
        }

        async_util::sleep(1).await;
    }
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: Model) -> io::Result<()> {
    let mut asi = async_stdin();

    terminal.clear()?;

    app.id_list.state.select(Some(0));

    app.info_list.index = 0;

    // acquire the mutex
    // let mut app = app.lock();

    loop {
        // clone everything
        terminal.draw(|f| {
            ui::ui(f, app.clone());
        })?;
        for k in asi.by_ref().keys() {
            match k.unwrap() {
                Key::Char('q') => {
                    terminal.clear()?;
                    return Ok(());
                }
                Key::Char('j') => {
                    app.id_list.next();
                    app.info_list.next().await;
                }
                Key::Char('k') => {
                    app.id_list.previous();
                    app.info_list.previous().await;
                }
                _ => (),
            }
        }
    }
}
