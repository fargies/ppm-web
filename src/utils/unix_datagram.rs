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

use ppm::utils::libc::{
    Fcntl, FdFlags,
    unix_datagram::{ControlMessage, ControlRequest, SocketControlTrait},
};
use salvo::{
    ConnCtrl,
    conn::{Accepted, Acceptor, Holding, SocketAddr, StraightStream, tcp::TcpCoupler},
    fuse::{ArcFusePolicy, FuseAction, FuseInfo, TransProto},
    http::{Version, uri::Scheme},
};
use std::{
    io::{self, Result as IoResult},
    os::fd::{FromRawFd, OwnedFd, RawFd},
};
use tokio::{
    io::Interest,
    net::{TcpStream, UnixDatagram},
};

#[derive(Debug)]
pub struct UnixDatagramAcceptor {
    inner: UnixDatagram,
    holdings: Vec<Holding>,
    pending: Option<(TcpStream, SocketAddr)>,
}

impl TryFrom<RawFd> for UnixDatagramAcceptor {
    type Error = io::Error;

    fn try_from(value: RawFd) -> Result<Self, Self::Error> {
        let inner = UnixDatagram::from_std(unsafe {
            value.add_flag(FdFlags::NONBLOCK)?;
            std::os::unix::net::UnixDatagram::from_raw_fd(value)
        })?;
        let holdings = vec![Holding {
            local_addr: inner.local_addr()?.into(),
            http_versions: vec![Version::HTTP_11, Version::HTTP_2],
            http_scheme: Scheme::HTTP,
        }];
        Ok(Self {
            inner,
            holdings,
            pending: None,
        })
    }
}

impl Acceptor for UnixDatagramAcceptor {
    type Coupler = TcpCoupler<Self::Stream>;
    type Stream = StraightStream<TcpStream>;

    #[inline]
    fn holdings(&self) -> &[Holding] {
        &self.holdings
    }

    #[inline]
    async fn accept(
        &mut self,
        fuse_policy: Option<ArcFusePolicy>,
    ) -> IoResult<Accepted<TcpCoupler<Self::Stream>, Self::Stream>> {
        loop {
            // Accept a raw socket, or resume one that a cancelled call left behind. It is
            // parked in `self.pending` across the async admission below so a dropped future
            // does not close an already-accepted client.
            if self.pending.is_none() {
                self.pending = self
                    .inner
                    .async_io(Interest::READABLE, || {
                        let mut buf = [0u8; 255];
                        let mut ctrl = ControlRequest::with_capacity(64);
                        let len = self.inner.recvmsg(&mut buf, &mut ctrl)?;
                        let remote_addr: SocketAddr =
                            serde_json::from_slice::<std::net::SocketAddr>(&buf[..len])
                                .map_err(|err| {
                                    tracing::warn!(?err, "invalid remote_addr in packet");
                                    io::Error::new(
                                        io::ErrorKind::InvalidData,
                                        "invalid remote_addr in packet",
                                    )
                                })?
                                .into();
                        let mut ret = None;
                        for msg in ctrl.iter() {
                            match msg {
                                ControlMessage::Rights(items) => {
                                    let mut iter = items.iter();
                                    let sock = unsafe {
                                        std::net::TcpStream::from_raw_fd(*iter.next().ok_or(
                                            io::Error::new(
                                                io::ErrorKind::InvalidData,
                                                "invalid SCM_RIGHTS packet",
                                            ),
                                        )?)
                                    };
                                    sock.set_nonblocking(true)?;
                                    iter.for_each(|fd| {
                                        tracing::warn!(fd, "extra SCM_RIGHTS fds");
                                        unsafe {
                                            OwnedFd::from_raw_fd(*fd);
                                        }
                                    });
                                    ret = Some((TcpStream::from_std(sock)?, remote_addr.clone()));
                                }
                                ControlMessage::Unknown {
                                    cmsg_level,
                                    cmsg_type,
                                    ..
                                } => {
                                    tracing::warn!(cmsg_level, cmsg_type, "unknown ControlMessage")
                                }
                            }
                        }
                        Ok(ret)
                    })
                    .await?;
            }

            let remote_addr = match self.pending.as_ref() {
                Some(p) => p.1.clone(),
                None => continue,
            };
            let local_addr = self.holdings[0].local_addr.clone();
            let conn_ctrl = ConnCtrl::new();
            let (fuse_config, observer) = match &fuse_policy {
                Some(policy) => {
                    let info = FuseInfo {
                        trans_proto: TransProto::Tcp,
                        remote_addr: remote_addr.clone(),
                        local_addr: local_addr.clone(),
                    };
                    match policy.decide(&info).await {
                        FuseAction::Accept(config) => {
                            (Some(config), policy.observe(&info, &conn_ctrl))
                        }
                        FuseAction::Reject => {
                            self.pending = None;
                            continue;
                        }
                    }
                }
                None => (None, None),
            };
            if let Some((conn, remote_addr)) = self.pending.take() {
                return Ok(Accepted {
                    coupler: TcpCoupler::new(),
                    stream: StraightStream::new(conn, fuse_config, conn_ctrl.clone(), observer),
                    fuse_config,
                    conn_ctrl,
                    local_addr: self.holdings[0].local_addr.clone(),
                    remote_addr,
                    http_scheme: Scheme::HTTP,
                });
            }
        }
    }
}
