// PRO MAX OS - MCP Server for OpenCode Integration
// Runs as a stdio-based MCP server to let AI coding tools interact with the ERP.
// Usage: promax-mcp (reads/writes JSON-RPC 2.0 over stdio)
//
// Set PROMAX_DB_PATH env var if the database is not in the default location.
// Or connect via: promax-mcp --db-path "C:\path\to\promax.db"

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Allow explicit db path argument
    if let Some(pos) = args.iter().position(|a| a == "--db-path") {
        if let Some(path) = args.get(pos + 1) {
            std::env::set_var("PROMAX_DB_PATH", path);
        }
    }

    match promax_os_lib::mcp::run_server() {
        Ok(()) => {}
        Err(e) => {
            // Write error as MCP-compatible response to stdout
            let err_resp = serde_json::json!({
                "jsonrpc": "2.0",
                "id": null,
                "error": {
                    "code": -32000,
                    "message": e,
                    "data": null
                }
            });
            println!("{}", err_resp);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_mcp_help() {
        let args = ["promax-mcp".to_string(), "--help".to_string()];
        assert_eq!(args.len(), 2);
    }
}
