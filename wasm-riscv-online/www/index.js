import * as wasm from 'wasm-riscv-online';

const PROCESSING_DELAY_MS = 500;

// 全局变量  
let isProcessing = false;
let keyboardShortcutsVisible = false;

try {
    // DOM 元素引用  
    const convertButton = document.getElementById('convertButton');
    const clearButton = document.getElementById('clearButton');
    const copyButton = document.getElementById('copyButton');
    const input = document.getElementById('input');
    const inputDisplay = document.getElementById('inputDisplay');
    const outputDisplay = document.getElementById('outputDisplay');
    const inputStatus = document.getElementById('inputStatus');
    const errorMessage = document.getElementById('errorMessage');
    const keyboardShortcuts = document.getElementById('keyboardShortcuts');

    // 输入验证函数  
    function validateHexInput(value) {
        const trimmed = value.trim();
        if (!trimmed) {
            return { valid: false, message: '请输入内容', type: 'warning' };
        }

        // 判断是否为010字节流
        const byteStreamPattern = /^([0-9a-fA-F]{2}[\s\r\n]*)+$/;
        if (byteStreamPattern.test(trimmed.replace(/[\r\n]+/g, ' ').replace(/\s+/g, ' '))) {
            return { valid: true, message: '字节流格式', type: 'valid', format: 'byteStream' };
        }

        // 否则逐行检查每行是否为十六进制串
        const lines = value.split('\n').filter(line => line.trim());
        const hexPattern = /^(0x|0X)?[0-9a-fA-F]+$/;

        for (let line of lines) {
            const cleanLine = line.trim();
            if (!hexPattern.test(cleanLine)) {
                return { valid: false, message: '包含无效的十六进制格式', type: 'error', format: null };
            }
        }

        return { valid: true, message: `${lines.length} 条指令`, type: 'valid', format: 'hex' };
    }

    // 更新输入状态显示  
    function updateInputStatus(validation) {
        inputStatus.textContent = validation.message;
        inputStatus.className = `input-status ${validation.type}`;

        input.className = validation.valid ? 'valid' : (validation.type === 'error' ? 'error' : '');
    }

    // 显示错误消息  
    function showError(message) {
        errorMessage.textContent = message;
        errorMessage.classList.add('show');
        setTimeout(() => {
            errorMessage.classList.remove('show');
        }, 5000);
    }
    
    // 解析 010 Editor 字节流为按行hex指令
    function parseByteStream(value) {
        // 规范化空白并分割字节 token（每个 token 应为两位十六进制）
        const tokens = value.replace(/\r\n/g, '\n').split(/\s+/).filter(Boolean);
        if (tokens.length === 0) return [];

        for (const t of tokens) {
            if (!/^[0-9a-fA-F]{2}$/.test(t)) {
                throw new Error(`非法字节 token: "${t}"`);
            }
        }

        const lower = tokens.map(t => t.toLowerCase());
        const instructions = [];
        let i = 0;
        while (i < lower.length) {
            if (i + 1 >= lower.length) {
                throw new Error('指令不完整，剩余字节不足');
            }
            const b0 = lower[i];     // 低字节（小端）
            const b1 = lower[i + 1]; // 高字节（小端的高）
            const value16 = (parseInt(b0, 16)) | (parseInt(b1, 16) << 8);

            // 如果最低两位为 11 -> 32 位指令（再读两个字节）
            if ((value16 & 0x3) === 0x3) {
                if (i + 3 >= lower.length) {
                    throw new Error('指令不完整，剩余字节不足');
                }
                const b2 = lower[i + 2];
                const b3 = lower[i + 3];
                // 为显示和后续处理生成大端顺序的 hex 字符（b3 b2 b1 b0）
                const hexBE = (b3 + b2 + b1 + b0).toLowerCase();
                instructions.push('0x' + hexBE.padStart(8, '0'));
                i += 4;
            } else {
                // 16 位指令，大端顺序 b1 b0
                const hexBE = (b1 + b0).toLowerCase();
                instructions.push('0x' + hexBE.padStart(4, '0'));
                i += 2;
            }
        }
        return instructions;
    }

    // 处理单条指令  
    function processSingleInstruction(hexValue) {
        // 移除 0x 前缀  
        if (hexValue.startsWith("0x") || hexValue.startsWith("0X")) {
            hexValue = hexValue.slice(2);
        }

        // 补齐偶数位  
        if (hexValue.length % 2 !== 0) {
            hexValue = '0' + hexValue;
        }

        // 转换为二进制判断指令长度  
        const binaryStr = parseInt(hexValue, 16).toString(2).padStart(32, '0');

        let formattedHexValue;
        if (binaryStr.endsWith('11')) {
            // 32 位指令  
            hexValue = hexValue.padStart(8, '0');
            formattedHexValue = '0x' + hexValue;
        } else {
            // 16 位指令  
            hexValue = hexValue.padStart(4, '0');
            formattedHexValue = '0x' + hexValue;
        }

        return {
            formatted: formattedHexValue,
            result: wasm.disassemble(formattedHexValue)
        };
    }

    // 语法高亮处理  
    function highlightAssembly(text) {
        if (text.startsWith('Error:')) {
            return `<span class="assembly-error">${text}</span>`;
        }

        // 简单的语法高亮  
        return text
            .replace(/\b(add|sub|mul|div|addi|subi|lw|sw|beq|bne|jal|jalr|nop|ret|li)\b/gi,
                '<span class="assembly-instruction">$1</span>')
            .replace(/\b(x[0-9]+|zero|ra|sp|gp|tp|t[0-6]|s[0-9]+|a[0-7])\b/g,
                '<span class="assembly-register">$1</span>')
            .replace(/\b(-?0x[0-9a-fA-F]+|-?[0-9]+)\b/g,
                '<span class="assembly-immediate">$1</span>');
    }

    // 主要的转换处理函数  
    function handleConversion() {
        if (isProcessing) return;

        let inputValue = input.value.trim();
        const validation = validateHexInput(inputValue);

        if (!validation.valid) {
            showError(validation.message);
            return;
        }

        if (validation.format === 'byteStream') {
            //转换为16进制格式
            try {
                const instructions = parseByteStream(inputValue);
                inputValue = instructions.join('\n');
            } catch (err) {
                showError(`${err.message}`);
                return;
            }
        }

        isProcessing = true;
        convertButton.disabled = true;
        convertButton.classList.add('loading');

        try {
            const lines = inputValue.split('\n').filter(line => line.trim());
            const results = [];
            const inputs = [];

            for (let line of lines) {
                const cleanLine = line.trim();
                try {
                    const result = processSingleInstruction(cleanLine);
                    inputs.push(result.formatted);
                    results.push(result.result);
                } catch (error) {
                    inputs.push(cleanLine);
                    results.push(`Error: ${error.message}`);
                }
            }

            // 显示结果  
            inputDisplay.innerHTML = inputs.map(input =>
                `<div style="margin: 2px 0;">${input}</div>`
            ).join('');

            outputDisplay.innerHTML = results.map(result =>
                `<div style="margin: 2px 0;" class="assembly-output">${highlightAssembly(result)}</div>`
            ).join('');

        } catch (error) {
            showError(`处理失败：${error.message}`);
            console.error('Conversion error:', error);
        } finally {
            setTimeout(() => {
                isProcessing = false;
                convertButton.disabled = false;
                convertButton.classList.remove('loading');
            }, PROCESSING_DELAY_MS);
        }
    }

    // 清空输入  
    function handleClear() {
        input.value = '';
        inputDisplay.innerHTML = '<div style="color: #666; font-style: italic;">输入的机器码将显示在这里...</div>';
        outputDisplay.innerHTML = '<div style="color: #666; font-style: italic;">反汇编结果将显示在这里...</div>';
        inputStatus.textContent = '';
        inputStatus.className = 'input-status';
        input.className = '';
        input.focus();
    }

    // 复制结果  
    function handleCopy() {
        if (!outputDisplay.textContent.trim() ||
            outputDisplay.textContent.includes('将显示在这里')) {
            showError('暂无结果可复制');
            return;
        }
        const outputText = outputDisplay.textContent;
        if (outputText && !outputText.includes('反汇编结果将显示在这里')) {
            navigator.clipboard.writeText(outputText).then(() => {
                const originalText = copyButton.innerHTML;
                copyButton.innerHTML = '<i class="fas fa-check"></i><span>已复制</span>';
                setTimeout(() => {
                    copyButton.innerHTML = originalText;
                }, 2000);
            }).catch(err => {
                showError('复制失败，请手动选择文本复制');
            });
        }
    }

    // 事件监听器  
    convertButton.addEventListener('click', handleConversion);
    clearButton.addEventListener('click', handleClear);
    copyButton.addEventListener('click', handleCopy);

    // 防抖计时器
    let inputDebounceTimer = null;

    // 输入实时验证（防抖 300 ms）
    input.addEventListener('input', () => {
        clearTimeout(inputDebounceTimer);          // 取消上一次的计时器
        inputDebounceTimer = setTimeout(() => {    // 重新计时
            const validation = validateHexInput(input.value);
            updateInputStatus(validation);
        }, 300);  // 300 ms 内没再输入才真正执行
    });

    // 键盘快捷键  
    document.addEventListener('keydown', (event) => {
        // Ctrl+Enter: 执行转换  
        if (event.ctrlKey && event.key === 'Enter') {
            event.preventDefault();
            handleConversion();
        }

        // Esc: 清空输入  
        if (event.key === 'Escape') {
            event.preventDefault();
            handleClear();
        }

        // F1: 切换快捷键提示  
        if (event.key === 'F1') {
            event.preventDefault();
            keyboardShortcutsVisible = !keyboardShortcutsVisible;
            keyboardShortcuts.classList.toggle('show', keyboardShortcutsVisible);
        }
    });

    // 初始化时聚焦到输入框  
    input.focus();

} catch (error) {
    console.error('Failed to initialize and run the WebAssembly module:', error);
    document.getElementById('outputDisplay').innerHTML =
        '<span class="assembly-error">Error loading the WebAssembly module.</span>';
}

// 全局函数供 HTML 调用  
window.toggleHelp = function () {
    const helpContent = document.getElementById('helpContent');
    helpContent.classList.toggle('show');
};

window.loadExample = function (example) {
    const input = document.getElementById('input');
    input.value = example;
    input.focus();

    // 触发输入验证  
    const event = new Event('input', { bubbles: true });
    input.dispatchEvent(event);
};