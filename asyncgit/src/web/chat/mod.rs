#![allow(unused)]
use super::images;
use super::js;
use crate::web::css;
use crate::web::template_html;
use crate::web::websock_index_html;

/// TEMPLATE_HTML
pub static TEMPLATE_HTML: &str = r#"<!DOCTYPE html>
<html lang=\"en\">
	<head>
		<meta charset=\"utf-8\">
		<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">
		<meta name=\"theme-color\" content=\"\#0f0f0f\"/>
		        <meta http-equiv=\"Content-Security-Policy\"
		            content=\"default-src 'none'; manifest-src 'self'; connect-src 'self' ws: wss:; script-src 'self'; script-src-elem 'self'; script-src-attr 'unsafe-hashes' 'sha256-Td3Y/ET9puc5SaGYiJIrX89xKCA2VzXvfyS6pEAPuUM='; style-src 'self' fonts.googleapis.com; img-src http: https: data:; media-src *; font-src 'self' fonts.gstatic.com; child-src 'none';\" />		<title>gnostr<h1>Hello 🅖!</h1>¬</title>
		<link rel=\"manifest\" href=\"/pwa/manifest.json\"/>
		<link rel=\"images\" href=\"/images/icon.svg\" type=\"image/svg+xml\"/>
		<link rel=\"apple-touch-icon\" href=\"/pwa/icon-256.png\"/>
		<link rel=\"stylesheet\" href=\"/css/vars.css?v=1\">
		<link rel=\"stylesheet\" href=\"/css/utils.css?v=1\">
		<link rel=\"stylesheet\" href=\"/css/styles.css?v=13\">
		<link rel=\"stylesheet\" href=\"/css/responsive.css?v=10\">
		<script defer src=\"/js/util.js?v=5\"></script>
		<script defer src=\"/js/ui/safe-html.js?v=1\"></script>
		<script defer src=\"/js/ui/util.js?v=8\"></script>
		<script defer src=\"/js/ui/render.js?v=15\"></script>
		<script defer src=\"/js/ui/state.js?v=1\"></script>
		<script defer src=\"/js/ui/fmt.js?v=1\"></script>
		<script defer src=\"/js/ui/profile.js?v=1\"></script>
		<script defer src=\"/js/ui/settings.js?v=1\"></script>
		<script defer src=\"/js/ui/dm.js?v=1\"></script>
		<script defer src=\"/js/nostr.js?v=7\"></script>
		<script defer src=\"/js/core.js?v=1\"></script>
		<script defer src=\"/js/model.js?v=1\"></script>
		<script defer src=\"/js/contacts.js?v=1\"></script>
		<script defer src=\"/js/event.js?v=1\"></script>
		        <script defer src=\"/js/lib.js?v=1\"></script>
				<script defer src=\"/js/main.js?v=1\"></script>
		        <script defer src=\"/js/db.js?v=1\"></script>
			</head>	<body>
		<div id=\"container-busy\">
			<div class=\"loader\" title=\"Loading...\">
				<img class=\"dark-invert\" src=\"/images/loader-fragment.svg\"/>
			</div>
		</div>
		<div id=\"container-welcome\" class=\"hide\">
			<div class=\"hero-box\">
				<div class=\"padded\">
					<h1>
						gnostr<h1>Hello 🅖!</h1>¬
						<img class=\"icon svg\" src=\"/images/logo-inverted.svg\"/>
					</h1>
					<p>A minimal experience for Nostr.</p>
					<p>Please access with a nos2x compatible browser.</p>
				</div>
			</div>
		</div>

		<div id=\"container-app\" class=\"hide\">

		<div id=\"container\">
			<div class=\"flex-fill vertical-hide\"></div>
			<nav id=\"nav\" class=\"nav full flex-noshrink vertical-hide\">
				<div>
					<button action=\"open-view\" data-view=\"nip34-global\" class=\"nav icon\"
						title=\"gnostr.org\">
						<img class=\"icon svg inactive\" src=\"/images/logo-inverted.svg\"/>
						<img class=\"icon svg active\" src=\"/images/logo.svg\"/>
					</button>
					<button action=\"open-view\" data-view=\"friends\" class=\"nav icon\"
						title=\"Home\">
						<img class=\"icon svg inactive\" src=\"/images/home.svg\"/>
						<img class=\"icon svg active\" src=\"/images/home-active.svg\"/>
					</button>
					<button action=\"open-view\" data-view=\"dm\" class=\"nav icon\"
						title=\"Direct Messages\">
						<img class=\"icon svg inactive\" src=\"/images/messages.svg\"/>
						<img class=\"icon svg active\" src=\"/images/messages-active.svg\"/>
						<div class=\"new-notifications hide\" role=\"dm\"></div>
					</button>
					<button action=\"open-view\" data-view=\"notifications\"
						class=\"nav icon\" title=\"Notifications\">
						<img class=\"icon svg inactive\" src=\"/images/notifications.svg\"/>
						<img class=\"icon svg active\" src=\"/images/notifications-active.svg\"/>
						<div class=\"new-notifications hide\" role=\"activity\"></div>
					</button>
					<button action=\"open-view\" data-view=\"settings\"
						title=\"Settings\" class=\"nav icon\">
						<img class=\"icon svg inactive\" src=\"/images/settings.svg\"/>
						<img class=\"icon svg active\" src=\"/images/settings-active.svg\"/>
					</button>
					<button action=\"new-note\" title=\"New Note\" class=\"nav icon new-note\">
						<img class=\"icon svg inactive\" src=\"/images/new-note.svg\"/>
						<img class=\"icon svg active\" src=\"/images/new-note.svg\"/>
					</button>
				</div>
			</nav>

			<div id=\"view\">
					<header>
						<label>Home</label>
						<div id=\"header-tools\">
							<button class=\"action small hide\"
							disabled action=\"mark-all-read\">
								Mark All Read
							</button>
							<img class=\"pfp hide\" role=\"their-pfp\" data-pubkey=\"\"
							src=\"/images/no-user.svg\"/>
							<img class=\"pfp hide\" role=\"my-pfp\" data-pubkey=\"\"
							src=\"/images/no-user.svg\"/>
						</div>
					</header>
					<div id=\"profile-info\" role=\"profile-info\" class=\"bottom-border hide\">
						<div class=\"profile-banner\" name=\"profile-banner\"></div>
						<div class=\"flex\">
							<img name=\"profile-image\" class=\"pfp jumbo hide\"/>
							<label name=\"profile-nip05\"></label>
							<div class=\"profile-tools\">
								<button class=\"icon link hide\"
								name=\"profile-website\" action=\"open-link\">
									<img class=\"icon svg\" src=\"/images/profile-website.svg\"/>
								</button>
								<button class=\"icon link hide\"
								title=\"Copy Lightning Address\"
								name=\"profile-lud06\" action=\"open-lud06\">
									<img class=\"icon svg\" src=\"/images/profile-zap.svg\"/>
								</button>
								<button class=\"icon\" name=\"message-user\"
								title=\"Directly Message\">
									<img class=\"icon svg\" src=\"/images/message-user.svg\"/>
								</button>
								<button class=\"icon\" name=\"copy-pk\"
								data-pk=\"\" title=\"Copy Public Key\">
									<img class=\"icon svg\" src=\"/images/pubkey.svg\"/></button>

								<button class=\"action\" name=\"follow-user\"
								data-pk=\"\">Follow</button>
								<button class=\"action\" name=\"edit-profile\"
								title=\"Update Profile\">
									Update
								</button>
							</div>
						</div>
						<div>
							<p name=\"profile-about\"></p>
						</div>
					</div>
					                    <div id=\"dms\" class=\"hide\">
										</div>
										<div id=\"show-new\" class=\"show-new hide\" action=\"show-timeline-new\">
											<button>Show <span>0</span> new notes</button>
										</div>
										<div id=\"timeline\" class=\"events\"></div>
										<div id=\"show-more\" class=\"show-more\">
											<button action=\"show-timeline-more\">Show More</button>
					                        <button action=\"show-nip34-more\" class=\"hide\">Show More NIP-34</button>
										</div>
					                    <div class=\"loading-events\">
											<div class=\"loader\" title=\"Loading...\">
												<img class=\"dark-invert\" src=\"/images/loader-fragment.svg\"/>
											</div>
										</div>
										<div id=\"settings\" class=\"hide\">						<section>
							<header>
								<label>Relays</label>
								<button id=\"add-relay\" class=\"btn-text\">
									<img class=\"svg icon small\" src=\"/images/add-relay.svg\"/>
								</button>
							</header>
							<table id=\"relay-list\" class=\"row\">
								<thead>
									<tr>
										<td>Address</td>
										<td>Remove</td>
									</tr>
								</thead>
								<tbody>
								</tbody>
							</table>
						</section>
						<section>
							<header><label>Info</label></header>
							<p>
							<a href=\"https://github.com/gnostr-org/gnostr\">Source Code</a>
							<a href=\"https://github.com/gnostr-org/gnostr/issues\">Bug Tracker</a>
							<a href=\"mailto:admin@gnostr.org\">Email Me</a>
							</p>
						</section>
					</div>
					<footer>
						<div id=\"dm-post\" class=\"hide\">
							<textarea class=\"post-input dm\" name=\"message\"></textarea>
							<div class=\"post-tools\">
								<button name=\"send-dm\" class=\"action\">Send</button>
							</div>
						</div>
						<nav class=\"nav mobile\">
							<button action=\"open-view\" data-view=\"friends\" class=\"icon\"
								title=\"Home\">
								<img class=\"icon svg inactive\" src=\"/images/home.svg\"/>
								<img class=\"icon svg active\" src=\"/images/home-active.svg\"/>
							</button>
							<button action=\"open-view\" data-view=\"dm\" class=\"icon\"
								title=\"Direct Messages\">
								<img class=\"icon svg inactive\" src=\"/images/messages.svg\"/>
								<img class=\"icon svg active\" src=\"/images/messages-active.svg\"/>
								<div class=\"new-notifications hide\" role=\"dm\"></div>
							</button>
							<button action=\"open-view\" data-view=\"notifications\"
								class=\"icon\" title=\"Notifications\">
								<img class=\"icon svg inactive\" src=\"/images/notifications.svg\"/>
								<img class=\"icon svg active\" src=\"/images/notifications-active.svg\"/>
								<div class=\"new-notifications hide\" role=\"activity\"></div>
							</button>
							<button action=\"open-view\" data-view=\"settings\"
								title=\"Settings\" class=\"icon\">
								<img class=\"icon svg inactive\" src=\"/images/settings.svg\"/>
								<img class=\"icon svg active\" src=\"/images/settings-active.svg\"/>
							</button>
							<button id=\"new-note-mobile\" action=\"new-note\"
								title=\"New Note\" class=\"nav icon new-note\">
								<img class=\"icon svg inactive\" src=\"/images/new-note.svg\"/>
								<img class=\"icon svg active\" src=\"/images/new-note.svg\"/>
							</button>
						</nav>
					</footer>
			</div>
			<div class=\"flex-fill vertical-hide\"></div>
		</div>
		</div>
        <div id=\"nip34-cache-size\" style=\"position: fixed; bottom: 0; width: 100%; text-align: right; background-color: #333; color: white; padding: 5px; font-size: 0.8em;\">NIP-34 Cache Size: Calculating...</div>

		<dialog id=\"media-preview\" action=\"close-media\">
			<img action=\"close-media\" src=\"\"/>
			<!-- TODO add loader to media preview -->
		</dialog>
		<dialog id=\"reply-modal\">
			<div class=\"container\">
				<header>
					<label>Reply To</label>
					<button class=\"icon\" action=\"close-modal\">
						<img class=\"icon svg\" src=\"/images/close-modal.svg\"/>
					</button>
				</header>
				<div id=\"replying-to\"></div>
				<div id=\"replybox\">
					<textarea id=\"reply-content\" class=\"post-input\"
						placeholder=\"Reply...\"></textarea>
					<div class=\"post-tools new\">
						<button class=\"action\" name=\"send\">Send</button>
					</div>
					<div class=\"post-tools reply\">
						<button class=\"action\" name=\"reply-all\" data-all=\"1\">Reply All</button>
						<button class=\"action\" name=\"reply\">Reply</button>
					</div>
				</div>
			</div>
		</dialog>
		<dialog id=\"profile-editor\">
			<div class=\"container\">
				<header>
					<label>Update Profile</label>
					<button class=\"icon\" action=\"close-modal\">
						<img class=\"icon svg\" src=\"/images/close-modal.svg\"/>
					</button>
				</header>
				<div>
					<input type=\"text\" class=\"block w100\" name=\"name\" placeholder=\"Name\"/>
					<input type=\"text\" class=\"block w100\" name=\"display_name\" placeholder=\"Display Name\"/>
					<input type=\"text\" class=\"block w100\" name=\"picture\" placeholder=\"Picture URL\"/>
					<input type=\"text\" class=\"block w100\" name=\"banner\" placeholder=\"Banner URL\"/>
					<input type=\"text\" class=\"block w100\" name=\"website\" placeholder=\"Website\"/>
					<input type=\"text\" class=\"block w100\" name=\"lud06\" placeholder=\"lud06\"/>
					<input type=\"text\" class=\"block w100\" name=\"nip05\" placeholder=\"nip05\"/>
					<textarea name=\"about\" class=\"block w100\" placeholder=\"A bit about you.\"></textarea>
					<button class=\"action float-right\" action=\"open-profile-editor\">
						Update
					</button>
				</div>
			</div>
		</dialog>
		<dialog id=\"event-details\">
			<div class=\"container\">
				<header>
					<label>Event Details</label>
					<button class=\"icon modal-floating-close-btn\" action=\"close-modal\">
						<img class=\"icon svg\" src=\"/images/close-modal.svg\"/>
					</button>
				</header>
				<div class=\"max-content\">
					<pre><code></code></pre>
				</div>
			</div>
		</dialog>

		</div>
	</body>
</html>
"#;

pub static _WEBSOCKET_CHAT_INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
    <head>
   <link rel="icon" href="/images/favicon.ico" type="image/x-icon">
    <!-- Or for older browsers, you might see: -->
    <!-- <link rel="shortcut icon" href="/images/favicon.ico" type="image/x-icon"> -->
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
        document.write( "{\"backupwords\":\"" + backupwords + "\"},<br>" );
        //var privKey = getPrivkeyHex( backupwords );
        var privKey = empty_sha256;
        console.log( "privKey=" + privKey );
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
        document.write( "{\"pubKeyMinus2\":\"" + pubKeyMinus2 + "\"}" );
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
            chat.innerHTML = 'Disconnected!';
            <!-- chat.getElementsByTagName('p')[0].innerText = 'Disconnected!'; -->
            <!-- chat.getElementsByTagName('em')[0].innerText = 'Disconnected!'; -->
        };

        send.onclick = function() {
            const msg = text.value;
            ws.send(msg);
            makeNote(msg);
            text.value = '';

            message('<You>: ' + msg);
        };


        <!-- normalizeRelay -->
        function normalizeRelayURL(e){let[t,...r]=e.trim().split("?");return"http"===t.slice(0,4)&&(t="ws"+t.slice(4)),"ws"!==t.slice(0,2)&&(t="wss://"+t),t.length&&"/"===t[t.length-1]&&(t=t.slice(0,-1)),[t,...r].join("?")}


        <!-- var relay = "wss://nproxy.kristapsk.lv"; -->
        var relay = "ws://127.0.0.1:8080";
        relay = normalizeRelayURL( relay );
        var socket = new WebSocket( relay );
        document.getElementById("relay").textContent = "{\"relay\":\"" + relay + "\"}";


        <!-- subscribe -->
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

        <!-- socket.addEventListener -->
        socket.addEventListener( 'open', function( event ) {


        <!-- const init -->
        const init = async () => {

        const { bitcoin: { blocks } } = mempoolJS({
        hostname: 'mempool.space'
        });

        const blocksTipHash = await blocks.getBlocksTipHash();

        document.getElementById("result").textContent = "{\"blocksTipsHash\":" + JSON.stringify(blocksTipHash, undefined, 2) + "},";

        };
        init();

        <!-- document.getElementById("nostr_functions").innerHTML -->
        document.getElementById("nostr_functions").innerHTML += `try using these functions: <button onclick="subscribe( pubKeyMinus2 )">Subscribe to yourself</button> and <input type="text" id="note input" placeholder="enter a public note here" /><button onclick="makeNote( document.getElementById( 'note input' ).value )">Make public note</button><br><br>`;

        <!-- document.getElementById("nostr_functions").innerHTML -->
        document.getElementById("nostr_functions").innerHTML += `also this one: <input type="text" id="subscribable pubkey" placeholder="enter a pubkey you want to subscribe to" style="width: 100%; max-width: 300px;" /><button onclick="subscribe( document.getElementById( 'subscribable pubkey' ).value )">Subscribe to someone else</button><br><br>`;

        <!-- document.getElementById("nostr_functions").innerHTML -->
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
