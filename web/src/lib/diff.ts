// 行级 diff 引擎（LCS），同时服务于 JSON 工具的 diff 模式与「文本 Diff」工具。
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
}

type Op =
  | { t: "equal"; a: string; b: string }
  | { t: "delete"; a: string }
  | { t: "insert"; b: string };

/** 经典 LCS，回溯出 equal/delete/insert 序列。 */
function lcsOps(a: string[], b: string[]): Op[] {
  const n = a.length;
  const m = b.length;
  // dp[i][j] = LCS 长度 of a[i:], b[j:]
  const dp: number[][] = Array.from({ length: n + 1 }, () =>
    new Array<number>(m + 1).fill(0),
  );
  for (let i = n - 1; i >= 0; i--) {
    for (let j = m - 1; j >= 0; j--) {
      dp[i][j] =
        a[i] === b[j]
          ? dp[i + 1][j + 1] + 1
          : Math.max(dp[i + 1][j], dp[i][j + 1]);
    }
  }
  const ops: Op[] = [];
  let i = 0;
  let j = 0;
  while (i < n && j < m) {
    if (a[i] === b[j]) {
      ops.push({ t: "equal", a: a[i], b: b[j] });
      i++;
      j++;
    } else if (dp[i + 1][j] >= dp[i][j + 1]) {
      ops.push({ t: "delete", a: a[i] });
      i++;
    } else {
      ops.push({ t: "insert", b: b[j] });
      j++;
    }
  }
  while (i < n) ops.push({ t: "delete", a: a[i++] });
  while (j < m) ops.push({ t: "insert", b: b[j++] });
  return ops;
}

/** 把连续的 delete 段与紧随的 insert 段逐行配对为 replace，便于并排对齐。 */
export function diffLines(aText: string, bText: string): DiffResult {
  const a = aText.split("\n");
  const b = bText.split("\n");
  const ops = lcsOps(a, b);

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

  return { rows, added, removed, changed };
}
