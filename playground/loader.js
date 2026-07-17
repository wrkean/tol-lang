const textEncoder = new TextEncoder();
const textDecoder = new TextDecoder("utf-8");

let wasm;

async function init() {
  const response = await fetch("./pkg/tol_playground.wasm");
  const module = response.headers.get("content-type") === "application/wasm"
    ? await WebAssembly.instantiateStreaming(response, {})
    : await WebAssembly.instantiate(await response.arrayBuffer(), {});
  const { instance } = module;
  wasm = instance.exports;
}

function writeString(value) {
  const bytes = textEncoder.encode(value);
  const ptr = wasm.alloc(bytes.length);
  const memory = new Uint8Array(wasm.memory.buffer);
  memory.set(bytes, ptr);
  return { ptr, len: bytes.length };
}

function readString(ptr, len) {
  if (len === 0) {
    return "";
  }

  const memory = new Uint8Array(wasm.memory.buffer, ptr, len);
  return textDecoder.decode(memory);
}

function readOutput() {
  return readString(wasm.output_ptr(), wasm.output_len());
}

function readDiagnostics() {
  return readString(wasm.diagnostics_ptr(), wasm.diagnostics_len());
}

function render(outputNode, diagnosticsNode) {
  outputNode.textContent = readOutput();
  diagnosticsNode.textContent = readDiagnostics();
}

async function main() {
  await init();

  const source = document.querySelector("#source");
  const output = document.querySelector("#output");
  const diagnostics = document.querySelector("#diagnostics");
  const run = document.querySelector("#run");

  const execute = () => {
    const { ptr, len } = writeString(source.value);
    const success = wasm.run_playground(ptr, len);
    wasm.dealloc(ptr, len);
    render(output, diagnostics);
  };

  run.addEventListener("click", execute);
  source.addEventListener("keydown", (event) => {
    if ((event.ctrlKey || event.metaKey) && event.key === "Enter") {
      execute();
    }
  });

  execute();
}

main().catch((error) => {
  console.error(error);
});
