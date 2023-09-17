use anyhow::Result;

use crate::cli::ServiceCommand;
use crate::AppContext;

impl AppContext {
    pub async fn run_service_command(&self, command: &ServiceCommand) -> Result<()> {
        Ok(())
    }
}
