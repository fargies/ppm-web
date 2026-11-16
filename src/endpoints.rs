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
use ppm::{
    api::{ActionResult, get_server_addr},
    utils::pre_json::JsonPreParser,
};
use salvo::{
    hyper::body::Bytes,
    prelude::*,
    websocket::{Message, WebSocket, WebSocketUpgrade},
};
use std::{net::SocketAddr, sync::Arc};
use tokio::io::AsyncWriteExt;

use crate::{client::Client, pool::Pool, utils::IdleWatch};

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
}

struct ConnectContext {
    pool: Arc<Pool<Client>>,
    addr: SocketAddr,
    idle_watch: Option<IdleWatch>,
    remote_addr: salvo::conn::SocketAddr,
    is_raw: bool,
}

impl ConnectContext {
    #[tracing::instrument(name = "ppm-web:ws", skip_all)]
    pub async fn connect(self, mut ws: WebSocket) {
        let _token = match self.idle_watch {
            Some(idle_watch) => Some(idle_watch.token().await),
            None => None,
        };

        tracing::trace!(remote_addr=?self.remote_addr, self.is_raw, "websocket connected");
        let ret = if self.is_raw {
            ws_raw(&mut ws, self.pool, self.addr).await
        } else {
            ws_handle(&mut ws, self.pool, self.addr).await
        };
        if let Err(err) = ret {
            tracing::error!(?err, "websocket error");
            let _ = ws
                .send(Message::binary(
                    serde_json::to_vec(&ActionResult::<()>::Err(format!("server error: {err:?}")))
                        .unwrap(),
                ))
                .await;
        };
        let _ = ws.close().await;
        tracing::trace!(remote_addr=?self.remote_addr, "websocket closed");
    }
}

#[handler]
#[tracing::instrument(parent = None, name = "ppm-web:ws", skip_all)]
pub async fn connect(
    req: &mut Request,
    res: &mut Response,
    depot: &Depot,
) -> Result<(), StatusError> {
    let ctx = ConnectContext {
        pool: depot
            .get_typed::<Arc<Pool<Client>>>()
            .map_err(|_| {
                tracing::error!("failed to acquire pool");
                StatusError::internal_server_error()
            })?
            .clone(),
        addr: depot
            .get::<SocketAddr>("addr")
            .copied()
            .unwrap_or_else(|_| get_server_addr()),
        idle_watch: depot.get::<IdleWatch>("idle_watch").cloned().ok(),
        remote_addr: req.remote_addr().clone(),
        is_raw: req.query::<String>("raw").is_some(),
    };

    WebSocketUpgrade::new()
        .upgrade(req, res, async move |ws| ctx.connect(ws).await)
        .await
}
