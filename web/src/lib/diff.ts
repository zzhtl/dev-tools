// 行级 diff 引擎（Myers O(ND)），同时服务于 JSON 工具的 diff 模式与「文本 Diff」工具。
// 产出对齐后的左右行，便于并排渲染。

export type DiffType = "equal" | "insert" | "delete" | "replace";

export interface DiffRow {
  type: DiffType;
  /** 左侧行号（1 起），无对应行时为 null */
  leftNo: number | null;
  rightNo: number | null;
  left: string;
  right: string;
}

export interface DiffResult {
  rows: DiffRow[];
  added: number; // 新增行数
  removed: number; // 删除行数
  changed: number; // 修改行数
  /** 超限或降级时的说明，正常对比为 undefined */
  notice?: string;
}

type Op =
  | { t: "equal"; a: string; b: string }
  | { t: "delete"; a: string }
  | { t: "insert"; b: string };

/** 超过这个行数就不再做精确 diff —— 逐行对齐的结果在这个量级也已经不可读了。 */
const MAX_LINES = 50_000;
/**
 * Myers 的编辑距离预算。d 每加 1 就多存一条 trace，
 * 两份内容差异极大时（D ≈ n+m）继续算下去内存会失控，因此设上限后降级。
 */
const MAX_EDIT_DISTANCE = 8_000;

/**
 * Myers O(ND) 差分。
 *
 * 旧实现是 O(n·m) 的全量 LCS 矩阵：两份 5000 行的文件就要分配 2500 万个格子，
 * 大文本直接把标签页拖垮。Myers 的开销只与「差异量」相关，
 * 对通常只改了几行的场景近乎线性。
 *
 * 这里先把行 intern 成整数，比较退化为整数比较，避免反复做字符串比较。
 */
function myersOps(a: string[], b: string[]): Op[] | null {
  const dictionary = new Map<string, number>();
  const intern = (line: string): number => {
    let id = dictionary.get(line);
    if (id === undefined) {
      id = dictionary.size;
      dictionary.set(line, id);
    }
    return id;
  };
  const av = a.map(intern);
  const bv = b.map(intern);

  const n = av.length;
  const m = bv.length;
  const max = n + m;
  const offset = max;
  const budget = Math.min(max, MAX_EDIT_DISTANCE);

  let v = new Int32Array(2 * max + 1);
  const trace: Int32Array[] = [];

  for (let d = 0; d <= budget; d++) {
    trace.push(v.slice());
    for (let k = -d; k <= d; k += 2) {
      let x: number;
      if (k === -d || (k !== d && v[k - 1 + offset] < v[k + 1 + offset])) {
        x = v[k + 1 + offset];
      } else {
        x = v[k - 1 + offset] + 1;
      }
      let y = x - k;
      while (x < n && y < m && av[x] === bv[y]) {
        x++;
        y++;
      }
      v[k + offset] = x;
      if (x >= n && y >= m) {
        return backtrack(trace, a, b, offset);
      }
    }
  }
  // 差异太大，交给调用方降级处理
  return null;
}

function backtrack(trace: Int32Array[], a: string[], b: string[], offset: number): Op[] {
  const ops: Op[] = [];
  let x = a.length;
  let y = b.length;

  for (let d = trace.length - 1; d >= 0; d--) {
    const v = trace[d];
    const k = x - y;
    const prevK =
      k === -d || (k !== d && v[k - 1 + offset] < v[k + 1 + offset]) ? k + 1 : k - 1;
    const prevX = v[prevK + offset];
    const prevY = prevX - prevK;

    while (x > prevX && y > prevY) {
      ops.push({ t: "equal", a: a[x - 1], b: b[y - 1] });
      x--;
      y--;
    }
    if (d > 0) {
      if (x === prevX) {
        ops.push({ t: "insert", b: b[y - 1] });
        y--;
      } else {
        ops.push({ t: "delete", a: a[x - 1] });
        x--;
      }
    }
  }
  ops.reverse();
  return ops;
}

/** 降级：整体替换。用于差异过大、Myers 预算耗尽的情况。 */
function wholesaleOps(a: string[], b: string[]): Op[] {
  return [
    ...a.map((line): Op => ({ t: "delete", a: line })),
    ...b.map((line): Op => ({ t: "insert", b: line })),
  ];
}

/** 剥掉两端完全相同的部分，Myers 只处理中间真正有差异的区域。 */
function diffOps(a: string[], b: string[]): { ops: Op[]; degraded: boolean } {
  let head = 0;
  while (head < a.length && head < b.length && a[head] === b[head]) head++;

  let tail = 0;
  while (
    tail < a.length - head &&
    tail < b.length - head &&
    a[a.length - 1 - tail] === b[b.length - 1 - tail]
  ) {
    tail++;
  }

  const midA = a.slice(head, a.length - tail);
  const midB = b.slice(head, b.length - tail);

  const middle = myersOps(midA, midB);
  const degraded = middle === null;

  const ops: Op[] = [];
  for (let i = 0; i < head; i++) ops.push({ t: "equal", a: a[i], b: b[i] });
  ops.push(...(middle ?? wholesaleOps(midA, midB)));
  for (let i = a.length - tail; i < a.length; i++) {
    ops.push({ t: "equal", a: a[i], b: b[i - (a.length - b.length)] });
  }
  return { ops, degraded };
}

/** 把连续的 delete 段与紧随的 insert 段逐行配对为 replace，便于并排对齐。 */
export function diffLines(aText: string, bText: string): DiffResult {
  const a = aText.split("\n");
  const b = bText.split("\n");

  if (a.length > MAX_LINES || b.length > MAX_LINES) {
    return {
      rows: [],
      added: 0,
      removed: 0,
      changed: 0,
      notice: `内容超过 ${MAX_LINES.toLocaleString("zh-CN")} 行，已跳过逐行对比`,
    };
  }

  const { ops, degraded } = diffOps(a, b);

  const rows: DiffRow[] = [];
  let leftNo = 0;
  let rightNo = 0;
  let added = 0;
  let removed = 0;
  let changed = 0;

  let k = 0;
  while (k < ops.length) {
    const op = ops[k];
    if (op.t === "equal") {
      leftNo++;
      rightNo++;
      rows.push({ type: "equal", leftNo, rightNo, left: op.a, right: op.b });
      k++;
      continue;
    }
    // 收集一段连续的 delete / insert
    const dels: string[] = [];
    const ins: string[] = [];
    while (k < ops.length && ops[k].t !== "equal") {
      const o = ops[k];
      if (o.t === "delete") dels.push(o.a);
      else if (o.t === "insert") ins.push(o.b);
      k++;
    }
    const pairs = Math.min(dels.length, ins.length);
    for (let p = 0; p < pairs; p++) {
      leftNo++;
      rightNo++;
      changed++;
      rows.push({
        type: "replace",
        leftNo,
        rightNo,
        left: dels[p],
        right: ins[p],
      });
    }
    for (let p = pairs; p < dels.length; p++) {
      leftNo++;
      removed++;
      rows.push({ type: "delete", leftNo, rightNo: null, left: dels[p], right: "" });
    }
    for (let p = pairs; p < ins.length; p++) {
      rightNo++;
      added++;
      rows.push({ type: "insert", leftNo: null, rightNo, left: "", right: ins[p] });
    }
  }

  return {
    rows,
    added,
    removed,
    changed,
    notice: degraded ? "两侧差异过大，已按整体替换呈现" : undefined,
  };
}
