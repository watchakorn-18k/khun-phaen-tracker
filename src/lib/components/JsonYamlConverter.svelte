<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";
  import {
    Braces, X, Copy, Check, ArrowLeftRight, AlertCircle,
    FileJson, FileText, Table2, Code2, Palette, Clock, Hash,
  } from "lucide-svelte";
  import { browser } from "$app/environment";

  const dispatch = createEventDispatcher<{ close: void }>();
  type Tab = "jsonYaml" | "jsonCsv" | "jsonModel" | "color" | "timestamp" | "numbase";
  let activeTab = $state<Tab>("jsonYaml");

  // ─── JSON/YAML ──────────────────────────────────
  type FormatYaml = "json" | "yaml";
  let jyInput = $state("");
  let jyInputFormat = $state<FormatYaml>("json");
  let jyOutputFormat = $state<FormatYaml>("yaml");
  let jyCopied = $state(false);

  let jyResult = $derived.by(() => {
    try {
      if (!jyInput.trim()) return { output: "", error: "" };
      if (jyInputFormat === "json" && jyOutputFormat === "yaml") {
        return { output: jsonToYaml(JSON.parse(jyInput), 0), error: "" };
      }
      if (jyInputFormat === "yaml" && jyOutputFormat === "json") {
        return { output: JSON.stringify(yamlToJson(jyInput), null, 2), error: "" };
      }
      if (jyInputFormat === "json" && jyOutputFormat === "json") {
        return { output: JSON.stringify(JSON.parse(jyInput), null, 2), error: "" };
      }
      return { output: jyInput, error: "" };
    } catch (e: any) { return { output: "", error: e.message }; }
  });

  // ─── JSON/CSV ───────────────────────────────────
  type FormatCsv = "json" | "csv";
  let jcInput = $state("");
  let jcInputFormat = $state<FormatCsv>("json");
  let jcOutputFormat = $state<FormatCsv>("csv");
  let jcCopied = $state(false);

  let jcResult = $derived.by(() => {
    try {
      if (!jcInput.trim()) return { output: "", error: "" };
      if (jcInputFormat === "json" && jcOutputFormat === "csv") {
        const parsed = JSON.parse(jcInput);
        const arr = Array.isArray(parsed) ? parsed : [parsed];
        if (arr.length === 0) return { output: "", error: "" };
        const keys = [...new Set(arr.flatMap((o: any) => Object.keys(o || {})))];
        const header = keys.join(",");
        const rows = arr.map((o: any) => keys.map((k) => csvEscape(String(o?.[k] ?? ""))).join(","));
        return { output: [header, ...rows].join("\n"), error: "" };
      }
      if (jcInputFormat === "csv" && jcOutputFormat === "json") {
        const lines = jcInput.split("\n").filter((l: string) => l.trim());
        if (lines.length < 2) return { output: "[]", error: "" };
        const headers = csvParseLine(lines[0]);
        const rows = lines.slice(1).map((line: string) => {
          const vals = csvParseLine(line);
          const obj: Record<string, string> = {};
          headers.forEach((h: string, i: number) => obj[h] = vals[i] || "");
          return obj;
        });
        return { output: JSON.stringify(rows, null, 2), error: "" };
      }
      return { output: jcInput, error: "" };
    } catch (e: any) { return { output: "", error: e.message }; }
  });

  // ─── JSON to Model ──────────────────────────────
  type Lang = "swift" | "kotlin" | "dart" | "go" | "rust" | "typescript" | "python" | "lua";
  let jmInput = $state("");
  let jmLang = $state<Lang>("swift");
  let jmClassName = $state("MyModel");
  let jmCopied = $state(false);

  let jmOutput = $derived.by(() => {
    if (!jmInput.trim()) return "";
    try {
      const parsed = JSON.parse(jmInput);
      const obj = Array.isArray(parsed) ? parsed[0] : parsed;
      if (!obj || typeof obj !== "object") return "";
      if (jmLang === "swift") return genSwift(obj, jmClassName);
      if (jmLang === "kotlin") return genKotlin(obj, jmClassName);
      if (jmLang === "dart") return genDart(obj, jmClassName);
      if (jmLang === "go") return genGo(obj, jmClassName);
      if (jmLang === "rust") return genRust(obj, jmClassName);
      if (jmLang === "typescript") return genTypeScript(obj, jmClassName);
      if (jmLang === "python") return genPython(obj, jmClassName);
      return genLua(obj, jmClassName);
    } catch { return "// Invalid JSON"; }
  });

  // ─── Color Converter ────────────────────────────
  let colorHex = $state("#6366f1");
  let colorCopied = $state("");

  let colorRgb = $derived({ r: parseInt(colorHex.slice(1, 3), 16), g: parseInt(colorHex.slice(3, 5), 16), b: parseInt(colorHex.slice(5, 7), 16) });

  let colorHsl = $derived.by(() => {
    const { r, g, b } = colorRgb;
    const rn = r / 255, gn = g / 255, bn = b / 255;
    const max = Math.max(rn, gn, bn), min = Math.min(rn, gn, bn);
    const l = (max + min) / 2;
    if (max === min) return { h: 0, s: 0, l: Math.round(l * 100) };
    const d = max - min, s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
    let h = 0;
    if (max === rn) h = ((gn - bn) / d + (gn < bn ? 6 : 0)) / 6;
    else if (max === gn) h = ((bn - rn) / d + 2) / 6;
    else h = ((rn - gn) / d + 4) / 6;
    return { h: Math.round(h * 360), s: Math.round(s * 100), l: Math.round(l * 100) };
  });

  let colorFormats = $derived([
    { label: "HEX", value: colorHex.toUpperCase() },
    { label: "RGB", value: `rgb(${colorRgb.r}, ${colorRgb.g}, ${colorRgb.b})` },
    { label: "RGBA", value: `rgba(${colorRgb.r}, ${colorRgb.g}, ${colorRgb.b}, 1.0)` },
    { label: "HSL", value: `hsl(${colorHsl.h}, ${colorHsl.s}%, ${colorHsl.l}%)` },
    { label: "Swift UIColor", value: `UIColor(red: ${(colorRgb.r / 255).toFixed(3)}, green: ${(colorRgb.g / 255).toFixed(3)}, blue: ${(colorRgb.b / 255).toFixed(3)}, alpha: 1.0)` },
    { label: "SwiftUI Color", value: `Color(red: ${(colorRgb.r / 255).toFixed(3)}, green: ${(colorRgb.g / 255).toFixed(3)}, blue: ${(colorRgb.b / 255).toFixed(3)})` },
    { label: "Kotlin Color", value: `Color(${colorRgb.r}, ${colorRgb.g}, ${colorRgb.b})` },
    { label: "Flutter Color", value: `Color(0xFF${colorHex.slice(1).toUpperCase()})` },
  ]);

  // ─── YAML parser/generator ──────────────────────
  function jsonToYaml(obj: any, indent: number): string {
    const pad = "  ".repeat(indent);
    if (obj === null || obj === undefined) return "null";
    if (typeof obj === "boolean") return String(obj);
    if (typeof obj === "number") return String(obj);
    if (typeof obj === "string") {
      if (/[:\n#"']/.test(obj) || obj === "" || obj === "true" || obj === "null" || /^\d/.test(obj) || obj.startsWith(" "))
        return `"${obj.replace(/\\/g, "\\\\").replace(/"/g, '\\"').replace(/\n/g, "\\n")}"`;
      return obj;
    }
    if (Array.isArray(obj)) {
      if (obj.length === 0) return "[]";
      return obj.map((item) => {
        const val = jsonToYaml(item, indent + 1);
        return typeof item === "object" && item !== null
          ? `${pad}- ${val.trimStart()}`
          : `${pad}- ${val}`;
      }).join("\n");
    }
    const entries = Object.entries(obj);
    if (entries.length === 0) return "{}";
    return entries.map(([key, val]) => {
      const yamlVal = jsonToYaml(val, indent + 1);
      if (typeof val === "object" && val !== null && ((Array.isArray(val) && val.length > 0) || (!Array.isArray(val) && Object.keys(val).length > 0)))
        return `${pad}${key}:\n${yamlVal}`;
      return `${pad}${key}: ${yamlVal}`;
    }).join("\n");
  }

  function yamlToJson(yaml: string): any {
    return parseYaml(yaml.split("\n"), 0, 0).value;
  }

  function getIndent(line: string) { const m = line.match(/^( *)/); return m ? m[1].length : 0; }

  function parseYamlValue(v: string): any {
    const t = v.trim();
    if (t === "null" || t === "~" || t === "") return null;
    if (t === "true") return true;
    if (t === "false") return false;
    if (/^-?\d+$/.test(t)) return parseInt(t, 10);
    if (/^-?\d+\.\d+$/.test(t)) return parseFloat(t);
    if (t.startsWith('"') && t.endsWith('"')) return t.slice(1, -1).replace(/\\"/g, '"').replace(/\\n/g, "\n").replace(/\\\\/g, "\\");
    if (t.startsWith("'") && t.endsWith("'")) return t.slice(1, -1);
    if (t === "[]") return [];
    if (t === "{}") return {};
    return t;
  }

  function parseYaml(lines: string[], si: number, bi: number): { value: any; nextIdx: number } {
    if (si >= lines.length) return { value: null, nextIdx: si };
    const fl = lines[si];
    if (fl.trim() === "" || fl.trim().startsWith("#")) return parseYaml(lines, si + 1, bi);

    if (fl.trim().startsWith("- ")) {
      const arr: any[] = [];
      let i = si;
      while (i < lines.length) {
        const line = lines[i];
        if (line.trim() === "" || line.trim().startsWith("#")) { i++; continue; }
        const indent = getIndent(line);
        if (indent < bi) break;
        if (indent === bi && line.trim().startsWith("- ")) {
          const after = line.trim().slice(2);
          if (after.includes(": ")) {
            const obj: Record<string, any> = {};
            const ci = after.indexOf(": ");
            const key = after.slice(0, ci).trim(), val = after.slice(ci + 2).trim();
            if (val === "" || val === "|" || val === ">") {
              const n = parseYaml(lines, i + 1, indent + 2); obj[key] = n.value; i = n.nextIdx;
            } else { obj[key] = parseYamlValue(val); i++; }
            while (i < lines.length) {
              const nl = lines[i];
              if (nl.trim() === "" || nl.trim().startsWith("#")) { i++; continue; }
              if (getIndent(nl) <= indent) break;
              const n = parseYaml(lines, i, getIndent(nl));
              if (typeof n.value === "object" && n.value !== null && !Array.isArray(n.value)) Object.assign(obj, n.value);
              i = n.nextIdx;
            }
            arr.push(obj);
          } else { arr.push(parseYamlValue(after)); i++; }
        } else break;
      }
      return { value: arr, nextIdx: i };
    }

    const obj: Record<string, any> = {};
    let i = si;
    while (i < lines.length) {
      const line = lines[i];
      if (line.trim() === "" || line.trim().startsWith("#")) { i++; continue; }
      const indent = getIndent(line);
      if (indent < bi) break;
      if (indent > bi) { const n = parseYaml(lines, i, indent); i = n.nextIdx; continue; }
      const cm = line.match(/^ *([^:]+):\s*(.*)/);
      if (cm) {
        const key = cm[1].trim(), val = cm[2].trim();
        if (val === "" || val === "|" || val === ">") {
          const n = parseYaml(lines, i + 1, getIndent(line) + 2); obj[key] = n.value; i = n.nextIdx;
        } else { obj[key] = parseYamlValue(val); i++; }
      } else i++;
    }
    return { value: obj, nextIdx: i };
  }

  // ─── CSV helpers ────────────────────────────────
  function csvEscape(v: string) {
    return (v.includes(",") || v.includes('"') || v.includes("\n")) ? `"${v.replace(/"/g, '""')}"` : v;
  }
  function csvParseLine(line: string): string[] {
    const r: string[] = []; let cur = "", inQ = false;
    for (let i = 0; i < line.length; i++) {
      const c = line[i];
      if (inQ) {
        if (c === '"' && line[i + 1] === '"') { cur += '"'; i++; }
        else if (c === '"') inQ = false;
        else cur += c;
      } else {
        if (c === '"') inQ = true;
        else if (c === ",") { r.push(cur); cur = ""; }
        else cur += c;
      }
    }
    r.push(cur); return r;
  }

  // ─── Model generators ───────────────────────────
  function inferType(val: any): string {
    if (val === null || val === undefined) return "String?";
    if (typeof val === "boolean") return "Bool";
    if (typeof val === "number") return Number.isInteger(val) ? "Int" : "Double";
    if (typeof val === "string") return /^\d{4}-\d{2}-\d{2}/.test(val) ? "Date" : "String";
    if (Array.isArray(val)) return "[Any]";
    return "Object";
  }

  function genSwift(obj: Record<string, any>, name: string): string {
    const lines = [`struct ${name}: Codable {`];
    for (const [k, v] of Object.entries(obj)) {
      const t = inferType(v);
      lines.push(`    let ${k}: ${t}${t.endsWith("?") ? "" : "?"}`);
    }
    lines.push("}");
    return lines.join("\n");
  }

  function genKotlin(obj: Record<string, any>, name: string): string {
    const lines = [`data class ${name}(`];
    const entries = Object.entries(obj);
    entries.forEach(([k, v], i) => {
      let t = inferType(v).replace("Bool", "Boolean").replace("Date", "String").replace("[Any]", "List<Any>").replace("Object", "Any");
      if (!t.endsWith("?") && v === null) t += "?";
      lines.push(`    val ${k}: ${t}${i < entries.length - 1 ? "," : ""}`);
    });
    lines.push(")");
    return lines.join("\n");
  }

  function genDart(obj: Record<string, any>, name: string): string {
    const lines = [`class ${name} {`];
    for (const [k, v] of Object.entries(obj)) {
      let t = inferType(v).replace("Bool", "bool").replace("Int", "int").replace("Double", "double").replace("Date", "DateTime").replace("[Any]", "List<dynamic>").replace("Object", "Map<String, dynamic>");
      if (v === null && !t.endsWith("?")) t += "?";
      lines.push(`  final ${t} ${k};`);
    }
    lines.push("", `  ${name}({`);
    for (const k of Object.keys(obj)) lines.push(`    required this.${k},`);
    lines.push("  });", "", `  factory ${name}.fromJson(Map<String, dynamic> json) => ${name}(`);
    for (const [k, v] of Object.entries(obj)) {
      lines.push(inferType(v) === "Date" ? `    ${k}: DateTime.parse(json['${k}']),` : `    ${k}: json['${k}'],`);
    }
    lines.push("  );", "}");
    return lines.join("\n");
  }

  function genGo(obj: Record<string, any>, name: string): string {
    const lines = [`type ${name} struct {`];
    for (const [k, v] of Object.entries(obj)) {
      let t = inferType(v);
      if (t === "String?" || t === "String") t = "string";
      else if (t === "Int") t = "int";
      else if (t === "Double") t = "float64";
      else if (t === "Bool") t = "bool";
      else if (t === "Date") t = "time.Time";
      else if (t === "[Any]") t = "[]any";
      else if (t === "Object") t = "map[string]any";
      else t = "any";
      const tag = k.slice(0, 1).toUpperCase() + k.slice(1);
      lines.push(`\t${tag} ${t} \`json:"${k}"\``);
    }
    lines.push("}");
    return lines.join("\n");
  }

  function genRust(obj: Record<string, any>, name: string): string {
    const lines = ["use serde::{Serialize, Deserialize};", "", `#[derive(Serialize, Deserialize, Debug)]`, `pub struct ${name} {`];
    for (const [k, v] of Object.entries(obj)) {
      let t = inferType(v);
      if (t === "String?" || t === "String") t = v === null ? "Option<String>" : "String";
      else if (t === "Int") t = v === null ? "Option<i64>" : "i64";
      else if (t === "Double") t = v === null ? "Option<f64>" : "f64";
      else if (t === "Bool") t = v === null ? "Option<bool>" : "bool";
      else if (t === "Date") t = "String";
      else if (t === "[Any]") t = "Vec<serde_json::Value>";
      else if (t === "Object") t = "serde_json::Map<String, serde_json::Value>";
      else t = "serde_json::Value";
      lines.push(`    pub ${k}: ${t},`);
    }
    lines.push("}");
    return lines.join("\n");
  }

  function genTypeScript(obj: Record<string, any>, name: string): string {
    const lines = [`export interface ${name} {`];
    for (const [k, v] of Object.entries(obj)) {
      let t = inferType(v);
      if (t === "String?" || t === "String") t = "string";
      else if (t === "Int" || t === "Double") t = "number";
      else if (t === "Bool") t = "boolean";
      else if (t === "Date") t = "string";
      else if (t === "[Any]") t = "any[]";
      else if (t === "Object") t = "Record<string, any>";
      else t = "any";
      const opt = v === null ? "?" : "";
      lines.push(`  ${k}${opt}: ${t};`);
    }
    lines.push("}");
    return lines.join("\n");
  }

  function genPython(obj: Record<string, any>, name: string): string {
    const lines = ["from dataclasses import dataclass", "from typing import Any, Optional", "", `@dataclass`, `class ${name}:`];
    for (const [k, v] of Object.entries(obj)) {
      let t = inferType(v);
      if (t === "String?" || t === "String") t = "str";
      else if (t === "Int") t = "int";
      else if (t === "Double") t = "float";
      else if (t === "Bool") t = "bool";
      else if (t === "Date") t = "str";
      else if (t === "[Any]") t = "list[Any]";
      else if (t === "Object") t = "dict[str, Any]";
      else t = "Any";
      const def = v === null ? " = None" : "";
      lines.push(`    ${k}: ${t}${def}`);
    }
    return lines.join("\n");
  }

  function genLua(obj: Record<string, any>, name: string): string {
    const lines = [`local ${name} = {}`, `${name}.__index = ${name}`, "", `function ${name}.new(data)`, "    local self = setmetatable({}, ${name})"];
    for (const [k, v] of Object.entries(obj)) {
      if (v === null) lines.push(`    self.${k} = data.${k} or nil`);
      else if (typeof v === "string") lines.push(`    self.${k} = data.${k} or ""`);
      else if (typeof v === "number") lines.push(`    self.${k} = data.${k} or 0`);
      else if (typeof v === "boolean") lines.push(`    self.${k} = data.${k} or false`);
      else lines.push(`    self.${k} = data.${k} or {}`);
    }
    lines.push("    return self", "end", "", `return ${name}`);
    return lines.join("\n");
  }

  // ─── Timestamp Converter ──────────────────────
  let tsInput = $state("");
  let tsCopied = $state("");

  function relativeTime(date: Date): string {
    const now = Date.now();
    const diff = now - date.getTime();
    const abs = Math.abs(diff);
    const suffix = diff > 0 ? "ago" : "from now";
    if (abs < 60_000) return `${Math.floor(abs / 1000)}s ${suffix}`;
    if (abs < 3_600_000) return `${Math.floor(abs / 60_000)}m ${suffix}`;
    if (abs < 86_400_000) return `${Math.floor(abs / 3_600_000)}h ${suffix}`;
    if (abs < 2_592_000_000) return `${Math.floor(abs / 86_400_000)}d ${suffix}`;
    return `${(abs / 2_592_000_000).toFixed(1)}mo ${suffix}`;
  }

  let tsParsed = $derived.by(() => {
    const raw = tsInput.trim();
    if (!raw) return null;
    let date: Date | null = null;
    // epoch seconds
    if (/^\d{10}$/.test(raw)) date = new Date(Number(raw) * 1000);
    // epoch milliseconds
    else if (/^\d{13}$/.test(raw)) date = new Date(Number(raw));
    // epoch microseconds
    else if (/^\d{16}$/.test(raw)) date = new Date(Number(raw) / 1000);
    // ISO string
    else {
      const d = new Date(raw);
      if (!isNaN(d.getTime())) date = d;
    }
    if (!date) return null;
    return {
      iso: date.toISOString(),
      isoLocal: date.toLocaleString("sv-SE", { dateStyle: "long", timeStyle: "medium" }),
      epochSec: Math.floor(date.getTime() / 1000),
      epochMs: date.getTime(),
      relative: relativeTime(date),
      utc: date.toUTCString(),
      rfc2822: date.toUTCString().replace("GMT", "+0000"),
      dayOfWeek: date.toLocaleDateString("en-US", { weekday: "long" }),
      dateOnly: date.toISOString().slice(0, 10),
      timeOnly: date.toISOString().slice(11, 19),
      unixCmd: `date -d @${Math.floor(date.getTime() / 1000)}`,
    };
  });

  function setTsNow() {
    tsInput = String(Math.floor(Date.now() / 1000));
  }

  // ─── Number Base Converter ─────────────────────
  let nbInput = $state("255");
  let nbBase = $state<2 | 8 | 10 | 16>(10);
  let nbCopied = $state("");

  let nbParsed = $derived.by(() => {
    const raw = nbInput.trim();
    if (!raw) return null;
    let dec: number;
    try {
      dec = parseInt(raw, nbBase);
      if (isNaN(dec)) return null;
    } catch { return null; }
    const hex = dec.toString(16).toUpperCase();
    const decStr = dec.toString(10);
    const oct = dec.toString(8);
    const bin = dec.toString(2);
    return {
      hex: hex,
      hexPrefixed: `0x${hex}`,
      dec: decStr,
      oct: oct,
      octPrefixed: `0o${oct}`,
      bin: bin,
      binPrefixed: `0b${bin}`,
      char: dec >= 32 && dec <= 126 ? `'${String.fromCharCode(dec)}'` : dec >= 0 && dec <= 0x10FFFF ? `U+${hex.padStart(4, "0")}` : "",
      bits: bin.length,
      byteHex: dec >= 0 && dec <= 255 ? `0x${hex.padStart(2, "0")}` : "",
    };
  });

  const nbQuickValues = [
    { label: "255", base: 10 as const, value: "255" },
    { label: "0xFF", base: 16 as const, value: "FF" },
    { label: "0b11111111", base: 2 as const, value: "11111111" },
    { label: "16", base: 10 as const, value: "16" },
    { label: "0x1A", base: 16 as const, value: "1A" },
    { label: "127", base: 10 as const, value: "127" },
    { label: "65535", base: 10 as const, value: "65535" },
    { label: "0o777", base: 8 as const, value: "777" },
  ];

  // ─── Lifecycle ──────────────────────────────────
  function handleKeydown(e: KeyboardEvent) { if (e.key === "Escape") handleClose(); }
  function handleClose() { dispatch("close"); }
  function copyText(text: string, setter: (v: boolean) => void) {
    navigator.clipboard.writeText(text); setter(true); setTimeout(() => setter(false), 2000);
  }

  onMount(() => {
    if (!browser) return;
    const s = localStorage.getItem("conv-jy"); if (s) jyInput = s;
    const c = localStorage.getItem("conv-jc"); if (c) jcInput = c;
    const m = localStorage.getItem("conv-jm"); if (m) jmInput = m;
  });

  $effect(() => { if (browser) localStorage.setItem("conv-jy", jyInput); });
  $effect(() => { if (browser) localStorage.setItem("conv-jc", jcInput); });
  $effect(() => { if (browser) localStorage.setItem("conv-jm", jmInput); });

  const tabs = [
    { id: "jsonYaml" as Tab, label: "JSON / YAML", icon: FileJson },
    { id: "jsonCsv" as Tab, label: "JSON / CSV", icon: Table2 },
    { id: "jsonModel" as Tab, label: "JSON to Model", icon: Code2 },
    { id: "color" as Tab, label: "Color", icon: Palette },
    { id: "timestamp" as Tab, label: "Timestamp", icon: Clock },
    { id: "numbase" as Tab, label: "Number Base", icon: Hash },
  ];

  const presetColors = [
    "#EF4444","#F97316","#F59E0B","#EAB308","#84CC16","#22C55E",
    "#10B981","#14B8A6","#06B6D4","#0EA5E9","#3B82F6","#6366F1",
    "#8B5CF6","#A855F7","#D946EF","#EC4899","#000000","#374151",
    "#6B7280","#9CA3AF","#D1D5DB","#F3F4F6","#FFFFFF","#92400E",
  ];
</script>

{#if browser}
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-[20000] p-4 backdrop-blur-sm"
    role="presentation" onclick={(e) => { if (e.target === e.currentTarget) handleClose(); }}>
    <div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-6xl w-full h-[85vh] flex flex-col animate-modal-in overflow-hidden"
      role="dialog" aria-modal="true" tabindex="-1" onkeydown={handleKeydown}>

      <!-- Header -->
      <div class="flex items-center justify-between px-5 py-3 border-b border-gray-200 dark:border-gray-700 shrink-0">
        <div class="flex items-center gap-3">
          <div class="p-2 bg-emerald-100 dark:bg-emerald-900/30 rounded-lg">
            <Braces class="text-emerald-600 dark:text-emerald-400" size={20} />
          </div>
          <div>
            <h3 class="text-lg font-semibold text-gray-900 dark:text-white">Converter</h3>
            <p class="text-xs text-gray-500 dark:text-gray-400">JSON, YAML, CSV, Model, Color, Timestamp, NumBase</p>
          </div>
        </div>
        <button onclick={handleClose} class="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg">
          <X size={20} />
        </button>
      </div>

      <!-- Tabs -->
      <div class="flex items-center gap-1 px-5 py-2 border-b border-gray-100 dark:border-gray-700 bg-gray-50/50 dark:bg-gray-800/50 shrink-0 overflow-x-auto">
        {#each tabs as tab}
          <button
            onclick={() => (activeTab = tab.id)}
            class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-bold whitespace-nowrap transition-all {activeTab === tab.id
              ? 'bg-emerald-100 dark:bg-emerald-900/30 text-emerald-600 dark:text-emerald-400 border border-emerald-300 dark:border-emerald-700'
              : 'text-gray-500 hover:text-gray-700 dark:hover:text-gray-300'}"
          >
            <tab.icon size={14} />
            {tab.label}
          </button>
        {/each}
      </div>

      <!-- Tab Content -->
      <div class="flex-1 overflow-hidden">

        <!-- ═══ JSON/YAML ═══ -->
        {#if activeTab === "jsonYaml"}
          <div class="h-full flex">
            <div class="flex flex-col overflow-hidden flex-1 border-r border-gray-200 dark:border-gray-700">
              <div class="flex items-center gap-2 px-4 py-2 bg-gray-50 dark:bg-gray-900/30 border-b border-gray-200 dark:border-gray-700 shrink-0">
                {#if jyInputFormat === "json"}<FileJson size={14} class="text-amber-500" />{:else}<FileText size={14} class="text-blue-500" />{/if}
                <select bind:value={jyInputFormat} onchange={() => (jyOutputFormat = jyInputFormat === "json" ? "yaml" : "json")}
                  class="bg-transparent text-[11px] font-bold text-gray-500 uppercase tracking-wider outline-none cursor-pointer">
                  <option value="json">JSON</option><option value="yaml">YAML</option>
                </select>
                <button onclick={() => { try { jyInput = JSON.stringify(JSON.parse(jyInput)); } catch {} }}
                  class="ml-auto text-[10px] font-bold text-gray-400 hover:text-gray-600 px-2 py-0.5 rounded bg-gray-100 dark:bg-gray-700">Minify</button>
                <button onclick={() => { if (jyResult.output && !jyResult.error) jyInput = jyResult.output; const t = jyInputFormat; jyInputFormat = jyOutputFormat; jyOutputFormat = t; }}
                  class="text-gray-400 hover:text-emerald-500"><ArrowLeftRight size={14} /></button>
              </div>
              <textarea bind:value={jyInput} class="flex-1 bg-transparent text-sm font-mono text-gray-800 dark:text-gray-200 p-4 resize-none outline-none placeholder-gray-400"
                placeholder={'{\n  "key": "value"\n}'} spellcheck="false"></textarea>
            </div>
            <div class="flex flex-col overflow-hidden flex-1">
              <div class="flex items-center gap-2 px-4 py-2 bg-gray-50 dark:bg-gray-900/30 border-b border-gray-200 dark:border-gray-700 shrink-0">
                <span class="text-[11px] font-bold text-emerald-500 uppercase tracking-wider">{jyOutputFormat.toUpperCase()}</span>
                <button onclick={() => copyText(jyResult.output, (v) => jyCopied = v)} class="ml-auto text-gray-400 hover:text-emerald-500">
                  {#if jyCopied}<Check size={14} class="text-green-500" />{:else}<Copy size={14} />{/if}
                </button>
              </div>
              <div class="flex-1 overflow-y-auto">
                {#if jyResult.error}
                  <div class="p-4"><div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-xl px-4 py-3 text-sm text-red-600 dark:text-red-400 font-mono whitespace-pre-wrap">{jyResult.error}</div></div>
                {:else}
                  <pre class="p-4 text-sm font-mono text-gray-800 dark:text-gray-200 whitespace-pre-wrap break-all leading-relaxed">{jyResult.output || ''}</pre>
                {/if}
              </div>
            </div>
          </div>

        <!-- ═══ JSON/CSV ═══ -->
        {:else if activeTab === "jsonCsv"}
          <div class="h-full flex">
            <div class="flex flex-col overflow-hidden flex-1 border-r border-gray-200 dark:border-gray-700">
              <div class="flex items-center gap-2 px-4 py-2 bg-gray-50 dark:bg-gray-900/30 border-b border-gray-200 dark:border-gray-700 shrink-0">
                {#if jcInputFormat === "json"}<FileJson size={14} class="text-amber-500" />{:else}<Table2 size={14} class="text-green-500" />{/if}
                <select bind:value={jcInputFormat} onchange={() => (jcOutputFormat = jcInputFormat === "json" ? "csv" : "json")}
                  class="bg-transparent text-[11px] font-bold text-gray-500 uppercase tracking-wider outline-none cursor-pointer">
                  <option value="json">JSON</option><option value="csv">CSV</option>
                </select>
                <button onclick={() => { if (jcResult.output && !jcResult.error) jcInput = jcResult.output; const t = jcInputFormat; jcInputFormat = jcOutputFormat; jcOutputFormat = t; }}
                  class="ml-auto text-gray-400 hover:text-cyan-500"><ArrowLeftRight size={14} /></button>
              </div>
              <textarea bind:value={jcInput} class="flex-1 bg-transparent text-sm font-mono text-gray-800 dark:text-gray-200 p-4 resize-none outline-none placeholder-gray-400"
                placeholder={'[\n  { "name": "John", "age": 30 }\n]'} spellcheck="false"></textarea>
            </div>
            <div class="flex flex-col overflow-hidden flex-1">
              <div class="flex items-center gap-2 px-4 py-2 bg-gray-50 dark:bg-gray-900/30 border-b border-gray-200 dark:border-gray-700 shrink-0">
                <span class="text-[11px] font-bold text-cyan-500 uppercase tracking-wider">{jcOutputFormat.toUpperCase()}</span>
                <button onclick={() => copyText(jcResult.output, (v) => jcCopied = v)} class="ml-auto text-gray-400 hover:text-cyan-500">
                  {#if jcCopied}<Check size={14} class="text-green-500" />{:else}<Copy size={14} />{/if}
                </button>
              </div>
              <div class="flex-1 overflow-y-auto">
                {#if jcResult.error}
                  <div class="p-4"><div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-xl px-4 py-3 text-sm text-red-600 dark:text-red-400 font-mono whitespace-pre-wrap">{jcResult.error}</div></div>
                {:else}
                  <pre class="p-4 text-sm font-mono text-gray-800 dark:text-gray-200 whitespace-pre-wrap break-all leading-relaxed">{jcResult.output || ''}</pre>
                {/if}
              </div>
            </div>
          </div>

        <!-- ═══ JSON to Model ═══ -->
        {:else if activeTab === "jsonModel"}
          <div class="h-full flex">
            <div class="flex flex-col overflow-hidden flex-1 border-r border-gray-200 dark:border-gray-700">
              <div class="flex items-center gap-2 px-4 py-2 bg-gray-50 dark:bg-gray-900/30 border-b border-gray-200 dark:border-gray-700 shrink-0">
                <FileJson size={14} class="text-amber-500" />
                <span class="text-[11px] font-bold text-gray-500 uppercase tracking-wider">JSON</span>
              </div>
              <textarea bind:value={jmInput} class="flex-1 bg-transparent text-sm font-mono text-gray-800 dark:text-gray-200 p-4 resize-none outline-none placeholder-gray-400"
                placeholder={'{\n  "name": "John",\n  "age": 30\n}'} spellcheck="false"></textarea>
            </div>
            <div class="flex flex-col overflow-hidden flex-1">
              <div class="flex items-center gap-2 px-4 py-2 bg-gray-50 dark:bg-gray-900/30 border-b border-gray-200 dark:border-gray-700 shrink-0">
                {#each [
                  { l: "swift" as Lang, n: "Swift" },
                  { l: "kotlin" as Lang, n: "Kotlin" },
                  { l: "dart" as Lang, n: "Dart" },
                  { l: "go" as Lang, n: "Go" },
                  { l: "rust" as Lang, n: "Rust" },
                  { l: "typescript" as Lang, n: "TypeScript" },
                  { l: "python" as Lang, n: "Python" },
                  { l: "lua" as Lang, n: "Lua" },
                ] as opt}
                  <button onclick={() => (jmLang = opt.l)}
                    class="px-2 py-0.5 rounded text-[10px] font-bold {jmLang === opt.l ? 'bg-amber-100 dark:bg-amber-900/30 text-amber-600 dark:text-amber-400' : 'text-gray-400 hover:text-gray-600'}">
                    {opt.n}
                  </button>
                {/each}
                <input type="text" bind:value={jmClassName}
                  class="ml-2 w-28 bg-gray-100 dark:bg-gray-700 border border-transparent focus:border-amber-400 dark:focus:border-amber-600 rounded px-2 py-0.5 text-[10px] font-mono text-gray-800 dark:text-gray-200 outline-none"
                  placeholder="ClassName" />
                <button onclick={() => copyText(jmOutput, (v) => jmCopied = v)} class="ml-auto text-gray-400 hover:text-amber-500">
                  {#if jmCopied}<Check size={14} class="text-green-500" />{:else}<Copy size={14} />{/if}
                </button>
              </div>
              <div class="flex-1 overflow-y-auto">
                <pre class="p-4 text-sm font-mono text-gray-800 dark:text-gray-200 whitespace-pre-wrap leading-relaxed">{jmOutput || ''}</pre>
              </div>
            </div>
          </div>

        <!-- ═══ Color ═══ -->
        {:else if activeTab === "color"}
          <div class="flex-1 overflow-y-auto p-5 space-y-5">
            <div class="flex items-center gap-4">
              <input type="color" bind:value={colorHex}
                class="w-16 h-16 rounded-xl border-2 border-gray-200 dark:border-gray-700 cursor-pointer" />
              <div class="flex-1 space-y-2">
                <div class="h-10 rounded-xl border border-gray-200 dark:border-gray-700" style="background-color: {colorHex}"></div>
                <input type="text" bind:value={colorHex}
                  class="w-full bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-lg px-3 py-1.5 text-sm font-mono text-gray-800 dark:text-gray-200 outline-none focus:border-pink-400 dark:focus:border-pink-600"
                  placeholder="#6366F1" />
              </div>
            </div>
            <div>
              <p class="text-[10px] font-bold text-gray-400 uppercase tracking-wider mb-2">Presets</p>
              <div class="flex flex-wrap gap-1.5">
                {#each presetColors as c}
                  <button onclick={() => (colorHex = c)}
                    class="w-7 h-7 rounded-lg border-2 transition-all hover:scale-110 {colorHex === c ? 'border-gray-800 dark:border-white ring-2 ring-offset-1 ring-gray-400' : 'border-gray-200 dark:border-gray-700'}"
                    style="background-color: {c}" title={c}></button>
                {/each}
              </div>
            </div>
            <div class="space-y-1.5">
              <p class="text-[10px] font-bold text-gray-400 uppercase tracking-wider mb-2">Formats</p>
              {#each colorFormats as f}
                <div class="flex items-center gap-2 bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-lg px-3 py-2 group">
                  <span class="text-[10px] font-bold text-pink-500 bg-pink-100 dark:bg-pink-900/30 px-2 py-0.5 rounded-md shrink-0 w-32 text-center">{f.label}</span>
                  <code class="flex-1 text-xs font-mono text-gray-800 dark:text-gray-200 truncate">{f.value}</code>
                  <button onclick={() => { navigator.clipboard.writeText(f.value); colorCopied = f.label; setTimeout(() => colorCopied = "", 2000); }}
                    class="p-1 text-gray-400 opacity-0 group-hover:opacity-100 hover:text-pink-600 transition-all shrink-0">
                    {#if colorCopied === f.label}<Check size={14} class="text-green-500" />{:else}<Copy size={14} />{/if}
                  </button>
                </div>
              {/each}
            </div>
          </div>

        <!-- ═══ Timestamp ═══ -->
        {:else if activeTab === "timestamp"}
          <div class="flex-1 overflow-y-auto p-5 space-y-4">
            <!-- Input -->
            <div class="flex items-center gap-2">
              <div class="relative flex-1">
                <Clock size={14} class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400" />
                <input
                  type="text"
                  bind:value={tsInput}
                  class="w-full bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-xl pl-9 pr-4 py-2.5 text-sm font-mono text-gray-800 dark:text-gray-200 outline-none focus:border-violet-400 dark:focus:border-violet-600 transition-colors placeholder-gray-400"
                  placeholder="1700000000 or 2024-01-15T12:00:00Z"
                  onkeydown={(e) => { if (e.key === "Enter") {} }}
                />
              </div>
              <button
                onclick={setTsNow}
                class="px-4 py-2.5 rounded-xl text-xs font-bold text-white bg-violet-600 hover:bg-violet-700 transition-all shrink-0"
              >Now</button>
            </div>

            {#if tsParsed}
              <!-- Results -->
              <div class="space-y-1.5">
                {#each [
                  { label: "ISO 8601", value: tsParsed.iso },
                  { label: "Local", value: tsParsed.isoLocal },
                  { label: "UTC", value: tsParsed.utc },
                  { label: "RFC 2822", value: tsParsed.rfc2822 },
                  { label: "Epoch (seconds)", value: String(tsParsed.epochSec) },
                  { label: "Epoch (ms)", value: String(tsParsed.epochMs) },
                  { label: "Relative", value: tsParsed.relative },
                  { label: "Day", value: tsParsed.dayOfWeek },
                  { label: "Date", value: tsParsed.dateOnly },
                  { label: "Time (UTC)", value: tsParsed.timeOnly },
                  { label: "Unix cmd", value: tsParsed.unixCmd },
                ] as f}
                  <div class="flex items-center gap-2 bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-lg px-3 py-2 group">
                    <span class="text-[10px] font-bold text-violet-500 bg-violet-100 dark:bg-violet-900/30 px-2 py-0.5 rounded-md shrink-0 min-w-[120px] text-center">{f.label}</span>
                    <code class="flex-1 text-xs font-mono text-gray-800 dark:text-gray-200 truncate">{f.value}</code>
                    <button
                      onclick={() => { navigator.clipboard.writeText(f.value); tsCopied = f.label; setTimeout(() => tsCopied = "", 2000); }}
                      class="p-1 text-gray-400 opacity-0 group-hover:opacity-100 hover:text-violet-600 transition-all shrink-0"
                    >
                      {#if tsCopied === f.label}<Check size={14} class="text-green-500" />{:else}<Copy size={14} />{/if}
                    </button>
                  </div>
                {/each}
              </div>
            {:else if tsInput.trim()}
              <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-xl px-4 py-3 text-sm text-red-600 dark:text-red-400 font-mono">
                Invalid timestamp — enter epoch (10/13/16 digits) or ISO date string
              </div>
            {:else}
              <div class="text-sm text-gray-400 dark:text-gray-500 text-center py-12">
                Enter a Unix epoch, ISO date, or any parseable date string
              </div>
            {/if}

            <!-- Quick reference -->
            <div class="bg-violet-50 dark:bg-violet-900/15 border border-violet-200 dark:border-violet-800/50 rounded-xl p-4">
              <p class="text-[10px] font-bold text-violet-600 dark:text-violet-400 uppercase tracking-wider mb-2">Quick Reference</p>
              <div class="grid grid-cols-2 gap-x-4 gap-y-1 text-[11px]">
                <div class="flex justify-between"><span class="text-gray-500 dark:text-gray-400">Epoch seconds</span><code class="font-mono text-gray-700 dark:text-gray-300">10 digits</code></div>
                <div class="flex justify-between"><span class="text-gray-500 dark:text-gray-400">Epoch millis</span><code class="font-mono text-gray-700 dark:text-gray-300">13 digits</code></div>
                <div class="flex justify-between"><span class="text-gray-500 dark:text-gray-400">Epoch micros</span><code class="font-mono text-gray-700 dark:text-gray-300">16 digits</code></div>
                <div class="flex justify-between"><span class="text-gray-500 dark:text-gray-400">ISO 8601</span><code class="font-mono text-gray-700 dark:text-gray-300">2024-01-15T…</code></div>
              </div>
            </div>
          </div>

        <!-- ═══ Number Base ═══ -->
        {:else if activeTab === "numbase"}
          <div class="flex-1 overflow-y-auto p-5 space-y-4">
            <!-- Input -->
            <div class="flex items-center gap-2">
              <div class="flex items-center gap-1 bg-gray-100 dark:bg-gray-700 rounded-lg p-1">
                {#each [
                  { base: 16 as const, label: "HEX" },
                  { base: 10 as const, label: "DEC" },
                  { base: 8 as const, label: "OCT" },
                  { base: 2 as const, label: "BIN" },
                ] as b}
                  <button
                    onclick={() => (nbBase = b.base)}
                    class="px-2.5 py-1 rounded-md text-[10px] font-bold transition-all {nbBase === b.base
                      ? 'bg-white dark:bg-gray-600 text-orange-600 dark:text-orange-400 shadow-sm'
                      : 'text-gray-500 dark:text-gray-400'}"
                  >{b.label}</button>
                {/each}
              </div>
              <input
                type="text"
                bind:value={nbInput}
                class="flex-1 bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-xl px-4 py-2.5 text-sm font-mono text-gray-800 dark:text-gray-200 outline-none focus:border-orange-400 dark:focus:border-orange-600 transition-colors placeholder-gray-400"
                placeholder="Enter number..."
                spellcheck="false"
              />
            </div>

            {#if nbParsed}
              <div class="space-y-1.5">
                {#each [
                  { label: "Hexadecimal", value: nbParsed.hex, prefix: nbParsed.hexPrefixed },
                  { label: "Decimal", value: nbParsed.dec, prefix: "" },
                  { label: "Octal", value: nbParsed.oct, prefix: nbParsed.octPrefixed },
                  { label: "Binary", value: nbParsed.bin, prefix: nbParsed.binPrefixed },
                ] as f}
                  <div class="bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-lg px-3 py-2.5 group">
                    <div class="flex items-center gap-2">
                      <span class="text-[10px] font-bold text-orange-500 bg-orange-100 dark:bg-orange-900/30 px-2 py-0.5 rounded-md shrink-0 min-w-[100px] text-center">{f.label}</span>
                      <code class="flex-1 text-sm font-mono text-gray-800 dark:text-gray-200 break-all">{f.value}</code>
                      <button
                        onclick={() => { navigator.clipboard.writeText(f.prefix || f.value); nbCopied = f.label; setTimeout(() => nbCopied = "", 2000); }}
                        class="p-1 text-gray-400 opacity-0 group-hover:opacity-100 hover:text-orange-600 transition-all shrink-0"
                      >
                        {#if nbCopied === f.label}<Check size={14} class="text-green-500" />{:else}<Copy size={14} />{/if}
                      </button>
                    </div>
                    {#if f.prefix}
                      <div class="mt-1 text-[10px] font-mono text-gray-400 pl-[112px]">{f.prefix}</div>
                    {/if}
                  </div>
                {/each}

                <!-- Extra info -->
                {#if nbParsed.char || nbParsed.bits}
                  <div class="flex flex-wrap gap-2 mt-2">
                    {#if nbParsed.char}
                      <div class="flex items-center gap-2 bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-lg px-3 py-1.5">
                        <span class="text-[10px] font-bold text-orange-500 bg-orange-100 dark:bg-orange-900/30 px-2 py-0.5 rounded-md">Char</span>
                        <code class="text-sm font-mono text-gray-800 dark:text-gray-200">{nbParsed.char}</code>
                      </div>
                    {/if}
                    <div class="flex items-center gap-2 bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-lg px-3 py-1.5">
                      <span class="text-[10px] font-bold text-orange-500 bg-orange-100 dark:bg-orange-900/30 px-2 py-0.5 rounded-md">Bits</span>
                      <code class="text-sm font-mono text-gray-800 dark:text-gray-200">{nbParsed.bits}</code>
                    </div>
                    {#if nbParsed.byteHex}
                      <div class="flex items-center gap-2 bg-gray-50 dark:bg-gray-900/50 border border-gray-200 dark:border-gray-700 rounded-lg px-3 py-1.5">
                        <span class="text-[10px] font-bold text-orange-500 bg-orange-100 dark:bg-orange-900/30 px-2 py-0.5 rounded-md">Byte</span>
                        <code class="text-sm font-mono text-gray-800 dark:text-gray-200">{nbParsed.byteHex}</code>
                      </div>
                    {/if}
                  </div>
                {/if}
              </div>
            {:else if nbInput.trim()}
              <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-xl px-4 py-3 text-sm text-red-600 dark:text-red-400 font-mono">
                Invalid number for base {nbBase}
              </div>
            {:else}
              <div class="text-sm text-gray-400 dark:text-gray-500 text-center py-12">
                Enter a number in the selected base
              </div>
            {/if}

            <!-- Quick values -->
            <div class="bg-orange-50 dark:bg-orange-900/15 border border-orange-200 dark:border-orange-800/50 rounded-xl p-4">
              <p class="text-[10px] font-bold text-orange-600 dark:text-orange-400 uppercase tracking-wider mb-2">Quick Values</p>
              <div class="flex flex-wrap gap-1.5">
                {#each nbQuickValues as qv}
                  <button
                    onclick={() => { nbBase = qv.base; nbInput = qv.value; }}
                    class="px-2.5 py-1 rounded-lg text-[11px] font-mono font-bold bg-white dark:bg-gray-800 text-gray-600 dark:text-gray-300 border border-gray-200 dark:border-gray-700 hover:border-orange-300 dark:hover:border-orange-700 hover:text-orange-600 dark:hover:text-orange-400 transition-all"
                  >{qv.label}</button>
                {/each}
              </div>
            </div>
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  @keyframes modal-in { from { opacity: 0; transform: scale(0.95) translateY(-10px); } to { opacity: 1; transform: scale(1) translateY(0); } }
  .animate-modal-in { animation: modal-in 0.2s ease-out; }
  textarea { tab-size: 2; }
  input[type="color"] { -webkit-appearance: none; appearance: none; padding: 0; }
  input[type="color"]::-webkit-color-swatch-wrapper { padding: 2px; }
  input[type="color"]::-webkit-color-swatch { border: none; border-radius: 8px; }
</style>
