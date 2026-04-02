import { useEffect, useState } from "react";
import Editor from "@monaco-editor/react";
import initWasm, { LuminaRuntime } from "lumina-wasm";

import { StatePanel } from "./StatePanel";
import { AlertTimeline } from "./AlertTimeline";
import { VirtualClock } from "./VirtualClock";
import { ShareButton, loadFromURL } from "./ShareButton";
import { EXAMPLES } from "./examples";
import "./App.css";

function getInitialSource(): string {
    const shared = loadFromURL();
    if (shared) return shared;
    const params = new URLSearchParams(window.location.search);
    const example = params.get("example");
    if (example && example in EXAMPLES) {
        return EXAMPLES[example as keyof typeof EXAMPLES].source;
    }
    return EXAMPLES.fleet.source;
}

function App() {
    const [source, setSource] = useState(getInitialSource());
    const [runtime, setRuntime] = useState<LuminaRuntime | null>(null);
    const [alerts, setAlerts] = useState<any[]>([]);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        initWasm().then(() => {
            compileAndRun(source);
        });
    }, []);

    const compileAndRun = (code: string) => {
        try {
            const err = LuminaRuntime.check(code);
            if (err) {
                setError(err);
                return;
            }
            const rt = new LuminaRuntime(code);
            setRuntime(rt);
            setError(null);
            setAlerts([]);
        } catch (e: any) {
            setError(e.toString());
        }
    };

    const handleRun = () => compileAndRun(source);

    const handleAlertsRaw = (tickResult: string) => {
        if (!tickResult) return;
        if (tickResult.startsWith("ERROR:")) {
            setError(tickResult.substring(6));
            return;
        }
        try {
            const evts = JSON.parse(tickResult);
            if (evts && evts.length > 0) {
                setAlerts(prev => [...prev, ...evts]);
            }
        } catch (e) {}
    };

    return (
        <div className="app-container">
            <header>
                <h1>Lumina Playground v2</h1>
                <div className="toolbar">
                    <select 
                        className="example-selector"
                        onChange={(e) => {
                            const val = e.target.value;
                            if (val in EXAMPLES) {
                                const newSource = EXAMPLES[val as keyof typeof EXAMPLES].source;
                                setSource(newSource);
                                compileAndRun(newSource);
                            }
                        }}
                        defaultValue={new URLSearchParams(window.location.search).get("example") || "fleet"}
                    >
                        {Object.entries(EXAMPLES).map(([key, ex]) => (
                            <option key={key} value={key}>{ex.name}</option>
                        ))}
                    </select>
                    <button onClick={handleRun}>Compile & Run</button>
                    <VirtualClock rt={runtime} onAlerts={handleAlertsRaw} />
                    <ShareButton source={source} />
                </div>
            </header>
            <main>
                <div className="editor-pane">
                    <Editor
                        height="100%"
                        defaultLanguage="shell"
                        theme="vs-dark"
                        value={source}
                        onChange={(v) => setSource(v || "")}
                        options={{ minimap: { enabled: false } }}
                    />
                    {error && <div className="error-panel"><pre>{error}</pre></div>}
                </div>
                <div className="state-pane">
                    <StatePanel rt={runtime} />
                    <AlertTimeline events={alerts} />
                </div>
            </main>
        </div>
    );
}

export default App;
