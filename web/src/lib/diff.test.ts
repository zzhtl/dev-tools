import { describe, expect, test } from "bun:test";
import { diffLines } from "./diff";

function render(a: string, b: string) {
  const result = diffLines(a, b);
  return {
    ...result,
    left: result.rows.map((row) => row.left),
    right: result.rows.map((row) => row.right),
    types: result.rows.map((row) => row.type),
  };
}

describe("diffLines", () => {
  test("完全相同的内容没有任何差异", () => {
    const result = render("a\nb\nc", "a\nb\nc");
    expect(result.added).toBe(0);
    expect(result.removed).toBe(0);
    expect(result.changed).toBe(0);
    expect(result.types).toEqual(["equal", "equal", "equal"]);
  });

  test("纯新增", () => {
    const result = render("a", "a\nb\nc");
    expect(result.added).toBe(2);
    expect(result.removed).toBe(0);
    expect(result.changed).toBe(0);
  });

  test("纯删除", () => {
    const result = render("a\nb\nc", "a");
    expect(result.removed).toBe(2);
    expect(result.added).toBe(0);
    expect(result.changed).toBe(0);
  });

  test("中间一行被修改，配对成 replace", () => {
    const result = render("a\nb\nc", "a\nB\nc");
    expect(result.changed).toBe(1);
    expect(result.added).toBe(0);
    expect(result.removed).toBe(0);
    expect(result.types).toEqual(["equal", "replace", "equal"]);
  });

  test("空输入两侧都算作一个空行", () => {
    const result = render("", "");
    expect(result.rows.length).toBe(1);
    expect(result.types).toEqual(["equal"]);
  });

  test("从空到有内容", () => {
    const result = render("", "x");
    expect(result.changed + result.added).toBeGreaterThan(0);
  });

  test("行号连续且与内容对应", () => {
    const result = diffLines("a\nb\nc", "a\nc");
    const lefts = result.rows.filter((row) => row.leftNo !== null).map((row) => row.leftNo);
    expect(lefts).toEqual([1, 2, 3]);
    const rights = result.rows.filter((row) => row.rightNo !== null).map((row) => row.rightNo);
    expect(rights).toEqual([1, 2]);
  });

  test("保留首尾相同段，只对中间做差分", () => {
    const a = ["head", "x", "tail"].join("\n");
    const b = ["head", "y", "tail"].join("\n");
    const result = render(a, b);
    expect(result.types).toEqual(["equal", "replace", "equal"]);
  });

  test("大文本不会退化到 O(n·m)：一万行只改一行", () => {
    const base = Array.from({ length: 10_000 }, (_, i) => `line ${i}`);
    const modified = [...base];
    modified[5_000] = "changed";
    const started = performance.now();
    const result = diffLines(base.join("\n"), modified.join("\n"));
    const elapsed = performance.now() - started;
    expect(result.changed).toBe(1);
    // 旧的 O(n·m) 实现在这个规模需要分配 1 亿个格子，这里给出宽松但有意义的上界
    expect(elapsed).toBeLessThan(2_000);
  });

  test("超过行数上限时给出提示而不是卡死", () => {
    const huge = Array.from({ length: 50_001 }, (_, i) => `l${i}`).join("\n");
    const result = diffLines(huge, "x");
    expect(result.rows.length).toBe(0);
    expect(result.notice).toContain("已跳过逐行对比");
  });

  test("两侧完全不同时降级为整体替换且不报错", () => {
    const a = Array.from({ length: 200 }, (_, i) => `a${i}`).join("\n");
    const b = Array.from({ length: 200 }, (_, i) => `b${i}`).join("\n");
    const result = diffLines(a, b);
    expect(result.rows.length).toBeGreaterThan(0);
    expect(result.added + result.removed + result.changed).toBeGreaterThan(0);
  });
});
