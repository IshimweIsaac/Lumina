"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const vscode_1 = require("vscode");
const node_1 = require("vscode-languageclient/node");
let client;
function activate(context) {
    const serverOptions = {
        command: "lumina-lsp",
        args: [],
        transport: node_1.TransportKind.stdio,
    };
    const clientOptions = {
        documentSelector: [{ scheme: "file", language: "lumina" }],
        synchronize: {
            fileEvents: vscode_1.workspace.createFileSystemWatcher("**/*.lum"),
        },
    };
    client = new node_1.LanguageClient("lumina", "Lumina Language Server", serverOptions, clientOptions);
    client.start();
}
function deactivate() {
    return client ? client.stop() : undefined;
}
//# sourceMappingURL=extension.js.map