// Copyright (c) 2019-2022 Naja Melan
// Copyright (c) 2023-2024 Yuki Kishimoto
// Distributed under the MIT software license

#![forbid(unsafe_code)]

use pharos::SharedPharos;
use wasm_bindgen_futures::spawn_local;

mod error;
mod event;
mod message;
mod meta;
mod state;
mod stream;
mod stream_io;

pub use self::error::WsErr;
pub use self::event::{CloseEvent, WsEvent};
pub use self::message::WsMessage;
pub use self::meta::WsMeta as WebSocket;
pub use self::state::WsState;
pub use self::stream::WsStream;
pub use self::stream_io::WsStreamIo;

/// Helper function to reduce code bloat
pub(crate) fn notify(pharos: SharedPharos<WsEvent>, evt: WsEvent) {
    let notify = async move {
        pharos
            .notify(evt)
            .await
            .map_err(|e| unreachable!("{:?}", e))
            .unwrap(); // only happens if we closed it.
    };

    spawn_local(notify);
}
