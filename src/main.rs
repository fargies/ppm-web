/*
** Copyright (C) 2025 Sylvain Fargier
**
** This software is provided 'as-is', without any express or implied
** warranty.  In no event will the authors be held liable for any damages
** arising from the use of this software.
**
** Permission is granted to anyone to use this software for any purpose,
** including commercial applications, and to alter it and redistribute it
** freely, subject to the following restrictions:
**
** 1. The origin of this software must not be misrepresented; you must not
**    claim that you wrote the original software. If you use this software
**    in a product, an acknowledgment in the product documentation would be
**    appreciated but is not required.
** 2. Altered source versions must be plainly marked as such, and must not be
**    misrepresented as being the original software.
** 3. This notice may not be removed or altered from any source distribution.
**
** Author: Sylvain Fargier <fargier.sylvain@gmail.com>
*/

use anyhow::{Context, Result, anyhow};
use clap::Parser;
use ppm::{
    api::{ActionResult, get_server_addr},
    utils::{libc::Dup, pre_json::JsonPreParser, serializers::human, tracing_utils::tracing_init},
};
use salvo::{
    async_trait,
    hyper::body::Bytes,
    logging::Logger,
    prelude::*,
    serve_static::static_embed,
    trailing_slash::remove_slash,
    websocket::{Message, WebSocket, WebSocketUpgrade},
};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    os::fd::IntoRawFd,
    sync::Arc,
    time::Duration,
};
use tokio::io::AsyncWriteExt;

mod client;
mod pool;
mod utils;
use crate::{
    client::Client,
    pool::Pool,
    utils::{IdleWatch, UnixDatagramAcceptor},
};

const DEFAULT_WEBAPP_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3000);
const MAX_CONNECTIONS: usize = 255;

#[derive(rust_embed::RustEmbed)]
#[folder = "src/app/dist"]
struct Assets;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(long, global = true, default_value_t = get_server_addr())]
    pub addr: SocketAddr,
    #[arg(long, default_value_t = false)]
    pub unix_socket: bool,
    #[arg(long, value_parser = human::duration::parse, default_value = "0")]
    pub idle_shutdown: Duration,
}

pub struct PPMHoop {
    logger: Logger,
    idle_watch: IdleWatch,
}

impl PPMHoop {
    pub fn new(idle_watch: IdleWatch) -> Self {
        Self {
            logger: Logger::new(),
            idle_watch,
        }
    }
}

#[async_trait]
impl Handler for PPMHoop {
    #[tracing::instrument(name = "ppm-web", skip_all)]
    async fn handle(
        &self,
        req: &mut Request,
        depot: &mut Depot,
        res: &mut Response,
        ctrl: &mut FlowCtrl,
    ) {
        self.logger.handle(req, depot, res, ctrl).await;
        self.idle_watch.handle(req, depot, res, ctrl).await;
        ctrl.call_next(req, depot, res).await;
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_init(std::io::stdout, Some("info"))?;
    let _span = tracing::info_span!(parent: None, "ppm-web").entered();
    let args = Args::parse();
    let idle_watch = IdleWatch::new(args.idle_shutdown);

    let router = Router::new()
        .hoop(
            affix_state::inject(Pool::<Client>::new(3))
                .insert("addr", args.addr)
                .insert("idle_watch", idle_watch.clone()),
        )
        .hoop(PPMHoop::new(idle_watch.clone()))
        .push(Router::with_path("ws").goal(connect))
        .push(
            Router::with_path("{*path}")
                .hoop(remove_slash())
                .get(static_embed::<Assets>().fallback("index.html")),
        );

    let addr = std::env::var("PPMWEB_LISTEN")
        .ok()
        .and_then(|x| x.parse::<SocketAddr>().ok())
        .unwrap_or(DEFAULT_WEBAPP_ADDR);

    tracing::info!("starting ppm-web");
    if args.unix_socket {
        let acceptor = UnixDatagramAcceptor::try_from(std::io::stdin().dup()?.into_raw_fd())?;
        let server = Server::new(acceptor).max_connections(MAX_CONNECTIONS);
        idle_watch.set_handle(server.handle()).await;
        server.serve(router).await
    } else {
        let acceptor = TcpListener::new(addr).bind().await;
        let server = Server::new(acceptor).max_connections(MAX_CONNECTIONS);
        idle_watch.set_handle(server.handle()).await;
        server.serve(router).await
    };

    Ok(())
}

async fn ws_handle(
    ws: &mut WebSocket,
    pool: Arc<Pool<Client>>,
    addr: SocketAddr,
) -> anyhow::Result<()> {
    while let Some(msg) = ws.recv().await {
        let msg = msg.context("websocket error")?;
        if msg.is_close() {
            tracing::trace!("client closed");
            return Ok(());
        }

        let mut client = pool
            .acquire(&addr)
            .await
            .context("failed to acquire client")?;

        let reply = client
            .raw_invoke(msg.as_bytes())
            .await
            .context("failed to invoke method")?;
        ws.send(Message::binary(Bytes::copy_from_slice(reply)))
            .await?;
    }
    Ok(())
}

async fn ws_raw(
    ws: &mut WebSocket,
    pool: Arc<Pool<Client>>,
    addr: SocketAddr,
) -> anyhow::Result<()> {
    /* destructure the client */
    let Client {
        stream,
        read_buffer: JsonPreParser { mut buffer, .. },
    } = pool
        .acquire(&addr)
        .await
        .context("failed to acquire client")?
        .into_inner()
        .context("failed to extract client")?;
    let mut stream = match stream {
        Some(stream) => stream,
        None => return Err(anyhow!("client closed")),
    };

    loop {
        tokio::select! {
            msg = ws.recv() => {
                tracing::debug!("websocket msg received");
                let msg = match msg {
                    Some(msg) => msg.context("websocket error")?,
                    None => {
                        tracing::trace!("client closed");
                        return Ok(())
                    },
                };
                if msg.is_close() {
                    tracing::trace!("client close message");
                    return Ok(());
                }
                tracing::debug!("websocket msg sending");
                stream.write_all(msg.as_bytes()).await?;
                tracing::debug!("websocket msg sent");
            }
            ret = stream.readable() => {
                ret?;
                tracing::debug!("remote readable");
                buffer.clear();
                match
                stream.try_read_buf(&mut buffer) {
                    Ok(0) => return Err(anyhow!("remote connection closed")),
                    Ok(_) => (),
                    Err(ref e) if e.kind() == tokio::io::ErrorKind::WouldBlock => {
                        continue;
                    }
                    Err(e) => {
                        return Err(e.into());
                    }
                }
                ws.send(Message::binary(Bytes::copy_from_slice(buffer.as_slice()))).await?;
            }
        }
    }
    // Ok(())
}

#[handler]
async fn connect(req: &mut Request, res: &mut Response, depot: &Depot) -> Result<(), StatusError> {
    let pool: Arc<Pool<Client>> = depot
        .get_typed::<Arc<Pool<Client>>>()
        .map_err(|_| {
            tracing::error!("failed to acquire pool");
            StatusError::internal_server_error()
        })?
        .clone();
    let addr = depot
        .get::<SocketAddr>("addr")
        .copied()
        .unwrap_or_else(|_| get_server_addr());
    let idle_watch = depot.get::<IdleWatch>("idle_watch").cloned().ok();
    let remote_addr = req.remote_addr().clone();
    let is_raw = req.query::<String>("raw").is_some();

    WebSocketUpgrade::new()
        .upgrade(req, res, async move |mut ws| {
            let _token = match idle_watch {
                Some(idle_watch) => Some(idle_watch.token().await),
                None => None,
            };
            tracing::trace!(remote_addr=?remote_addr, is_raw, "websocket connected");
            let ret = if is_raw {
                ws_raw(&mut ws, pool, addr).await
            } else {
                ws_handle(&mut ws, pool, addr).await
            };
            if let Err(err) = ret {
                tracing::error!(?err, "websocket error");
                let _ = ws
                    .send(Message::binary(
                        serde_json::to_vec(&ActionResult::<()>::Err(format!(
                            "server error: {err:?}"
                        )))
                        .unwrap(),
                    ))
                    .await;
            };
            let _ = ws.close().await;
            tracing::trace!(remote_addr=?remote_addr, "websocket closed");
        })
        .await
}
