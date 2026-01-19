use rig::completion::ToolDefinition;
use rig::tool::{ToolDyn, ToolError};
use rmcp::model::{ClientCapabilities, ClientInfo, Implementation, InitializeRequestParam, Tool};
use rmcp::service::{RunningService, ServerSink};
use rmcp::transport::StreamableHttpClientTransport;
use rmcp::{RoleClient, ServiceExt};
use std::borrow::Cow;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct LoggingMcpTool {
    definition: Tool,
    client: ServerSink,
    called_tools: Arc<Mutex<Vec<String>>>,
}

impl LoggingMcpTool {
    pub fn new(
        definition: Tool,
        client: ServerSink,
        called_tools: Arc<Mutex<Vec<String>>>,
    ) -> Self {
        Self {
            definition,
            client,
            called_tools,
        }
    }
}

impl ToolDyn for LoggingMcpTool {
    fn name(&self) -> String {
        self.definition.name.to_string()
    }

    fn definition(
        &self,
        _prompt: String,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ToolDefinition> + Send + '_>> {
        Box::pin(async move {
            ToolDefinition {
                name: self.definition.name.to_string(),
                description: self
                    .definition
                    .description
                    .clone()
                    .unwrap_or(Cow::from(""))
                    .to_string(),
                parameters: self.definition.schema_as_json_value(),
            }
        })
    }

    fn call(
        &self,
        args: String,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, ToolError>> + Send + '_>>
    {
        let name = self.definition.name.clone();
        let arguments: Option<serde_json::Map<String, serde_json::Value>> =
            serde_json::from_str(&args).unwrap_or_default();
        let called_tools = self.called_tools.clone();

        Box::pin(async move {
            {
                let mut tools = called_tools.lock().await;
                tools.push(name.to_string());
            }

            eprintln!("\r[>] Calling tool: {}\x1b[K", name);

            let result = self
                .client
                .call_tool(rmcp::model::CallToolRequestParam { name, arguments })
                .await
                .map_err(|e| {
                    ToolError::ToolCallError(Box::new(std::io::Error::other(format!(
                        "Tool call error: {:?}",
                        e
                    ))))
                })?;

            if let Some(true) = result.is_error {
                let error_msg: Vec<String> = result
                    .content
                    .iter()
                    .filter_map(|x| x.raw.as_text().map(|t| t.text.clone()))
                    .collect();
                return Err(ToolError::ToolCallError(Box::new(std::io::Error::other(
                    error_msg.join("\n"),
                ))));
            }

            Ok(result
                .content
                .into_iter()
                .filter_map(|c| c.raw.as_text().map(|t| t.text.clone()))
                .collect::<Vec<_>>()
                .join("\n"))
        })
    }
}

pub struct McpClient {
    client: RunningService<RoleClient, InitializeRequestParam>,
}

impl McpClient {
    pub async fn connect(server_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let transport = StreamableHttpClientTransport::from_uri(server_url);

        let client_info = ClientInfo {
            protocol_version: Default::default(),
            capabilities: ClientCapabilities::default(),
            client_info: Implementation {
                name: "singleshot".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                ..Default::default()
            },
        };

        let client = client_info
            .serve(transport)
            .await
            .map_err(|e| format!("Failed to connect to MCP server at {}: {:?}", server_url, e))?;

        Ok(Self { client })
    }

    pub async fn list_tools(&self) -> Result<Vec<Tool>, Box<dyn std::error::Error>> {
        let response = self
            .client
            .list_tools(Default::default())
            .await
            .map_err(|e| format!("Failed to list tools from MCP server: {:?}", e))?;

        Ok(response.tools)
    }

    pub fn peer(&self) -> ServerSink {
        self.client.peer().clone()
    }
}

pub struct McpSession {
    #[allow(dead_code)]
    client: McpClient,
    pub tools: Vec<Tool>,
    pub peer: ServerSink,
}

impl McpSession {
    pub async fn connect(server_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = McpClient::connect(server_url).await?;
        let tools = client.list_tools().await?;
        let peer = client.peer();

        Ok(Self {
            client,
            tools,
            peer,
        })
    }

    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }

    pub fn tool_names(&self) -> Vec<String> {
        self.tools.iter().map(|t| t.name.to_string()).collect()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_mcp_module_compiles() {
        assert!(true);
    }
}
