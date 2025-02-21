<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let jsonInput = $state("");
  let jsonOutput = $state("");
  let goStructOutput = $state("");
  let errorMessage = $state("");
  let isProcessing = $state(false);

  async function parseJson() {
    if (isProcessing) return;
    isProcessing = true;
    try {
      const parsed = await processJsonInChunks(jsonInput, (chunk) => JSON.parse(chunk));
      jsonOutput = JSON.stringify(parsed, null, 2);
      errorMessage = "";
    } catch (e) {
      errorMessage = "Invalid JSON: " + e;
    } finally {
      isProcessing = false;
    }
  }

  async function compressJson() {
    if (isProcessing) return;
    isProcessing = true;
    try {
      const parsed = await processJsonInChunks(jsonInput, (chunk) => JSON.parse(chunk));
      jsonOutput = JSON.stringify(parsed);
      errorMessage = "";
    } catch (e) {
      errorMessage = "Invalid JSON: " + e;
    } finally {
      isProcessing = false;
    }
  }

  async function convertToGoStruct() {
    if (isProcessing) return;
    isProcessing = true;
    try {
      const parsed = await processJsonInChunks(jsonInput, (chunk) => JSON.parse(chunk));
      goStructOutput = await invoke("json_to_go_struct", { json: jsonInput });
      errorMessage = "";
    } catch (e) {
      errorMessage = "Invalid JSON: " + e;
    } finally {
      isProcessing = false;
    }
  }

  async function processJsonInChunks(json: string, processFn: (chunk: string) => any) {
    const chunkSize = 500000; // Adjust chunk size as needed
    let result = {};
    for (let i = 0; i < json.length; i += chunkSize) {
      const chunk = json.slice(i, i + chunkSize);
      result = { ...result, ...processFn(chunk) };
      await new Promise((resolve) => setTimeout(resolve, 0)); // Yield to the event loop
    }
    return result;
  }
</script>

<main class="container">
  <div class="row">
    <textarea id="json-input" placeholder="Enter JSON..." bind:value={jsonInput}></textarea>
    <textarea id="json-output" placeholder="JSON Output..." bind:value={jsonOutput} readonly></textarea>
  </div>

  <div class="row">
    <button type="button" on:click={parseJson}>Parse JSON</button>
    <button type="button" on:click={compressJson}>Compress JSON</button>
    <button type="button" on:click={convertToGoStruct}>Convert to Go Struct</button>
  </div>

  {#if errorMessage}
    <p class="error">{errorMessage}</p>
  {/if}

  <div class="row">
    <textarea id="go-struct-output" placeholder="Go Struct Output..." bind:value={goStructOutput} readonly></textarea>
  </div>
</main>

<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.container {
  margin: 0;
  padding-top: 10px;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}

.row {
  display: flex;
  justify-content: center;
  gap: 20px; /* 添加间距 */
}

button {
  cursor: pointer;
}

button:hover {
  border-color: #396cd8;
}
button:active {
  border-color: #396cd8;
  background-color: #e8e8e8;
}

button {
  outline: none;
}

textarea {
  width: 100%; /* 自适应宽度 */
  height: calc(100vh - 200px); /* 自适应高度 */
  margin: 10px 0;
  padding: 10px;
  border-radius: 8px;
  border: 1px solid #ccc;
  font-family: monospace;
}

.error {
  color: red;
  font-weight: bold;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }

  button {
    color: #ffffff;
    background-color: #0f0f0f98;
  }
  button:active {
    background-color: #0f0f0f69;
  }
}
</style>
