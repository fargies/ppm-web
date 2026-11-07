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

use std::{
    ops::Deref,
    sync::{
        Arc, Weak,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use ppm::utils::serializers::human;
use salvo::{Depot, FlowCtrl, Handler, Request, Response, async_trait, server::ServerHandle};
use tokio::{sync::Mutex, task::JoinHandle, time::sleep};
use tracing::Instrument;

/**
 * IdleWatch for Salvo
 *
 * Shut the server down when idle
 */
#[derive(Clone)]
pub struct IdleWatch(Arc<IdleWatchData>);

impl IdleWatch {
    pub fn new(idle_delay: Duration) -> Self {
        Self(IdleWatchData::new(idle_delay))
    }
}

impl Deref for IdleWatch {
    type Target = Arc<IdleWatchData>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Default)]
struct IdleWatchHandles {
    server_handle: Option<ServerHandle>,
    timer_handle: Option<JoinHandle<()>>,
}

impl Drop for IdleWatchHandles {
    fn drop(&mut self) {
        self.timer_handle.take().inspect(JoinHandle::<()>::abort);
    }
}

#[derive(Default)]
pub struct IdleWatchData {
    handles: Mutex<IdleWatchHandles>,
    counter: AtomicUsize,
    idle_delay: Duration,
}

impl IdleWatchData {
    fn new(idle_delay: Duration) -> Arc<Self> {
        let ret = Arc::new(Self {
            idle_delay,
            ..Default::default()
        });
        if !ret.idle_delay.is_zero() {
            let ret = Arc::clone(&ret);
            tokio::spawn(async move { ret.start_timer().await });
        }
        ret
    }

    #[tracing::instrument(name = "idle_watch:token", skip(self))]
    pub async fn token(self: &Arc<Self>) -> IdleWatchToken {
        let count = self.counter.fetch_add(1, Ordering::Relaxed) + 1;
        tracing::debug!(count);
        if count == 1 && !self.idle_delay.is_zero() {
            let mut guard = self.handles.lock().await;
            guard.timer_handle.take().inspect(JoinHandle::<()>::abort);
        }
        IdleWatchToken(Some(self.clone()))
    }

    pub async fn set_handle(&self, server_handle: ServerHandle) {
        self.handles.lock().await.server_handle = Some(server_handle);
    }

    #[tracing::instrument(name = "idle_watch:timer_routine", skip(watch))]
    async fn timer_routine(watch: Weak<Self>) {
        let idle_delay = match watch.upgrade() {
            Some(idle_watch) => idle_watch.idle_delay,
            None => return,
        };

        tracing::debug!(
            idle_delay = human::duration::format(idle_delay).to_string(),
            "starting idle delay"
        );
        sleep(idle_delay).await;
        if let Some(data) = watch.upgrade() {
            match data.handles.lock().await.server_handle.as_ref() {
                Some(server_handle) => {
                    tracing::info!("server idle, graceful shutdown");
                    server_handle.stop_graceful(Duration::from_secs(30));
                }
                None => {
                    tracing::warn!("no server handle, shutdown impossible");
                }
            }
        }
    }

    async fn start_timer(self: &Arc<Self>) {
        let mut guard = self.handles.lock().await;
        guard.timer_handle.take().inspect(JoinHandle::<()>::abort);
        guard.timer_handle = Some(tokio::spawn(
            Self::timer_routine(Arc::downgrade(self))
                .instrument(tracing::info_span!("ppm-web")),
        ));
    }

    async fn release_token(self: &Arc<Self>) {
        if self.counter.fetch_sub(1, Ordering::Relaxed) <= 1 && !self.idle_delay.is_zero() {
            self.start_timer().await;
        }
    }
}

pub struct IdleWatchToken(Option<Arc<IdleWatchData>>);

impl Drop for IdleWatchToken {
    fn drop(&mut self) {
        let data = self.0.take().unwrap();
        tokio::task::spawn(async move { data.release_token().await });
    }
}

#[async_trait]
impl Handler for IdleWatch {
    async fn handle(
        &self,
        req: &mut Request,
        depot: &mut Depot,
        res: &mut Response,
        ctrl: &mut FlowCtrl,
    ) {
        let _token = self.token().await;
        ctrl.call_next(req, depot, res).await;
    }
}
