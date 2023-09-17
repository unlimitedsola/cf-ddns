use anyhow::Result;

use crate::AppContext;
use crate::cli::ServiceCommand;

impl AppContext {
    pub async fn run_service_command(&self, command: &ServiceCommand) -> Result<()> {
        Ok(())
    }
}
