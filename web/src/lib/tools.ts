import type { Component } from "svelte";

/** 工具组件不接收 props，这里用宽松签名以兼容各组件的具体类型。 */
type ToolComponent = Component<any>;

export interface ToolMeta {
  /** URL 中使用，必须稳定 */
  id: string;
  name: string;
  /** 侧边栏与命令面板中的图标，等宽字符或 emoji */
  icon: string;
  desc: string;
  category: string;
  /** 供 ⌘K 模糊搜索，中英文与常见别名都写进来 */
  keywords: string[];
  /** 动态 import：首屏不再加载全部工具 */
  load: () => Promise<{ default: ToolComponent }>;
}

export const TOOLS: ToolMeta[] = [
  {
    id: "json",
    name: "JSON 工具",
    icon: "{ }",
    desc: "格式化、转换、Diff、Schema、查询",
    category: "数据处理",
    keywords: ["json", "格式化", "format", "diff", "对比", "schema", "jsonpath", "jq", "yaml", "toml", "xml", "csv", "结构体", "struct"],
    load: () => import("../tools/json_tool/json_tool.svelte"),
  },
  {
    id: "regex",
    name: "正则表达式",
    icon: ".*",
    desc: "正则测试、匹配高亮与替换",
    category: "数据处理",
    keywords: ["regex", "正则", "regexp", "匹配", "match", "替换", "replace"],
    load: () => import("../tools/regex_tool/regex_tool.svelte"),
  },
  {
    id: "html",
    name: "HTML 工具",
    icon: "</>",
    desc: "格式化、压缩、实时预览",
    category: "数据处理",
    keywords: ["html", "格式化", "format", "压缩", "minify", "预览", "preview"],
    load: () => import("../tools/html_tool/html_tool.svelte"),
  },
  {
    id: "text-diff",
    name: "文本对比",
    icon: "⇆",
    desc: "两段文本行级差异对比",
    category: "数据处理",
    keywords: ["diff", "对比", "比较", "compare", "文本", "text"],
    load: () => import("../tools/text_diff_tool/text_diff_tool.svelte"),
  },
  {
    id: "markdown",
    name: "Markdown",
    icon: "M↓",
    desc: "实时预览与导出 HTML",
    category: "数据处理",
    keywords: ["markdown", "md", "预览", "preview", "gfm"],
    load: () => import("../tools/markdown_tool/markdown_tool.svelte"),
  },
  {
    id: "text",
    name: "文本处理",
    icon: "T",
    desc: "去重、排序、统计、大小写",
    category: "数据处理",
    keywords: ["文本", "text", "去重", "排序", "统计", "大小写", "camel", "snake", "行号"],
    load: () => import("../tools/text_tool/text_tool.svelte"),
  },
  {
    id: "sql",
    name: "SQL 格式化",
    icon: "DB",
    desc: "格式化、压缩、关键字大小写",
    category: "数据处理",
    keywords: ["sql", "格式化", "format", "美化", "mysql", "postgres", "查询"],
    load: () => import("../tools/sql_tool/sql_tool.svelte"),
  },
  {
    id: "yaml",
    name: "YAML / TOML",
    icon: "Y",
    desc: "校验、格式化与互转",
    category: "数据处理",
    keywords: ["yaml", "yml", "toml", "配置", "格式化", "校验", "转换"],
    load: () => import("../tools/yaml_tool/yaml_tool.svelte"),
  },
  {
    id: "time",
    name: "时间转换",
    icon: "⏱",
    desc: "时间戳与日期互转",
    category: "时间日期",
    keywords: ["时间", "时间戳", "timestamp", "unix", "date", "日期", "时区", "timezone"],
    load: () => import("../tools/time_tool/time_convert.svelte"),
  },
  {
    id: "cron",
    name: "Cron 表达式",
    icon: "⏰",
    desc: "解析、预测与自然语言生成",
    category: "时间日期",
    keywords: ["cron", "定时", "crontab", "调度", "schedule", "表达式"],
    load: () => import("../tools/cron_tool/cron_tool.svelte"),
  },
  {
    id: "crypto",
    name: "加密解密",
    icon: "🔐",
    desc: "AES / DES / 3DES / RSA",
    category: "编码加密",
    keywords: ["加密", "解密", "aes", "des", "3des", "rsa", "encrypt", "decrypt"],
    load: () => import("../tools/crypto_tool/crypto_tool.svelte"),
  },
  {
    id: "jwt",
    name: "JWT 解析",
    icon: "🔑",
    desc: "解码 header / payload 与验签",
    category: "编码加密",
    keywords: ["jwt", "token", "解码", "decode", "验签", "hs256", "claims"],
    load: () => import("../tools/jwt_tool/jwt_tool.svelte"),
  },
  {
    id: "base64",
    name: "Base64",
    icon: "B64",
    desc: "文本、文件与图片 Base64 编解码",
    category: "编码加密",
    keywords: ["base64", "编码", "解码", "encode", "decode", "图片", "dataurl"],
    load: () => import("../tools/base64_tool/base64_tool.svelte"),
  },
  {
    id: "hash",
    name: "Hash 计算",
    icon: "#",
    desc: "MD5 / SHA 系列摘要",
    category: "编码加密",
    keywords: ["hash", "哈希", "md5", "sha1", "sha256", "sha512", "sha3", "ripemd", "摘要", "校验"],
    load: () => import("../tools/hash_tool/hash_tool.svelte"),
  },
  {
    id: "url",
    name: "URL 编解码",
    icon: "%",
    desc: "URL encode / decode",
    category: "编码加密",
    keywords: ["url", "编码", "解码", "encode", "decode", "转义", "percent"],
    load: () => import("../tools/url_tool/url_tool.svelte"),
  },
  {
    id: "escape",
    name: "转义工具",
    icon: "\\\\",
    desc: "Unicode / HTML / JSON / SQL / Shell",
    category: "编码加密",
    keywords: ["转义", "escape", "unicode", "html", "实体", "json", "sql", "shell", "反转义"],
    load: () => import("../tools/escape_tool/escape_tool.svelte"),
  },
  {
    id: "radix",
    name: "进制转换",
    icon: "0b",
    desc: "2/8/10/16 互转与位运算",
    category: "编码加密",
    keywords: ["进制", "radix", "二进制", "十六进制", "hex", "binary", "位运算", "bit", "补码"],
    load: () => import("../tools/radix_tool/radix_tool.svelte"),
  },
  {
    id: "qrcode",
    name: "二维码",
    icon: "▣",
    desc: "文本生成二维码并下载",
    category: "生成工具",
    keywords: ["二维码", "qrcode", "qr", "扫码", "wifi"],
    load: () => import("../tools/qrcode_tool/qrcode_tool.svelte"),
  },
  {
    id: "uuid",
    name: "UUID",
    icon: "⊕",
    desc: "批量生成 UUID",
    category: "生成工具",
    keywords: ["uuid", "guid", "唯一", "标识", "v4", "v7", "随机"],
    load: () => import("../tools/uuid_tool/uuid_tool.svelte"),
  },
  {
    id: "password",
    name: "密码生成",
    icon: "***",
    desc: "强随机密码与强度评估",
    category: "生成工具",
    keywords: ["密码", "password", "随机", "random", "强度", "熵", "生成"],
    load: () => import("../tools/password_tool/password_tool.svelte"),
  },
  {
    id: "color",
    name: "颜色选择器",
    icon: "◐",
    desc: "取色与 HEX / RGB / HSL 互转",
    category: "生成工具",
    keywords: ["颜色", "color", "取色", "hex", "rgb", "hsl", "调色板", "对比度"],
    load: () => import("../tools/color_tool/color_tool.svelte"),
  },
  {
    id: "dns",
    name: "DNS 解析",
    icon: "◎",
    desc: "DNS 记录查询与端口连通性探测",
    category: "网络工具",
    keywords: ["dns", "域名", "解析", "resolve", "a", "aaaa", "mx", "cname", "txt", "ns", "端口", "port", "连通性", "telnet"],
    load: () => import("../tools/dns_tool/dns_tool.svelte"),
  },
  {
    id: "ip",
    name: "IP / CIDR",
    icon: "⊞",
    desc: "子网掩码与网段计算",
    category: "网络工具",
    keywords: ["ip", "cidr", "子网", "掩码", "subnet", "网段", "ipv4", "ipv6", "网络"],
    load: () => import("../tools/ip_tool/ip_tool.svelte"),
  },
  {
    id: "linux",
    name: "Linux 命令",
    icon: "$_",
    desc: "常用命令速查",
    category: "系统工具",
    keywords: ["linux", "命令", "command", "shell", "bash", "速查", "cheatsheet"],
    load: () => import("../tools/cmd_linux/cmd_linux.svelte"),
  },
  {
    id: "image",
    name: "图片转换",
    icon: "◲",
    desc: "格式互转、缩放与压缩",
    category: "系统工具",
    keywords: ["图片", "image", "转换", "convert", "png", "jpeg", "jpg", "webp", "gif", "ico", "压缩", "缩放"],
    load: () => import("../tools/image_tool/image_tool.svelte"),
  },
];

export const TOOL_BY_ID = new Map(TOOLS.map((t) => [t.id, t]));

export const DEFAULT_TOOL_ID = "json";

/** 保持 TOOLS 中的出现顺序分组，避免再维护一份分类数组。 */
export function groupByCategory(tools: ToolMeta[]): { name: string; tools: ToolMeta[] }[] {
  const groups: { name: string; tools: ToolMeta[] }[] = [];
  for (const tool of tools) {
    let group = groups.find((g) => g.name === tool.category);
    if (!group) {
      group = { name: tool.category, tools: [] };
      groups.push(group);
    }
    group.tools.push(tool);
  }
  return groups;
}
