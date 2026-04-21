use std::thread;
use std::time::Duration;
use serde::Serialize;

#[derive(Serialize)]
struct SplashSignal {
    progress: Option<u32>,
    status: Option<String>,
    done: Option<bool>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let context = zmq::Context::new();
    let socket = context.socket(zmq::PUSH)?;
    
    // On Windows with libzmq, ipc:// addresses can be absolute paths.
    let mut pipe_path = std::env::temp_dir();
    pipe_path.push("fast_splash.ipc");
    let pipe_str = pipe_path.to_string_lossy().replace("\\", "/");
    let endpoint = format!("ipc://{}", pipe_str);

    println!("Connecting to {}...", endpoint);
    socket.connect(&endpoint)?;

    let steps = [
        (10, "Loading configuration..."),
        (30, "Initializing graphics..."),
        (50, "Connecting to database..."),
        (80, "Synchronizing data..."),
        (100, "Ready!"),
    ];

    for (prog, status) in steps {
        println!("Sending: {}% - {}", prog, status);
        let signal = SplashSignal {
            progress: Some(prog),
            status: Some(status.to_string()),
            done: Some(false),
        };
        
        let msg = serde_json::to_string(&signal)?;
        socket.send(&msg, 0)?;
        
        thread::sleep(Duration::from_millis(800));
    }

    println!("Sending done signal...");
    let done_signal = SplashSignal {
        progress: Some(100),
        status: Some("Starting application...".to_string()),
        done: Some(true),
    };
    let msg = serde_json::to_string(&done_signal)?;
    socket.send(&msg, 0)?;

    println!("Tester finished.");
    Ok(())
}
