use std::fs;
use std::io::Result;

use crate::agent::Agent;

impl Agent {
    pub fn clean_dead_agent_socket(&self) -> Result<()> {
        if self.is_running {
            println!(
                "{} at {} is running and the socket can't be removed",
                &self,
                &self.socket_path.display()
            );

            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "ssh-agent running",
            ));
        }

        fs::remove_file(&self.socket_path)?;
        let socket_dir = if let Some(dir) = &self.socket_path.parent() {
            dir
        } else {
            self.socket_path.as_path()
        };
        fs::remove_dir(socket_dir)?;

        Ok(())
    }
}
