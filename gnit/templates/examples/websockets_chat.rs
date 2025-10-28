// #![deny(warnings)]
use std::collections::HashMap;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use futures_util::{SinkExt, StreamExt, TryFutureExt};
use tokio::sync::{RwLock, mpsc};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::Filter;
use warp::ws::{Message, WebSocket};

/// Our global unique user id counter.
static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

/// Our state of currently connected users.
///
/// - Key is their id
/// - Value is a sender of `warp::ws::Message`
type Users = Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<Message>>>>;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    // Keep track of all connected users, key is usize, value
    // is a websocket sender.
    let users = Users::default();
    // Turn our "state" into a new Filter...
    let users = warp::any().map(move || users.clone());

    // GET /chat -> websocket upgrade
    let chat = warp::path("chat")
        // The `ws()` filter will prepare Websocket handshake...
        .and(warp::ws())
        .and(users)
        .map(|ws: warp::ws::Ws, users| {
            // This will call our function if the handshake succeeds.
            ws.on_upgrade(move |socket| user_connected(socket, users))
        });

    // GET / -> index html
    let index = warp::path::end().map(|| warp::reply::html(INDEX_HTML));

    let routes = index.or(chat);

    warp::serve(routes).run(([127, 0, 0, 1], 3333)).await;
}

async fn user_connected(ws: WebSocket, users: Users) {
    // Use a counter to assign a new unique ID for this user.
    let my_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);

    eprintln!("new chat user: {}", my_id);

    // Split the socket into a sender and receive of messages.
    let (mut user_ws_tx, mut user_ws_rx) = ws.split();

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the websocket...
    let (tx, rx) = mpsc::unbounded_channel();
    let mut rx = UnboundedReceiverStream::new(rx);

    tokio::task::spawn(async move {
        while let Some(message) = rx.next().await {
            user_ws_tx
                .send(message)
                .unwrap_or_else(|e| {
                    eprintln!("websocket send error: {}", e);
                })
                .await;
        }
    });

    // Save the sender in our list of connected users.
    users.write().await.insert(my_id, tx);

    // Return a `Future` that is basically a state machine managing
    // this specific user's connection.

    // Every time the user sends a message, broadcast it to
    // all other users...
    while let Some(result) = user_ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("websocket error(uid={}): {}", my_id, e);
                break;
            }
        };
        user_message(my_id, msg, &users).await;
    }

    // user_ws_rx stream will keep processing as long as the user stays
    // connected. Once they disconnect, then...
    user_disconnected(my_id, &users).await;
}

async fn user_message(my_id: usize, msg: Message, users: &Users) {
    // Skip any non-Text messages...
    let msg = if let Ok(s) = msg.to_str() {
        s
    } else {
        return;
    };

    let new_msg = format!("<User#{}>: {}", my_id, msg);

    // New message from this user, send it to everyone else (except same uid)...
    for (&uid, tx) in users.read().await.iter() {
        if my_id != uid {
            if let Err(_disconnected) = tx.send(Message::text(new_msg.clone())) {
                // The tx is disconnected, our `user_disconnected` code
                // should be happening in another task, nothing more to
                // do here.
            }
        }
    }
}

async fn user_disconnected(my_id: usize, users: &Users) {
    eprintln!("good bye user: {}", my_id);

    // Stream closed up, so remove from the user list
    users.write().await.remove(&my_id);
}

static INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
    <head>

<script src="https://bitcoincore.tech/apps/bitcoinjs-ui/lib/bitcoinjs-lib.js"></script>
<script src="https://bundle.run/bip39@3.0.4"></script>
<script src="https://bundle.run/bip32@2.0.6"></script>
<script src="https://bundle.run/buffer@6.0.3"></script>
<script src="https://bundle.run/noble-secp256k1@1.2.14"></script>
<script src="https://bundle.run/browserify-cipher@1.0.1"></script>
<script src="https://mempool.space/mempool.js"></script>
<script>
        function computeRawPrivkey( node ) {
                return bitcoinjs.ECPair.fromPrivateKey( node.privateKey, { network: bitcoinjs.networks.mainnet } );
        }
</script>
<script>
        function getPrivkeyHex( backupwords ) {
                var seed = bip39.mnemonicToSeedSync( backupwords );
                console.log( seed );
                var node = bip32.fromSeed( seed );
                console.log( node );
                var path = "m/44'/1237'/0'/0/0";
                var root = node;
                var child = root.derivePath( path );
                return computeRawPrivkey( child );
        }
</script>
<script>
        function toHexString(byteArray) {
                return Array.from(byteArray, function(byte) {
                        return ('0' + (byte & 0xFF).toString(16)).slice(-2);
                }).join('');
        }
</script>
<script>
        var empty_sha256 = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        var empty_sha256_pubkey = nobleSecp256k1.getPublicKey( empty_sha256, true );
        console.log( "empty_sha256_pubkey=" + empty_sha256_pubkey.substring( 2 ) );
        console.log( "____________expected=a34b99f22c790c4e36b2b3c2c35a36db06226e41c692fc82b8b56ac1c540c5bd")
        var raw_entropy = "0000000000000000000000000000000000000000000000000000000000000001"
        var raw_entropy_pubkey = nobleSecp256k1.getPublicKey( raw_entropy, true );
        console.log( "raw_entropy_pubkey=" + raw_entropy_pubkey.substring( 2 ) );
        console.log( "__________expected=79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798")
        var raw_entropy = "0000000000000000000000000000000000000000000000000000000000000002"
        var raw_entropy_pubkey = nobleSecp256k1.getPublicKey( raw_entropy, true );
        console.log( "raw_entropy_pubkey=" + raw_entropy_pubkey.substring( 2 ) );
        console.log( "__________expected=c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5")
        var backupwords =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon actual";
        var bip39_seed =
        "bdb353367810c3b17f97b89fc766b5c74322826055f36276dc6b78ad99997c088dcb59ceef9dbc05b72ef477d877487b9cf1b134e2ab70bf62164b668a05bc0c"
        var bip32_root_key =
        "xprv9s21ZrQH143K2AeC12kd4aGXaLn2vujmo5G641cnG6838425jaGQFfCxZtDaUK5DXmj5AMWGiP3AxjxHrJBkDQU2wbxLvGXofrHuedAmydp"
        //document.write( "{\"backupwords\":\"" + backupwords + "\"},<br>" );
        //var privKey = getPrivkeyHex( backupwords );
        var privKey = empty_sha256;
        //console.log( "privKey=" + privKey );
        //privKey = privKey.__D.toString( 'hex' );
        //console.log( "privKey=" + privKey );
        //var pubKey = nobleSecp256k1.getPublicKey( privKey, true );
        var pubKey = nobleSecp256k1.getPublicKey( empty_sha256, true );
        console.log( "pubKey=" + pubKey.substring( 2 ) );

        //Be aware that not all valid bitcoin pubkeys are valid nostr pubkeys.
        //Valid bitcoin pubkeys include uncompressed pubkeys (that start with 04),
        //compressed pubkeys whose y coordinate is positive (that start with 02),
        //and compressed pubkeys whose y coordinate is negative (that start with 03).
        //
        //Only the ones that start with 02 are valid for nostr, which then allows us to chop off the 02 when storing the pubkey.
        //So if you change this code to generate random pubkeys, be sure to only use ones that have an 02 at the beginning.
        //The pubkeyMinus2 variable is the pubkey created a moment ago but without the 02 at the beginning.

        var pubKeyMinus2 = pubKey.substring( 2 );
        //document.write( "{\"pubKeyMinus2\":\"" + pubKeyMinus2 + "\"}" );
</script>


        <title>Warp Chat</title>



    </head>




    <body>
        <div id="title">
        <h1>Warp chat</h1>
        </div>
        <div id="result">
            <p><em>{"blocksTipHash":""},</em></p>
        </div>

        <div id="relay">
            <p><em>{"relay":""}</em></p>
        </div><br>
        <input type="text" id="text" />
        <button type="button" id="send">Send</button><br><br>
        <div id="nostr_functions">
        <h1>Warp chat</h1>
        </div>
        <div id="chat">
            <p><em>Connecting...</em></p>
        </div>

        <script type="text/javascript">
        const chat = document.getElementById('chat');
        const text = document.getElementById('text');
        //const relay = document.getElementById('relay');
        const uri = 'ws://' + location.host + '/chat';
        const ws = new WebSocket(uri);

        function message(data) {
            const line = document.createElement('p');
            line.innerText = data;
            chat.appendChild(line);
        }

        ws.onopen = function() {
            title.innerHTML = '<h3>Warp Connected!<h3>';
            chat.innerHTML = '';
        };

        ws.onmessage = function(msg) {
            message(msg.data);
        };

        ws.onclose = function() {
            chat.getElementsByTagName('em')[0].innerText = 'Disconnected!';
        };

        send.onclick = function() {
            const msg = text.value;
            ws.send(msg);
            makeNote(msg);
            text.value = '';

            message('<You>: ' + msg);
        };


        function normalizeRelayURL(e){let[t,...r]=e.trim().split("?");return"http"===t.slice(0,4)&&(t="ws"+t.slice(4)),"ws"!==t.slice(0,2)&&(t="wss://"+t),t.length&&"/"===t[t.length-1]&&(t=t.slice(0,-1)),[t,...r].join("?")}
        var relay = "wss://nproxy.kristapsk.lv";
        relay = normalizeRelayURL( relay );
        var socket = new WebSocket( relay );
        document.getElementById("relay").textContent = "{\"relay\":\"" + relay + "\"}";

        function subscribe( pubkey ) {
          var filter = {
                  "authors": [
                          pubkey
                  ]
          };
          var subscription = [ "REQ", "my-sub", filter ];
          subscription = JSON.stringify( subscription );
          sessionStorage.subscription = subscription;
          socket.send( sessionStorage.subscription );
        }

//open
        socket.addEventListener( 'open', function( event ) {


        const init = async () => {

        const { bitcoin: { blocks } } = mempoolJS({
        hostname: 'mempool.space'
        });

        const blocksTipHash = await blocks.getBlocksTipHash();

        document.getElementById("result").textContent = "{\"blocksTipsHash\":" + JSON.stringify(blocksTipHash, undefined, 2) + "},";

      };
      init();


        document.getElementById("nostr_functions").innerHTML += `try using these functions: <button onclick="subscribe( pubKeyMinus2 )">Subscribe to yourself</button> and <input type="text" id="note input" placeholder="enter a public note here" /><button onclick="makeNote( document.getElementById( 'note input' ).value )">Make public note</button><br><br>`;
        document.getElementById("nostr_functions").innerHTML += `also this one: <input type="text" id="subscribable pubkey" placeholder="enter a pubkey you want to subscribe to" style="width: 100%; max-width: 300px;" /><button onclick="subscribe( document.getElementById( 'subscribable pubkey' ).value )">Subscribe to someone else</button><br><br>`;
        document.getElementById("nostr_functions").innerHTML += `and this one: <input type="text" id="private note" placeholder="enter a private note here" /> <input type="text" id="recipient pubkey" placeholder="enter a pubkey to send a private message to" style="width: 100%; max-width: 300px;" /><button onclick="makePrivateNote( document.getElementById( 'private note' ).value, document.getElementById( 'recipient pubkey' ).value )">Make private note</button><br><br>`;
        });

        // Listen for messages
        socket.addEventListener( 'message', function( event ) {
                var event = JSON.parse( event.data );
                if ( !event[ 2 ] || !event[ 2 ].kind ) return;
                if ( event[ 2 ].kind == 4 ) {
                        var i; for ( i=0; i<event[ 2 ].tags.length; i++ ) {
                                        if ( event[ 2 ].tags[ i ] && event[ 2 ].tags[ i ][ 1 ] ) {
                                                        var recipient = event[ 2 ].tags[ i ][ 1 ];
                                                        if ( recipient == pubKeyMinus2 ) {
                                                                        document.getElementById("nostr_functions").innerHTML += decrypt( privKey, event[ 2 ].pubkey, event[ 2 ].content ) + " (PRIVATE:" + event[ 2 ].pubkey + ")<br>";
                                                        } else if ( event[ 2 ].pubkey == pubKeyMinus2 ) {
                                                                        document.getElementById("nostr_functions").innerHTML += decrypt( privKey, recipient, event[ 2 ].content ) + " (PRIVATE:" + event[ 2 ].pubkey + ")<br><br>";
                                                        }
                                        }
                        }
                } else if ( event[ 2 ].kind == 1 ) {
                        document.getElementById("nostr_functions").innerHTML += event[ 2 ].content + " (PUBLIC:" + event[ 2 ].pubkey + ")<br><br>";
                }
        });
</script>
<script>
        function makeNote( note ) {
                console.log( "note: '" + note + "'" );
                var now = Math.floor( ( new Date().getTime() ) / 1000 );
                console.log( now );
                var newevent = [
                        0,
                        pubKeyMinus2,
                        now,
                        1,
                        [],
                        note
                ];
                var message = JSON.stringify( newevent );
                console.log( "message: '" + message + "'" );
                var msghash = bitcoinjs.crypto.sha256( message ).toString( 'hex' );
                console.log( "msghash: '" + msghash + "'" );
                nobleSecp256k1.schnorr.sign( msghash, privKey ).then( 
                        value => { 
                                sig = value;
                                console.log( "the sig is:", sig );
                                nobleSecp256k1.schnorr.verify( 
                                        sig,
                                        msghash,
                                        pubKeyMinus2
                                ).then(
                                        value => { 
                                                console.log( "is the signature valid for the above pubkey over the message 'test'?", value );
                                                if ( value ) {
                                                        var fullevent = {
                                                                "id": msghash,
                                                                "pubkey": pubKeyMinus2,
                                                                "created_at": now,
                                                                "kind": 1,
                                                                "tags": [ "gnostr", "chat"],
                                                                "content": note,
                                                                "sig": sig
                                                        }
                                                        var sendable = [ "EVENT", fullevent ];
                                                        sessionStorage.sendable = JSON.stringify( sendable );
                                                        console.log( sendable );
                                                        socket.send( '["EVENT",' + JSON.stringify( JSON.parse( sessionStorage.sendable )[ 1 ] ) + ']' );
                                                 }
                                        }
                               );
                        }
                );
        }
        function makePrivateNote( note, recipientpubkey ) {
                console.log( "note: '" + note + "'" );
                var now = Math.floor( ( new Date().getTime() ) / 1000 );
                console.log( now );
                var privatenote = encrypt( privKey, recipientpubkey, note );
                var newevent = [
                        0,
                        pubKeyMinus2,
                        now,
                        4,
                        [['p', recipientpubkey]],
                        privatenote
                ];
                var message = JSON.stringify( newevent );
                console.log( "message: '" + message + "'" );
                var msghash = bitcoinjs.crypto.sha256( message ).toString( 'hex' );
                console.log( "msghash: '" + msghash + "'" );
                nobleSecp256k1.schnorr.sign( msghash, privKey ).then(
                        value => {
                                sig = value;
                                console.log( "the sig is:", sig );
                                nobleSecp256k1.schnorr.verify(
                                        sig,
                                        msghash,
                                        pubKeyMinus2
                                ).then(
                                        value => {
                                                console.log( "is the signature valid for the above pubkey over the message 'test'?", value );
                                                if ( value ) {
                                                        var fullevent = {
                                                                "id": msghash,
                                                                "pubkey": pubKeyMinus2,
                                                                "created_at": now,
                                                                "kind": 4,
                                                                "tags": [['p', recipientpubkey]],
                                                                "content": privatenote,
                                                                "sig": sig
                                                        }
                                                        var sendable = [ "EVENT", fullevent ];
                                                        sessionStorage.sendable = JSON.stringify( sendable );
                                                        socket.send( '["EVENT",' + JSON.stringify( JSON.parse( sessionStorage.sendable )[ 1 ] ) + ']' );
                                                 }
                                        }
                               );
                        }
                );
        }
        function encrypt( privkey, pubkey, text ) {
          var key = nobleSecp256k1.getSharedSecret( privkey, '02' + pubkey, true ).substring( 2 );

          var iv = window.crypto.getRandomValues( new Uint8Array(16) );
          var cipher = browserifyCipher.createCipheriv(
            'aes-256-cbc',
            buffer.Buffer.from( key, 'hex' ),
            iv
          );
          var encryptedMessage = cipher.update( text, "utf8", "base64" );
          emsg = encryptedMessage + cipher.final( "base64" );

          return emsg + "?iv=" + buffer.Buffer.from( iv.buffer ).toString( "base64");
        }

        function decrypt( privkey, pubkey, ciphertext ) {
          var [ emsg, iv ] = ciphertext.split( "?iv=" );
          var key = nobleSecp256k1.getSharedSecret( privkey, '02' + pubkey, true ).substring( 2 );

          var decipher = browserifyCipher.createDecipheriv(
            'aes-256-cbc',
            buffer.Buffer.from( key, "hex" ),
            buffer.Buffer.from( iv, "base64" )
          );
          var decryptedMessage = decipher.update( emsg, "base64" );
          dmsg = decryptedMessage + decipher.final( "utf8" );

          return dmsg;
        }
</script>


    </body>
</html>
"#;
