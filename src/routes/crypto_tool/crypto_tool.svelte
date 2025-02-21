<script lang="ts">
    import { onMount } from 'svelte';

    let inputText = '';
    let outputText = '';
    let selectedAlgorithm = 'AES';

    const algorithms = ['AES', 'RSA', 'DES', 'Blowfish', 'Base64'];

    const encrypt = () => {
        if (selectedAlgorithm === 'Base64') {
            outputText = btoa(inputText); // Base64 编码
        } else {
            // 这里实现其他加密逻辑
            outputText = `Encrypted: ${inputText}`;
        }
    };

    const decrypt = () => {
        if (selectedAlgorithm === 'Base64') {
            outputText = atob(inputText); // Base64 解码
        } else {
            // 这里实现其他解密逻辑
            outputText = `Decrypted: ${inputText}`;
        }
    };
</script>

<style>
    .container {
        max-width: 600px;
        margin: 0 auto;
        padding: 20px;
        text-align: center;
    }
    textarea {
        width: 100%;
        height: 100px;
        margin-bottom: 10px;
    }
    select, button {
        margin: 5px;
        padding: 10px;
    }
</style>

<div class="container">
    <h1>编码加解密工具</h1>
    <textarea bind:value={inputText} placeholder="输入待加密或解密的文本"></textarea>
    <select bind:value={selectedAlgorithm}>
        {#each algorithms as algorithm}
            <option value={algorithm}>{algorithm}</option>
        {/each}
    </select>
    <button on:click={encrypt}>加密</button>
    <button on:click={decrypt}>解密</button>
    <textarea readonly bind:value={outputText} placeholder="加密或解密后的结果"></textarea>
</div>