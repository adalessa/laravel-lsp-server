use parser::MyParser;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use tree_sitter::Point;

mod parser;

#[derive(Debug)]
struct Backend {
    client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![">".to_string()]),
                    all_commit_characters: None,
                    work_done_progress_options: Default::default(),
                    ..Default::default()
                }),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    file_operations: None,
                }),
                definition_provider: Some(OneOf::Left(true)),
                ..ServerCapabilities::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "[Laravel LSP] server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let folders = match self.client.workspace_folders().await {
            Ok(result) => match result {
                Some(res) => res,
                None => {
                    return Ok(None);
                }
            },
            Err(_) => {
                return Ok(None);
            }
        };

        let project_workspace = match folders.first() {
            Some(workspace) => workspace,
            None => {
                return Ok(None);
            }
        };

        let file_path = match params
            .text_document_position_params
            .text_document
            .uri
            .to_file_path()
        {
            Ok(file_path) => file_path,
            Err(_) => {
                return Ok(None);
            }
        };

        let source_code = match std::fs::read_to_string(file_path) {
            Ok(source_code) => source_code,
            Err(_) => {
                return Ok(None);
            }
        };

        let my_parser = MyParser::new(source_code.as_str());

        let point = Point {
            row: params.text_document_position_params.position.line as usize,
            column: params.text_document_position_params.position.character as usize,
        };

        // from here is code to get for the view command
        let node = match my_parser.get_node_at_point(&point) {
            Some(node) => node,
            None => {
                return Ok(None);
            }
        };

        match my_parser.get_view_path_from_node(node) {
            Some(path) => {
                let uri = Url::parse(
                    format!("{}/{}", project_workspace.uri.to_string(), path).as_str()
                ).unwrap();
                let range = Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: 0,
                        character: 0,
                    },
                };

                return Ok(Some(GotoDefinitionResponse::Scalar(Location::new(
                    uri, range,
                ))));
            }
            None => {
                return Ok(None);
            }
        }
    }

    async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
        // let document_position = params.text_document_position;
        // let file_path = document_position.text_document.uri.to_file_path().unwrap();
        //
        // let completion_point = Point{row: document_position.position.line as usize, column: document_position.position.character as usize};
        //
        // let mut my_parser = MyParser::new(file_path);
        // let _function = my_parser.get_function_at_position(&completion_point);

        Ok(Some(CompletionResponse::Array(vec![
            CompletionItem::new_simple("Hello".to_string(), "Some detail".to_string()),
            CompletionItem::new_simple("Bye".to_string(), "More detail".to_string()),
        ])))
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend { client });
    Server::new(stdin, stdout, socket).serve(service).await;
}
