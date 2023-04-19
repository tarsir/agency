use std::fs;
use std::io;
use std::path::{PathBuf};

#[derive(Debug, Default, Clone)]
struct Agent {
    pub pid: String,
    pub socket_path: PathBuf,
}

fn main() -> io::Result<()> {
    let agents: Vec<Agent> = get_current_agents()?;
    println!("{:?}", agents);
    Ok(())
}

fn get_current_agents() -> io::Result<Vec<Agent>> {
    let agent_dirs: Vec<Agent> =
        fs::read_dir("/tmp")?
            .filter_map(|res| {
                match res {
                    Ok(res) => {
                        if res.path().to_str().unwrap_or_default().starts_with("/tmp/ssh-") {
                            Some(res)
                        } else {
                            None
                        }
                    },
                    Err(_) => None
                }
            })
            .map(|dir| -> io::Result<Agent> {
                let socket: Option<fs::DirEntry> = fs::read_dir(dir.path())?.next().map(|f| {
                    if let Ok(f) = f {
                        Some(f)
                    } else {
                        None
                    }
                }).flatten();
                if let Some(socket) = socket {
                    let pid = socket.file_name().into_string();
                    let pid = if let Ok(pid) = pid {
                        pid.split(".").nth(1).unwrap_or_else(|| "N/A").to_string()
                    } else {
                        "N/A".to_string()
                    };
                    Ok(Agent {
                        pid,
                        socket_path: socket.path(),
                    })
                } else {
                    Err(io::Error::new(io::ErrorKind::Other, "argh"))
                }
            })
            .filter_map(|a| a.ok())
            .collect();

    Ok(agent_dirs)
}
