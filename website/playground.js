import init, { LuminaRuntime } from './wasm/lumina_wasm.js';

let isWasmLoaded = false;
let initPromise = null;

// Initialize WASM globally once
export async function initializeLumina() {
  if (isWasmLoaded) return;
  if (initPromise) return initPromise;
  
  initPromise = init('/lumina_wasm_bg.wasm').then(() => {
    isWasmLoaded = true;
    console.log("Lumina core initialized via WASM.");
  }).catch(err => {
    console.error("Failed to load Lumina WASM engine:", err);
  });
  
  return initPromise;
}

// Ensure it's initialized
initializeLumina();

// Execute a single snippet of Lumina code
export async function runLuminaScript(sourceText) {
  if (!isWasmLoaded) {
    await initializeLumina();
  }

  const result = {
    output: "",
    state: "",
    error: null,
    ticks: 0
  };

  try {
    // 1. Compile/Check
    const checkResult = LuminaRuntime.check(sourceText);
    let diagnostics = [];
    try {
        diagnostics = JSON.parse(checkResult);
    } catch(e) {
        // Fallback for non-JSON errors (shouldn't happen in v1.8)
        result.error = checkResult;
        return result;
    }

    if (diagnostics.length > 0) {
      result.error = diagnostics; // Return the array of diagnostic objects
      return result;
    }

    // 2. Instantiate Runtime
    const runtime = new LuminaRuntime(sourceText);
    
    // 3. Tick multiple times to allow reactive rules to settle
    for (let i = 0; i < 10; i++) {
        runtime.tick();
        result.ticks++;
    }

    // 4. Extract Output and Final State
    result.output = runtime.get_output();
    result.state = runtime.export_state();
    
    runtime.free();

  } catch (e) {
      // constructor failed check if it's a JSON diagnostic list
      try {
          const diags = JSON.parse(e.toString());
          result.error = diags;
      } catch(_) {
          result.error = e.toString();
      }
  }

  return result;
}
