use libp2p_core::{InboundUpgrade, OutboundUpgrade, UpgradeInfo};
use libp2p_swarm::NegotiatedSubstream;

use std::{io, iter, time::Duration};

pub const CHAT_MSG_HDR_SIZE: usize = 1 + 2;

#[derive(Default, Debug, Copy, Clone)]
pub struct Chat;

impl UpgradeInfo for Chat {
    type Info = &'static [u8];
    type InfoIter = iter::Once<Self::Info>;

    fn protocol_info(&self) -> Self::InfoIter {
        iter::once(b"/locha/chat/0.1.0")
    }
}

impl InboundUpgrade<NegotiatedSubstream> for Chat {
    type Output = NegotiatedSubstream;
    type Error = Void;
    type Future = future::Ready<Result<Self::Output, Self::Error>>;

    fn upgrade_inboud(self, stream: NegotiatedSubstream, _: Self::Info) -> Self::Future {
        future::ok(stream)
    }
}

impl OutboundUpgrade<NegotiatedSubstream> for Chat {
    type Output = NegotiatedSubstream;
    type Error = Void;
    type Future = future::Ready<Result<Self::Output, Self::Error>>;

    fn upgrade_outboud(self, stream: NegotiatedSubstream, _: Self::Info) -> Self::Future {
        future::ok(stream)
    }
}

pub async fn send_message<S, P>(mut stream: S, payload: P) -> io::Result<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
    P: AsRef<[u8]>
{
    let payload = payload.as_ref();
    let hdr = [0u8; CHAT_MSG_HDR_SIZE];
    hdr[0] = b'L';
    stream.write_all(payload.len()).await?;
    stream.write_all(payload.as_ref()).await?;
}