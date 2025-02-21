<script lang="ts">
    import { onMount } from "svelte";
    import JSEncrypt from "jsencrypt"; // 引入 JSEncrypt 库
    import CryptoJS from "crypto-js"; // 引入 CryptoJS 库

    let inputText = "";
    let outputText = "";
    let selectedAlgorithm = "AES";
    let publicKey = "";
    let privateKey = "";

    const algorithms = ["AES", "RSA", "DES", "Blowfish", "Base64"];

    // 新增：生成 RSA 私钥的函数
    const generateRSAKeyPair = () => {
        const encryptor = new JSEncrypt({ default_key_size: "2048" });
        encryptor.getKey();
        privateKey = encryptor.getPrivateKey();
        publicKey = encryptor.getPublicKey();
    };

    const encrypt = () => {
        if (selectedAlgorithm === "Base64") {
            outputText = btoa(inputText); // Base64 编码
        } else if (selectedAlgorithm === "RSA") {
            // 使用公钥进行加密
            const encryptor = new JSEncrypt();
            encryptor.setPublicKey(publicKey);
            const encrypted = encryptor.encrypt(inputText);
            outputText = encrypted || "加密失败"; // 如果加密失败，返回默认值
        } else if (selectedAlgorithm === "DES") {
            // 使用 DES 加密
            const cipher = CryptoJS.DES.encrypt(inputText, privateKey).toString();
            outputText = cipher;
        } else if (selectedAlgorithm === "Blowfish") {
            // 使用 Blowfish 加密
            const cipher = CryptoJS.Blowfish.encrypt(inputText, privateKey).toString();
            outputText = cipher;
        } else {
            // 使用 AES 加密
            const cipher = CryptoJS.AES.encrypt(inputText, privateKey).toString();
            outputText = cipher;
        }
    };

    const decrypt = () => {
        if (selectedAlgorithm === "Base64") {
            outputText = atob(inputText); // Base64 解码
        } else if (selectedAlgorithm === "RSA") {
            // 使用私钥进行解密
            const decryptor = new JSEncrypt();
            decryptor.setPrivateKey(privateKey);
            const decrypted = decryptor.decrypt(inputText);
            outputText = decrypted || "解密失败"; // 如果解密失败，返回默认值
        } else if (selectedAlgorithm === "DES") {
            // 使用 DES 解密
            const bytes = CryptoJS.DES.decrypt(inputText, privateKey);
            outputText = bytes.toString(CryptoJS.enc.Utf8) || "解密失败"; // 确保 outputText 为 string 类型
        } else if (selectedAlgorithm === "Blowfish") {
            // 使用 Blowfish 解密
            const bytes = CryptoJS.Blowfish.decrypt(inputText, privateKey);
            outputText = bytes.toString(CryptoJS.enc.Utf8) || "解密失败"; // 确保 outputText 为 string 类型
        } else {
            // 使用 AES 解密
            const bytes = CryptoJS.AES.decrypt(inputText, privateKey);
            outputText = bytes.toString(CryptoJS.enc.Utf8) || "解密失败"; // 确保 outputText 为 string 类型
        }
    };

    const requiresKeys = (algorithm: string) => {
        return algorithm !== "Base64";
    };

    const requiresPublicKey = (algorithm: string) => {
        return algorithm === "RSA";
    };
</script>

<div class="container">
    <h1>编码加解密工具</h1>
    <textarea bind:value={inputText} placeholder="输入待加密或解密的文本"
    ></textarea>
    {#if requiresKeys(selectedAlgorithm)}
        {#if requiresPublicKey(selectedAlgorithm)}
            <input
                class="key-input"
                bind:value={publicKey}
                placeholder="输入公钥"
            />
        {/if}
        <input
            class="key-input"
            bind:value={privateKey}
            placeholder="输入密钥"
        />
        {#if selectedAlgorithm === "RSA"}
            <button on:click={generateRSAKeyPair}>生成 RSA 密钥对</button>
        {/if}
    {/if}
    <select bind:value={selectedAlgorithm}>
        {#each algorithms as algorithm}
            <option value={algorithm}>{algorithm}</option>
        {/each}
    </select>
    <button on:click={encrypt}>加密</button>
    <button on:click={decrypt}>解密</button>
    <textarea readonly bind:value={outputText} placeholder="加密或解密后的结果"
    ></textarea>
</div>

<style>
    .container {
        max-width: 100%;
        max-height: 100%;
        margin: 0 auto;
        padding: 20px;
        text-align: center;
    }
    textarea {
        width: 100%;
        height: 200px;
        margin-bottom: 10px;
        padding: 10px;
        border: 1px solid #ccc;
        border-radius: 4px;
        font-size: 16px;
    }
    select,
    button {
        margin: 5px;
        padding: 10px 20px;
        border: none;
        border-radius: 4px;
        background-color: #007bff;
        color: white;
        font-size: 16px;
        cursor: pointer;
    }
    select:hover,
    button:hover {
        background-color: #0056b3;
    }
    select {
        background-color: #007bff; /* 设置选项框背景色为蓝色 */
        color: rgb(12, 12, 12); /* 设置文字颜色为白色 */
        padding: 10px 20px; /* 增加内边距 */
        font-size: 16px; /* 设置字体大小 */
    }
    .key-input {
        width: 100%;
        margin-bottom: 10px;
        padding: 10px;
        border: 1px solid #ccc;
        border-radius: 4px;
        font-size: 16px;
    }
    h1 {
        color: #333;
        margin-bottom: 20px;
    }
</style>
