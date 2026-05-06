<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { X, Copy, Check, ArrowRightLeft, Key, ShieldCheck, AlertTriangle, Info } from "lucide-svelte";
  import { _ } from "svelte-i18n";

  const dispatch = createEventDispatcher<{ close: void }>();

  type Tab = "base64" | "jwt";

  let activeTab: Tab = $state("base64");
  let copiedField = $state<string | null>(null);
  let showHelp = $state(false);

  // Base64 state
  let base64Input = $state("");
  let base64Output = $state("");
  let base64Mode: "encode" | "decode" = $state("encode");
  let base64Error = $state("");

  // JWT state
  let jwtInput = $state("");
  let jwtHeader: Record<string, unknown> | null = $state(null);
  let jwtPayload: Record<string, unknown> | null = $state(null);
  let jwtError = $state("");
  let jwtSignatureValid: boolean | null = $state(null);

  function copyToClipboard(text: string, field: string) {
    navigator.clipboard.writeText(text);
    copiedField = field;
    setTimeout(() => (copiedField = null), 2000);
  }

  // Base64 encode/decode
  function processBase64() {
    base64Error = "";
    base64Output = "";

    if (!base64Input.trim()) return;

    try {
      if (base64Mode === "encode") {
        base64Output = btoa(unescape(encodeURIComponent(base64Input)));
      } else {
        base64Output = decodeURIComponent(escape(atob(base64Input.trim())));
      }
    } catch {
      base64Error = base64Mode === "encode"
        ? "Failed to encode — invalid input"
        : "Failed to decode — not valid Base64";
    }
  }

  function swapBase64() {
    const temp = base64Input;
    base64Input = base64Output;
    base64Output = temp;
    base64Mode = base64Mode === "encode" ? "decode" : "encode";
    base64Error = "";
  }

  // JWT decode
  function decodeJwt() {
    jwtError = "";
    jwtHeader = null;
    jwtPayload = null;
    jwtSignatureValid = null;

    const token = jwtInput.trim();
    if (!token) return;

    const parts = token.split(".");
    if (parts.length !== 3) {
      jwtError = "Invalid JWT — must have 3 parts (header.payload.signature)";
      return;
    }

    try {
      const headerJson = atob(parts[0].replace(/-/g, "+").replace(/_/g, "/"));
      jwtHeader = JSON.parse(headerJson);
    } catch {
      jwtError = "Failed to decode header";
      return;
    }

    try {
      const payloadJson = atob(parts[1].replace(/-/g, "+").replace(/_/g, "/"));
      jwtPayload = JSON.parse(payloadJson);
    } catch {
      jwtError = "Failed to decode payload";
      return;
    }

    // Check expiration
    if (jwtPayload?.exp) {
      const expMs = (jwtPayload.exp as number) * 1000;
      jwtSignatureValid = expMs > Date.now();
    }
  }

  function formatTimestamp(ts: number): string {
    const date = new Date(ts * 1000);
    const isExpired = ts * 1000 < Date.now();
    return `${date.toLocaleString()} ${isExpired ? "(expired)" : "(valid)"}`;
  }

  function handleClose() {
    dispatch("close");
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      if (showHelp) { showHelp = false; return; }
      handleClose();
    }
  }

  $effect(() => {
    if (activeTab === "base64") processBase64();
  });

  $effect(() => {
    if (activeTab === "jwt") decodeJwt();
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<div
  class="fixed inset-0 bg-black/50 flex items-center justify-center z-[20000] p-4 backdrop-blur-sm"
  role="presentation"
  onclick={(e) => { if (e.target === e.currentTarget) handleClose(); }}
>
  <div
    class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-3xl w-full max-h-[85vh] flex flex-col animate-modal-in overflow-hidden"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
  >
    <!-- Header -->
    <div class="flex items-center justify-between px-5 py-3 border-b border-gray-200 dark:border-gray-700 shrink-0">
      <div class="flex items-center gap-3">
        <div class="p-2 bg-violet-100 dark:bg-violet-900/30 rounded-lg">
          <Key class="text-violet-600 dark:text-violet-400" size={20} />
        </div>
        <div>
          <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
            Encoder / Decoder
          </h3>
          <p class="text-xs text-gray-500 dark:text-gray-400">
            Base64 Encoder/Decoder & JWT Decoder
          </p>
        </div>
      </div>

      <div class="flex items-center gap-1">
        <div class="relative">
          <button
            onclick={() => showHelp = !showHelp}
            class="p-2 text-gray-400 hover:text-violet-600 dark:hover:text-violet-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
            title="How to use"
          >
            <Info size={18} />
          </button>
          {#if showHelp}
            <div class="absolute right-0 top-full mt-2 w-80 bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-700 rounded-xl shadow-xl z-50 p-4 space-y-3 animate-modal-in">
              <div>
                <div class="flex items-center gap-1.5 mb-1">
                  <Key size={12} class="text-violet-500" />
                  <span class="text-xs font-black text-gray-800 dark:text-gray-200">Base64</span>
                </div>
                <ul class="text-[11px] text-gray-500 dark:text-gray-400 space-y-1 pl-1">
                  <li>&#8226; เลือก <b>Encode</b> เพื่อแปลงข้อความเป็น Base64</li>
                  <li>&#8226; เลือก <b>Decode</b> เพื่อแปลง Base64 กลับเป็นข้อความ</li>
                  <li>&#8226; พิมพ์แล้วผลลัพธ์จะอัปเดต <b>real-time</b> ทันที</li>
                  <li>&#8226; กดปุ่ม <ArrowRightLeft size={10} class="inline" /> เพื่อสลับ input/output</li>
                  <li>&#8226; ใช้ encode API key, token ก่อนส่งใน header/query</li>
                </ul>
              </div>
              <div class="border-t border-gray-100 dark:border-gray-800 pt-2">
                <div class="flex items-center gap-1.5 mb-1">
                  <ShieldCheck size={12} class="text-violet-500" />
                  <span class="text-xs font-black text-gray-800 dark:text-gray-200">JWT Decoder</span>
                </div>
                <ul class="text-[11px] text-gray-500 dark:text-gray-400 space-y-1 pl-1">
                  <li>&#8226; วาง JWT token ทั้ง 3 ส่วน (header.payload.signature)</li>
                  <li>&#8226; ระบบจะแตก <b>Header</b> + <b>Payload</b> ให้ทันที</li>
                  <li>&#8226; ตรวจ <b>exp</b> แล้วบอกว่า token หมดอายุหรือยัง</li>
                  <li>&#8226; <b>Key Claims</b> จะ highlight ฟิลด์สำคัญ (sub, role, user_id, exp...)</li>
                  <li>&#8226; Timestamp fields แปลงเป็นวันที่อ่านง่ายอัตโนมัติ</li>
                  <li>&#8226; กด <b>Copy</b> เพื่อคัดลอก header/payload ไปใช้ได้เลย</li>
                </ul>
              </div>
              <button
                onclick={() => showHelp = false}
                class="w-full text-center text-[10px] font-bold text-violet-500 hover:text-violet-600 pt-1"
              >
                Got it
              </button>
            </div>
          {/if}
        </div>

        <button
          onclick={handleClose}
          class="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
        >
          <X size={20} />
        </button>
      </div>
    </div>

    <!-- Tabs -->
    <div class="flex border-b border-gray-200 dark:border-gray-700 shrink-0">
      <button
        onclick={() => { activeTab = "base64"; base64Error = ""; }}
        class="flex-1 flex items-center justify-center gap-2 px-4 py-2.5 text-sm font-bold transition-colors {activeTab === 'base64'
          ? 'text-violet-600 dark:text-violet-400 border-b-2 border-violet-600 dark:border-violet-400 bg-violet-50/50 dark:bg-violet-900/10'
          : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300'}"
      >
        <Key size={15} />
        Base64
      </button>
      <button
        onclick={() => { activeTab = "jwt"; jwtError = ""; }}
        class="flex-1 flex items-center justify-center gap-2 px-4 py-2.5 text-sm font-bold transition-colors {activeTab === 'jwt'
          ? 'text-violet-600 dark:text-violet-400 border-b-2 border-violet-600 dark:border-violet-400 bg-violet-50/50 dark:bg-violet-900/10'
          : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300'}"
      >
        <ShieldCheck size={15} />
        JWT Decoder
      </button>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto p-5 space-y-4">
      {#if activeTab === "base64"}
        <!-- Mode Toggle -->
        <div class="flex items-center gap-2">
          <button
            onclick={() => { base64Mode = "encode"; processBase64(); }}
            class="px-3 py-1.5 rounded-lg text-xs font-bold transition-colors {base64Mode === 'encode'
              ? 'bg-violet-600 text-white'
              : 'bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600'}"
          >
            Encode
          </button>
          <button
            onclick={() => { base64Mode = "decode"; processBase64(); }}
            class="px-3 py-1.5 rounded-lg text-xs font-bold transition-colors {base64Mode === 'decode'
              ? 'bg-violet-600 text-white'
              : 'bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600'}"
          >
            Decode
          </button>

          <button
            onclick={swapBase64}
            class="ml-auto p-1.5 text-gray-400 hover:text-violet-600 dark:hover:text-violet-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
            title="Swap input/output"
          >
            <ArrowRightLeft size={14} />
          </button>
        </div>

        <!-- Input -->
        <div>
          <label for="base64-input" class="block text-xs font-bold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-1.5">
            {base64Mode === "encode" ? "Plain Text" : "Base64 String"}
          </label>
          <textarea
            id="base64-input"
            bind:value={base64Input}
            oninput={() => processBase64()}
            rows="4"
            class="w-full bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-xl px-4 py-3 text-sm font-mono text-gray-800 dark:text-gray-200 outline-none placeholder-gray-400 resize-y focus:border-violet-400 dark:focus:border-violet-600 transition-colors"
            placeholder={base64Mode === "encode" ? "Enter text to encode..." : "Enter Base64 string to decode..."}
          ></textarea>
        </div>

        <!-- Error -->
        {#if base64Error}
          <div class="flex items-center gap-2 px-3 py-2 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-xl">
            <AlertTriangle size={14} class="text-red-500 shrink-0" />
            <span class="text-xs text-red-600 dark:text-red-400 font-medium">{base64Error}</span>
          </div>
        {/if}

        <!-- Output -->
        <div>
          <div class="flex items-center justify-between mb-1.5">
            <p class="block text-xs font-bold text-gray-500 dark:text-gray-400 uppercase tracking-wider">
              {base64Mode === "encode" ? "Base64 Output" : "Decoded Output"}
            </p>
            {#if base64Output}
              <button
                onclick={() => copyToClipboard(base64Output, "base64-output")}
                class="flex items-center gap-1 px-2 py-0.5 text-[10px] font-bold text-gray-400 hover:text-violet-600 dark:hover:text-violet-400 transition-colors"
              >
                {#if copiedField === "base64-output"}
                  <Check size={10} class="text-green-500" />
                  Copied
                {:else}
                  <Copy size={10} />
                  Copy
                {/if}
              </button>
            {/if}
          </div>
          <div class="w-full bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-xl px-4 py-3 text-sm font-mono text-gray-800 dark:text-gray-200 min-h-[80px] break-all whitespace-pre-wrap">
            {#if base64Output}
              {base64Output}
            {:else}
              <span class="text-gray-400">Output will appear here...</span>
            {/if}
          </div>
        </div>

      {:else if activeTab === "jwt"}
        <!-- JWT Input -->
        <div>
          <label for="jwt-input" class="block text-xs font-bold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-1.5">
            JWT Token
          </label>
          <textarea
            id="jwt-input"
            bind:value={jwtInput}
            oninput={() => decodeJwt()}
            rows="3"
            class="w-full bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-xl px-4 py-3 text-sm font-mono text-gray-800 dark:text-gray-200 outline-none placeholder-gray-400 resize-y focus:border-violet-400 dark:focus:border-violet-600 transition-colors"
            placeholder="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c"
          ></textarea>
        </div>

        <!-- Error -->
        {#if jwtError}
          <div class="flex items-center gap-2 px-3 py-2 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-xl">
            <AlertTriangle size={14} class="text-red-500 shrink-0" />
            <span class="text-xs text-red-600 dark:text-red-400 font-medium">{jwtError}</span>
          </div>
        {/if}

        {#if jwtHeader && jwtPayload}
          <!-- Expiration Status -->
          {#if jwtPayload.exp}
            <div class="flex items-center gap-2 px-3 py-2 rounded-xl border {jwtSignatureValid
              ? 'bg-green-50 dark:bg-green-900/20 border-green-200 dark:border-green-800'
              : 'bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800'}">
              {#if jwtSignatureValid}
                <ShieldCheck size={14} class="text-green-500 shrink-0" />
                <span class="text-xs text-green-700 dark:text-green-300 font-bold">Token is valid (not expired)</span>
              {:else}
                <AlertTriangle size={14} class="text-red-500 shrink-0" />
                <span class="text-xs text-red-700 dark:text-red-300 font-bold">Token has expired</span>
              {/if}
            </div>
          {/if}

          <!-- Header -->
          <div>
            <div class="flex items-center justify-between mb-1.5">
              <p class="block text-xs font-bold text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                Header
              </p>
              <button
                onclick={() => copyToClipboard(JSON.stringify(jwtHeader, null, 2), "jwt-header")}
                class="flex items-center gap-1 px-2 py-0.5 text-[10px] font-bold text-gray-400 hover:text-violet-600 dark:hover:text-violet-400 transition-colors"
              >
                {#if copiedField === "jwt-header"}
                  <Check size={10} class="text-green-500" />
                  Copied
                {:else}
                  <Copy size={10} />
                  Copy
                {/if}
              </button>
            </div>
            <pre class="bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-xl px-4 py-3 text-xs font-mono text-gray-800 dark:text-gray-200 overflow-x-auto">{JSON.stringify(jwtHeader, null, 2)}</pre>
          </div>

          <!-- Payload -->
          <div>
            <div class="flex items-center justify-between mb-1.5">
              <p class="block text-xs font-bold text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                Payload
              </p>
              <button
                onclick={() => copyToClipboard(JSON.stringify(jwtPayload, null, 2), "jwt-payload")}
                class="flex items-center gap-1 px-2 py-0.5 text-[10px] font-bold text-gray-400 hover:text-violet-600 dark:hover:text-violet-400 transition-colors"
              >
                {#if copiedField === "jwt-payload"}
                  <Check size={10} class="text-green-500" />
                  Copied
                {:else}
                  <Copy size={10} />
                  Copy
                {/if}
              </button>
            </div>
            <pre class="bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-xl px-4 py-3 text-xs font-mono text-gray-800 dark:text-gray-200 overflow-x-auto">{JSON.stringify(jwtPayload, null, 2)}</pre>
          </div>

          <!-- Decoded Claims -->
          <div>
            <p class="block text-xs font-bold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-2">
              Key Claims
            </p>
            <div class="space-y-1.5">
              {#each Object.entries(jwtPayload) as [key, value]}
                {@const isKnown = ["exp", "iat", "nbf", "sub", "iss", "aud", "jti", "role", "user_id", "email", "name", "scope", "permissions"].includes(key)}
                <div class="flex items-start gap-2 px-3 py-2 rounded-lg {isKnown
                  ? 'bg-violet-50 dark:bg-violet-900/10 border border-violet-200 dark:border-violet-800'
                  : 'bg-gray-50 dark:bg-gray-900/30 border border-gray-200 dark:border-gray-700'}">
                  <span class="text-[10px] font-black uppercase tracking-wider shrink-0 pt-0.5 {isKnown
                    ? 'text-violet-500 dark:text-violet-400'
                    : 'text-gray-400 dark:text-gray-500'}" style="min-width: 60px">
                    {key}
                  </span>
                  <span class="text-xs font-mono text-gray-800 dark:text-gray-200 break-all flex-1">
                    {#if (key === "exp" || key === "iat" || key === "nbf") && typeof value === "number"}
                      {formatTimestamp(value)}
                    {:else}
                      {typeof value === "object" ? JSON.stringify(value) : String(value)}
                    {/if}
                  </span>
                </div>
              {/each}
            </div>
          </div>
        {/if}
      {/if}
    </div>

    <!-- Footer -->
    <div class="px-5 py-2 border-t border-gray-100 dark:border-gray-700 flex justify-between items-center bg-gray-50/50 dark:bg-gray-800/50 shrink-0">
      <div class="flex items-center gap-2">
        <span class="inline-block w-2 h-2 rounded-full {activeTab === 'base64' ? 'bg-violet-500' : 'bg-emerald-500'}"></span>
        <span class="text-[10px] text-gray-400 uppercase tracking-widest font-medium">
          {activeTab === "base64" ? "Base64 Mode" : "JWT Mode"}
        </span>
      </div>
      <div class="text-[10px] text-gray-400">
        {activeTab === "base64" ? "Real-time encode/decode" : "Decode & inspect token claims"}
      </div>
    </div>
  </div>
</div>

<style>
  @keyframes modal-in {
    from {
      opacity: 0;
      transform: scale(0.95) translateY(-10px);
    }
    to {
      opacity: 1;
      transform: scale(1) translateY(0);
    }
  }

  .animate-modal-in {
    animation: modal-in 0.2s ease-out;
  }
</style>
