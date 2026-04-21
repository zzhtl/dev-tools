<script lang="ts">
    import JSEncrypt from "jsencrypt";
    import CryptoJS from "crypto-js";

    let inputText = $state("");
    let outputText = $state("");
    let selectedAlgorithm = $state("AES");
    let publicKey = $state("");
    let privateKey = $state("");
    let errorMessage = $state("");
    let successMessage = $state("");
    let copied = $state(false);

    const algorithms = [
        { id: "AES", name: "AES", desc: "高级加密标准，对称加密" },
        { id: "RSA", name: "RSA", desc: "非对称加密，需要公私钥对" },
        { id: "DES", name: "DES", desc: "数据加密标准，对称加密" },
        { id: "TripleDES", name: "3DES", desc: "三重DES，更安全" },
        { id: "Base64", name: "Base64", desc: "编码方式，非加密" },
    ];

    function generateRSAKeyPair() {
        try {
            const encryptor = new JSEncrypt({ default_key_size: "2048" });
            encryptor.getKey();
            privateKey = encryptor.getPrivateKey();
            publicKey = encryptor.getPublicKey();
            successMessage = "RSA密钥对生成成功";
            setTimeout(() => successMessage = "", 2000);
        } catch (err: any) {
            errorMessage = "生成密钥对失败: " + err.message;
            setTimeout(() => errorMessage = "", 3000);
        }
    }

    function encrypt() {
        errorMessage = "";
        if (!inputText.trim()) {
            errorMessage = "请输入要加密的文本";
            return;
        }

        try {
            if (selectedAlgorithm === "Base64") {
                outputText = btoa(unescape(encodeURIComponent(inputText)));
            } else if (selectedAlgorithm === "RSA") {
                if (!publicKey) {
                    errorMessage = "请先生成或输入RSA公钥";
                    return;
                }
                const encryptor = new JSEncrypt();
                encryptor.setPublicKey(publicKey);
                const encrypted = encryptor.encrypt(inputText);
                outputText = encrypted || "";
                if (!encrypted) {
                    errorMessage = "RSA加密失败，请检查公钥格式";
                }
            } else if (selectedAlgorithm === "DES") {
                if (!privateKey) {
                    errorMessage = "请输入DES密钥";
                    return;
                }
                outputText = CryptoJS.DES.encrypt(inputText, privateKey).toString();
            } else if (selectedAlgorithm === "TripleDES") {
                if (!privateKey) {
                    errorMessage = "请输入3DES密钥";
                    return;
                }
                outputText = CryptoJS.TripleDES.encrypt(inputText, privateKey).toString();
            } else {
                // AES
                if (!privateKey) {
                    errorMessage = "请输入AES密钥";
                    return;
                }
                outputText = CryptoJS.AES.encrypt(inputText, privateKey).toString();
            }
            
            if (outputText) {
                successMessage = "加密成功";
                setTimeout(() => successMessage = "", 2000);
            }
        } catch (err: any) {
            errorMessage = "加密失败: " + err.message;
        }
    }

    function decrypt() {
        errorMessage = "";
        if (!inputText.trim()) {
            errorMessage = "请输入要解密的文本";
            return;
        }

        try {
            if (selectedAlgorithm === "Base64") {
                outputText = decodeURIComponent(escape(atob(inputText)));
            } else if (selectedAlgorithm === "RSA") {
                if (!privateKey) {
                    errorMessage = "请先生成或输入RSA私钥";
                    return;
                }
                const decryptor = new JSEncrypt();
                decryptor.setPrivateKey(privateKey);
                const decrypted = decryptor.decrypt(inputText);
                outputText = decrypted || "";
                if (!decrypted) {
                    errorMessage = "RSA解密失败，请检查私钥或密文";
                }
            } else if (selectedAlgorithm === "DES") {
                if (!privateKey) {
                    errorMessage = "请输入DES密钥";
                    return;
                }
                const bytes = CryptoJS.DES.decrypt(inputText, privateKey);
                outputText = bytes.toString(CryptoJS.enc.Utf8);
                if (!outputText) {
                    errorMessage = "DES解密失败，请检查密钥或密文";
                }
            } else if (selectedAlgorithm === "TripleDES") {
                if (!privateKey) {
                    errorMessage = "请输入3DES密钥";
                    return;
                }
                const bytes = CryptoJS.TripleDES.decrypt(inputText, privateKey);
                outputText = bytes.toString(CryptoJS.enc.Utf8);
                if (!outputText) {
                    errorMessage = "3DES解密失败，请检查密钥或密文";
                }
            } else {
                // AES
                if (!privateKey) {
                    errorMessage = "请输入AES密钥";
                    return;
                }
                const bytes = CryptoJS.AES.decrypt(inputText, privateKey);
                outputText = bytes.toString(CryptoJS.enc.Utf8);
                if (!outputText) {
                    errorMessage = "AES解密失败，请检查密钥或密文";
                }
            }

            if (outputText) {
                successMessage = "解密成功";
                setTimeout(() => successMessage = "", 2000);
            }
        } catch (err: any) {
            errorMessage = "解密失败: " + err.message;
        }
    }

    async function copyOutput() {
        if (!outputText) return;
        try {
            await navigator.clipboard.writeText(outputText);
            copied = true;
            setTimeout(() => copied = false, 1500);
        } catch (err) {
            console.error("复制失败:", err);
        }
    }

    function swapInputOutput() {
        const temp = inputText;
        inputText = outputText;
        outputText = temp;
    }

    function clearAll() {
        inputText = "";
        outputText = "";
        errorMessage = "";
        successMessage = "";
    }

    const needsKey = $derived(selectedAlgorithm !== "Base64");
    const isRSA = $derived(selectedAlgorithm === "RSA");
</script>

<div class="crypto-tool">
    <div class="algorithm-selector">
        <h3>选择加密算法</h3>
        <div class="algorithm-grid">
            {#each algorithms as algo}
                <button
                    class="algorithm-card"
                    class:active={selectedAlgorithm === algo.id}
                    onclick={() => selectedAlgorithm = algo.id}
                >
                    <span class="algo-name">{algo.name}</span>
                    <span class="algo-desc">{algo.desc}</span>
                </button>
            {/each}
        </div>
    </div>

    {#if needsKey}
        <div class="key-section">
            <div class="key-header">
                <h3>密钥设置</h3>
                {#if isRSA}
                    <button class="generate-btn" onclick={generateRSAKeyPair}>
                        ⟳ 生成 RSA 密钥对
                    </button>
                {/if}
            </div>
            
            {#if isRSA}
                <div class="key-grid">
                    <div class="key-input-group">
                        <label for="crypto-public-key">公钥 (用于加密)</label>
                        <textarea
                            id="crypto-public-key"
                            bind:value={publicKey}
                            placeholder="RSA公钥..."
                            rows="4"
                        ></textarea>
                    </div>
                    <div class="key-input-group">
                        <label for="crypto-private-key">私钥 (用于解密)</label>
                        <textarea
                            id="crypto-private-key"
                            bind:value={privateKey}
                            placeholder="RSA私钥..."
                            rows="4"
                        ></textarea>
                    </div>
                </div>
            {:else}
                <div class="key-input-group">
                    <label for="crypto-secret-key">密钥</label>
                    <input
                        id="crypto-secret-key"
                        type="text"
                        bind:value={privateKey}
                        placeholder="输入加密密钥..."
                    />
                </div>
            {/if}
        </div>
    {/if}

    <div class="converter-area">
        <div class="input-panel">
            <div class="panel-header">
                <span>输入文本</span>
                <span class="char-count">{inputText.length} 字符</span>
            </div>
            <textarea
                bind:value={inputText}
                placeholder="输入要加密或解密的文本..."
                rows="8"
            ></textarea>
        </div>

        <div class="action-buttons-center">
            <button class="action-btn encrypt-btn" onclick={encrypt}>
                🔒 加密
            </button>
            <button class="swap-btn" onclick={swapInputOutput} title="交换">
                ⇄
            </button>
            <button class="action-btn decrypt-btn" onclick={decrypt}>
                🔓 解密
            </button>
        </div>

        <div class="output-panel">
            <div class="panel-header">
                <span>输出结果</span>
                <div class="output-actions">
                    <span class="char-count">{outputText.length} 字符</span>
                    {#if outputText}
                        <button class="copy-btn" onclick={copyOutput}>
                            {copied ? '✓ 已复制' : '复制'}
                        </button>
                    {/if}
                </div>
            </div>
            <textarea
                bind:value={outputText}
                placeholder="处理结果将显示在这里..."
                rows="8"
                readonly
            ></textarea>
        </div>
    </div>

    {#if errorMessage}
        <div class="message error-message">
            ⚠️ {errorMessage}
        </div>
    {/if}

    {#if successMessage}
        <div class="message success-message">
            ✓ {successMessage}
        </div>
    {/if}

    <div class="tool-actions">
        <button class="secondary-btn" onclick={clearAll}>清空所有</button>
    </div>
</div>

<style>
    .crypto-tool {
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
    }

    .algorithm-selector h3,
    .key-section h3 {
        font-size: 0.95rem;
        margin-bottom: 0.75rem;
        color: var(--text-secondary);
    }

    .algorithm-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
        gap: 0.75rem;
    }

    .algorithm-card {
        display: flex;
        flex-direction: column;
        gap: 0.25rem;
        padding: 1rem !important;
        background: var(--bg-dark) !important;
        border: 1px solid var(--border) !important;
        text-align: left;
        cursor: pointer;
        transition: all 0.2s;
    }

    .algorithm-card:hover {
        border-color: var(--primary) !important;
        transform: none !important;
        box-shadow: none !important;
    }

    .algorithm-card.active {
        border-color: var(--accent) !important;
        background: rgba(34, 211, 238, 0.1) !important;
    }

    .algo-name {
        font-weight: 600;
        font-size: 1rem;
        color: var(--text-primary);
    }

    .algo-desc {
        font-size: 0.8rem;
        color: var(--text-muted);
    }

    .key-section {
        padding: 1rem;
        background: var(--bg-dark);
        border: 1px solid var(--border);
        border-radius: var(--radius-sm);
    }

    .key-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 1rem;
    }

    .key-header h3 {
        margin-bottom: 0;
    }

    .generate-btn {
        padding: 0.5rem 1rem !important;
        font-size: 0.85rem !important;
        background: linear-gradient(135deg, var(--accent-green) 0%, #059669 100%) !important;
    }

    .key-grid {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 1rem;
    }

    .key-input-group {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .key-input-group label {
        font-size: 0.85rem;
        font-weight: 500;
    }

    .key-input-group textarea {
        font-family: 'JetBrains Mono', monospace;
        font-size: 0.8rem;
        resize: vertical;
        min-height: 80px;
    }

    .converter-area {
        display: grid;
        grid-template-columns: 1fr auto 1fr;
        gap: 1rem;
        align-items: stretch;
    }

    .input-panel, .output-panel {
        border: 1px solid var(--border);
        border-radius: var(--radius-sm);
        overflow: hidden;
        display: flex;
        flex-direction: column;
    }

    .panel-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 0.75rem 1rem;
        background: var(--bg-dark);
        border-bottom: 1px solid var(--border);
        font-size: 0.85rem;
        color: var(--text-secondary);
    }

    .output-actions {
        display: flex;
        align-items: center;
        gap: 0.75rem;
    }

    .char-count {
        font-size: 0.8rem;
        color: var(--text-muted);
    }

    .copy-btn {
        padding: 0.3rem 0.75rem !important;
        font-size: 0.8rem !important;
        background: var(--bg-hover) !important;
        border: 1px solid var(--border) !important;
    }

    .input-panel textarea,
    .output-panel textarea {
        flex: 1;
        border: none !important;
        border-radius: 0 !important;
        resize: none;
        font-family: 'JetBrains Mono', monospace;
        min-height: 200px;
    }

    .output-panel textarea {
        background: var(--bg-dark) !important;
        color: var(--accent);
    }

    .action-buttons-center {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
        align-items: center;
        justify-content: center;
    }

    .action-btn {
        padding: 0.75rem 1.5rem !important;
        font-weight: 600 !important;
        min-width: 100px;
    }

    .encrypt-btn {
        background: linear-gradient(135deg, var(--primary) 0%, var(--primary-dark) 100%) !important;
    }

    .decrypt-btn {
        background: linear-gradient(135deg, var(--accent-green) 0%, #059669 100%) !important;
    }

    .swap-btn {
        width: 40px !important;
        height: 40px !important;
        padding: 0 !important;
        border-radius: 50% !important;
        background: var(--bg-dark) !important;
        border: 1px solid var(--border) !important;
        font-size: 1.1rem;
    }

    .swap-btn:hover {
        border-color: var(--primary) !important;
    }

    .message {
        padding: 0.75rem 1rem;
        border-radius: var(--radius-sm);
        font-size: 0.9rem;
    }

    .error-message {
        background: rgba(239, 68, 68, 0.1);
        border: 1px solid rgba(239, 68, 68, 0.3);
        color: #f87171;
    }

    .success-message {
        background: rgba(16, 185, 129, 0.1);
        border: 1px solid rgba(16, 185, 129, 0.3);
        color: #34d399;
    }

    .tool-actions {
        display: flex;
        justify-content: center;
    }

    .secondary-btn {
        padding: 0.5rem 1.5rem !important;
        background: var(--bg-hover) !important;
        border: 1px solid var(--border) !important;
    }

    @media (max-width: 768px) {
        .converter-area {
            grid-template-columns: 1fr;
        }

        .action-buttons-center {
            flex-direction: row;
            padding: 0.5rem 0;
        }

        .swap-btn {
            transform: rotate(90deg);
        }

        .key-grid {
            grid-template-columns: 1fr;
        }
    }
</style>
