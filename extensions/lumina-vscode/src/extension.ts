import { workspace, ExtensionContext } from "vscode";
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient;

export function activate(context: ExtensionContext) {
    const config = workspace.getConfiguration("lumina");
    const serverPath = config.get<string>("serverPath") || "lumina-lsp";

    const serverOptions: ServerOptions = {
        command: serverPath,
        args: [],
        transport: TransportKind.stdio,
        options: {
            shell: true,
        },
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: "file", language: "lumina" }],
        synchronize: {
            fileEvents: workspace.createFileSystemWatcher("**/*.lum"),
        },
        initializationOptions: {},
    };

    client = new LanguageClient(
        "lumina",
        "Lumina Language Server",
        serverOptions,
        clientOptions
    );

    client.start().catch((err) => {
        console.error("Failed to start Lumina LSP:", err);
    });
}

export function deactivate(): Thenable<void> | undefined {
    return client ? client.stop() : undefined;
}
