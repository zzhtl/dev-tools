<script lang="ts">
  interface Option {
    value: string;
    label: string;
  }

  interface Props {
    value?: string;
    options: Option[];
    disabled?: boolean;
    size?: "sm" | "md";
    ariaLabel?: string;
    onchange?: (event: Event) => void;
  }

  let {
    value = $bindable(""),
    options,
    disabled = false,
    size = "md",
    ariaLabel,
    onchange,
  }: Props = $props();
</script>

<select class="select {size}" bind:value {disabled} aria-label={ariaLabel} {onchange}>
  {#each options as option (option.value)}
    <option value={option.value}>{option.label}</option>
  {/each}
</select>

<style>
  .select {
    background-color: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    color: var(--color-text);
    cursor: pointer;
    appearance: none;
    background-image: url("data:image/svg+xml;charset=UTF-8,%3csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='%238a8d9c' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3e%3cpolyline points='6 9 12 15 18 9'%3e%3c/polyline%3e%3c/svg%3e");
    background-repeat: no-repeat;
    background-position: right var(--sp-2) center;
    background-size: 1em;
    transition:
      border-color var(--motion),
      box-shadow var(--motion);
  }

  .md {
    padding: 0.5rem 2.1rem 0.5rem var(--sp-3);
    font-size: var(--fs-md);
  }

  .sm {
    padding: 0.3rem 1.9rem 0.3rem var(--sp-2);
    font-size: var(--fs-sm);
  }

  .select:focus {
    outline: none;
    border-color: var(--color-primary);
    box-shadow: 0 0 0 3px var(--color-focus-ring);
  }

  .select:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  option {
    background: var(--color-surface);
    color: var(--color-text);
  }
</style>
