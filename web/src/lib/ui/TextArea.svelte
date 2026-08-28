<script lang="ts">
  interface Props {
    value?: string;
    placeholder?: string;
    disabled?: boolean;
    readonly?: boolean;
    mono?: boolean;
    invalid?: boolean;
    rows?: number;
    /** 撑满父容器高度而不是按 rows 固定 */
    fill?: boolean;
    spellcheck?: boolean;
    ariaLabel?: string;
    oninput?: (event: Event) => void;
    onkeydown?: (event: KeyboardEvent) => void;
  }

  let {
    value = $bindable(""),
    placeholder,
    disabled = false,
    readonly = false,
    mono = true,
    invalid = false,
    rows = 8,
    fill = false,
    spellcheck = false,
    ariaLabel,
    oninput,
    onkeydown,
  }: Props = $props();
</script>

<textarea
  class="area"
  class:mono
  class:invalid
  class:fill
  bind:value
  {placeholder}
  {disabled}
  {readonly}
  {rows}
  {spellcheck}
  aria-label={ariaLabel}
  aria-invalid={invalid}
  {oninput}
  {onkeydown}
></textarea>

<style>
  .area {
    width: 100%;
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    color: var(--color-text);
    padding: var(--sp-3);
    font-size: var(--fs-md);
    line-height: var(--lh-normal);
    resize: vertical;
    transition:
      border-color var(--motion),
      box-shadow var(--motion);
  }

  .mono {
    font-family: var(--font-mono);
  }

  .fill {
    flex: 1;
    min-height: 0;
    resize: none;
  }

  .area::placeholder {
    color: var(--color-text-muted);
  }

  .area:focus {
    outline: none;
    border-color: var(--color-primary);
    box-shadow: 0 0 0 3px var(--color-focus-ring);
  }

  .area:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .invalid {
    border-color: var(--color-danger);
  }
</style>
