<script lang="ts">
  interface Props {
    value?: string;
    /** 用 value/oninput 手动绑定，因此 type 可以是动态的 */
    type?: "text" | "number" | "password" | "search" | "url";
    placeholder?: string;
    disabled?: boolean;
    readonly?: boolean;
    mono?: boolean;
    size?: "sm" | "md";
    invalid?: boolean;
    ariaLabel?: string;
    oninput?: (event: Event) => void;
    onkeydown?: (event: KeyboardEvent) => void;
  }

  let {
    value = $bindable(""),
    type = "text",
    placeholder,
    disabled = false,
    readonly = false,
    mono = false,
    size = "md",
    invalid = false,
    ariaLabel,
    oninput,
    onkeydown,
  }: Props = $props();

  function handleInput(event: Event) {
    value = (event.currentTarget as HTMLInputElement).value;
    oninput?.(event);
  }
</script>

<input
  class="input {size}"
  class:mono
  class:invalid
  {type}
  {value}
  {placeholder}
  {disabled}
  {readonly}
  aria-label={ariaLabel}
  aria-invalid={invalid}
  oninput={handleInput}
  {onkeydown}
/>

<style>
  .input {
    width: 100%;
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    color: var(--color-text);
    transition:
      border-color var(--motion),
      box-shadow var(--motion);
  }

  .md {
    padding: 0.5rem var(--sp-3);
    font-size: var(--fs-md);
  }

  .sm {
    padding: 0.3rem var(--sp-2);
    font-size: var(--fs-sm);
  }

  .mono {
    font-family: var(--font-mono);
  }

  .input::placeholder {
    color: var(--color-text-muted);
  }

  .input:focus {
    outline: none;
    border-color: var(--color-primary);
    box-shadow: 0 0 0 3px var(--color-focus-ring);
  }

  .input:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .invalid {
    border-color: var(--color-danger);
  }

  .invalid:focus {
    box-shadow: 0 0 0 3px var(--color-danger-bg);
  }
</style>
