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

use anyhow::Result;
use clap::Parser;
use ppm::{
    api::get_server_addr,
    utils::{
        libc::{Dup, SocketTrait},
        serializers::human,
        tracing_utils::tracing_init,
    },
};
use salvo::{
    async_trait, logging::Logger, prelude::*, serve_static::static_embed, server::ServerHandle,
    trailing_slash::remove_slash,
};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    os::fd::RawFd,
    time::Duration,
};
use tokio::signal;

mod client;
mod endpoints;
mod pool;
mod utils;
use crate::{
    client::Client,
    pool::Pool,
    utils::{IdleWatch, inetd},
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
    #[arg(long, value_parser = human::duration::parse, default_value = "0")]
    pub idle_shutdown: Duration,
}

pub struct PPMHoop<H>(pub H);

#[async_trait]
impl<H> Handler for PPMHoop<H>
where
    H: Handler,
{
    #[tracing::instrument(parent = None, name = "ppm-web:hoop", skip_all)]
    async fn handle(
        &self,
        req: &mut Request,
        depot: &mut Depot,
        res: &mut Response,
        ctrl: &mut FlowCtrl,
    ) {
        self.0.handle(req, depot, res, ctrl).await;
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_init(
        std::io::stdout,
        Some(
            "error,salvo_extra::logging=info,ppm_web=info,salvo_core::server=info,tungstenite=error,tokio_tungstenite=error",
        ),
    )?;
    let _span = tracing::info_span!(parent: None, "ppm-web").entered();

    let args = Args::parse();
    let idle_watch = IdleWatch::new(args.idle_shutdown);

    let router = Router::new()
        .hoop(PPMHoop(Logger::new()))
        .hoop(
            affix_state::inject(Pool::<Client>::new(3))
                .insert("addr", args.addr)
                .insert("idle_watch", idle_watch.clone()),
        )
        .hoop(PPMHoop(idle_watch.clone()))
        .push(Router::with_path("ws").goal(endpoints::connect))
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
    let acceptor = {
        if let stdin = (0 as RawFd)
            && cfg_select! {
                target_os = "linux" => stdin.is_listening(),
                _ => stdin.is_socket()
            }
        {
            tracing::info!("xinetd mode detected");
            inetd::make_acceptor(stdin.dup()?)?
        } else {
            TcpListener::new(addr).bind().await
        }
    };

    let server = Server::new(acceptor).max_connections(MAX_CONNECTIONS);
    idle_watch.set_handle(server.handle()).await;

    tokio::spawn(shutdown_signal(server.handle()));
    server.serve(router).await;

    Ok(())
}

async fn shutdown_signal(handle: ServerHandle) {
    signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C handler");
    tracing::info!("termination request");
    handle.stop_graceful(None);
}
