use crate::remote::message_stream::{MessageStream, TransitionToRead};
use crate::remote::messages;
use crate::remote::messages::*;
use crate::remote::options::*;
use anyhow::*;
//https://crates.io/crates/bincode/1.3.1
//bincode
use core::result::Result::Ok;
use log::{debug, error, info, trace};
use std::{
    fs::File,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    process::{Child, Command, Stdio},
    sync::mpsc::{Receiver, Sender, channel},
    thread,
};

#[cfg(not(target_os = "windows"))]
use std::os::unix::fs::PermissionsExt;

type IoOut = Receiver<Vec<u8>>;

#[derive(Default)]
struct Context {
    /// Used for tracking running executable.
    stdout: Option<IoOut>,
    /// Used for tracking running executable.
    stderr: Option<IoOut>,
    /// Used for tracking running executable.
    proc: Option<Child>,
}

impl Context {
    /// Handles incoming messages and sends back reply (if needed) if returns false it means we
    /// should exit the update
    pub fn handle_incoming_msg<S: Write + Read>(
        &mut self,
        msg_stream: &mut MessageStream,
        stream: &mut S,
        message: Messages,
    ) -> Result<bool> {
        match message {
            Messages::HandshakeRequest => {
                let msg: HandshakeRequest = bincode::deserialize(&msg_stream.data)?;

                if msg.version_major != messages::REMOTELINK_MAJOR_VERSION {
                    return Err(anyhow!(
                        "Major version miss-match (target {} host {})",
                        messages::REMOTELINK_MAJOR_VERSION,
                        msg.version_major
                    ));
                }

                if msg.version_minor != messages::REMOTELINK_MINOR_VERSION {
                    debug!("Minor version miss-matching, but continuing");
                }

                let handshake_reply = HandshakeReply {
                    version_major: messages::REMOTELINK_MAJOR_VERSION,
                    version_minor: messages::REMOTELINK_MINOR_VERSION,
                };

                msg_stream.begin_write_message(
                    stream,
                    &handshake_reply,
                    Messages::HandshakeReply,
                    TransitionToRead::Yes,
                )?;
            }

            Messages::StopExecutableRequest => {
                trace!("StopExecutableRequest");

                if let Some(proc) = self.proc.as_mut() {
                    proc.kill()?;
                }

                let stop_reply = StopExecutableReply::default();

                msg_stream.begin_write_message(
                    stream,
                    &stop_reply,
                    Messages::StopExecutableReply,
                    TransitionToRead::Yes,
                )?;

                return Ok(false);
            }

            Messages::LaunchExecutableRequest => {
                trace!("LaunchExecutableRequest");

                let file: bincode::Result<messages::LaunchExecutableRequest> =
                    bincode::deserialize(&msg_stream.data);

                match file {
                    Ok(f) => {
                        self.start_executable(&f);

                        let exe_launch = LaunchExecutableReply {
                            launch_status: 0,
                            error_info: None,
                        };

                        msg_stream.begin_write_message(
                            stream,
                            &exe_launch,
                            Messages::LaunchExecutableReply,
                            TransitionToRead::Yes,
                        )?;
                    }

                    Err(e) => {
                        panic!("{}", e);
                    }
                }
            }

            _ => {
                // if we didn't handle the message switch over to waiting for new data
                dbg!(message);
            }
        }

        Ok(true)
    }

    /// Pipe streams are blocking, we need separate threads to monitor them without blocking the primary thread.
    fn child_stream_to_vec<R>(mut stream: R, out: Sender<Vec<u8>>)
    where
        R: Read + Send + 'static,
    {
        thread::Builder::new()
            .name("child_stream_to_vec".into())
            .spawn(move || {
                loop {
                    let mut buf = [0u8; 2];
                    match stream.read(&mut buf) {
                        Err(err) => {
                            error!("{}] Error reading from stream: {}", line!(), err);
                            break;
                        }
                        Ok(got) => {
                            if got == 0 {
                                break;
                            }

                            let mut vec = Vec::with_capacity(got);
                            vec.extend_from_slice(&buf[..got]);
                            // TODO: Fix this
                            let _ = out.send(vec);
                        }
                    }
                }
            })
            .expect("!thread");
    }

    #[cfg(unix)]
    fn set_executable_permissions(path: &str) {
        use std::fs;
        use std::os::unix::fs::PermissionsExt; // Import the Unix-specific extension trait

        debug!("Setting Unix-like permissions for '{}'", path);
        if let Err(e) = fs::set_permissions(path, fs::Permissions::from_mode(0o700)) {
            eprintln!("Error setting Unix permissions: {}", e);
        }
    }

    #[cfg(windows)]
    fn set_executable_permissions(path: &str) {
        debug!(
            "Skipping explicit permission setting on Windows for '{}'.",
            path
        );
        // On Windows, you typically rely on the file extension (.exe) for executability
        // and that the user running it has default "Execute" ACL permissions.
        // If you need fine-grained ACL control, you'd need external crates or FFI to Windows API.
    }

    fn start_executable(&mut self, f: &messages::LaunchExecutableRequest) {
        trace!("Want to launch {} size {}", f.path, f.data.len());

        {
            let mut file = File::create("executable").unwrap();
            file.write_all(f.data).unwrap();
        }

        // make exe executable
        Self::set_executable_permissions("executable");

        let mut p = Command::new("./executable")
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to execute child");

        let (stdout_tx, stdout_rx) = channel();
        let (stderr_tx, stderr_rx) = channel();

        Self::child_stream_to_vec(p.stdout.take().expect("!stdout"), stdout_tx);
        Self::child_stream_to_vec(p.stderr.take().expect("!stderr"), stderr_tx);

        self.stdout = Some(stdout_rx);
        self.stderr = Some(stderr_rx);
        self.proc = Some(p);
    }
}

fn handle_client(stream: &mut TcpStream) -> Result<()> {
    info!("Incoming connection from: {}", stream.peer_addr()?);

    stream.set_nonblocking(true)?;

    let mut msg_stream = MessageStream::new();

    msg_stream.begin_read(stream, false)?;

    // Setup a context so we can keep track of a running process and such
    let mut context = Context::default();

    loop {
        let msg = msg_stream.update(stream)?;

        if let Some(msg) = msg {
            if !context.handle_incoming_msg(&mut msg_stream, stream, msg)? {
                info!("exit client");
                return Ok(());
            }
        }

        if let Some(stdout) = context.stdout.as_mut() {
            if let Ok(data) = stdout.try_recv() {
                if !data.is_empty() {
                    let text_message = TextMessage { data: &data };

                    msg_stream.begin_write_message(
                        stream,
                        &text_message,
                        Messages::StdoutOutput,
                        TransitionToRead::Yes,
                    )?;
                }
            }
        } else {
            // If there isn't much going on we sleep for 1 ms to not hammer the CPU
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
}

pub fn update(_opts: &Opt) {
    let listener = TcpListener::bind("0.0.0.0:8888").expect("Could not bind");
    info!("Waiting on incoming host");
    for stream in listener.incoming() {
        match stream {
            Err(e) => error!("failed: {}", e),
            Ok(mut stream) => {
                thread::spawn(move || {
                    handle_client(&mut stream).unwrap_or_else(|error| error!("{:?}", error));
                });
            }
        }
    }
}
