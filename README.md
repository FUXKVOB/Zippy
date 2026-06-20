<img src="logo.png" alt="Zippy" width="150">

**Zippy** — реактивный фронтенд-фреймворк. Свой компилятор (Rust), сигналы, scoped CSS, `{#if}`/`{#each}`, двусторонний binding.

```html
<script>
  let count = signal(0);
</script>

<style>
  button { padding: 8px 16px; }
</style>

<template>
  <button @click={() => count.val++}>
    Clicked {count} times
  </button>
</template>
```

## Быстрый старт

```bash
git clone https://github.com/FUXKVOB/Zippy.git
cd Zippy
bun install
cargo build --manifest-path packages/compiler/Cargo.toml

# Dev-сервер
bun run dev

# Скомпилировать .zippy → .js
packages/compiler/target/debug/zippy-compiler examples/Counter.zippy
```

## Архитектура

```
packages/
├── runtime/        — сигналы, эффекты, DOM-хелперы (TypeScript)
├── compiler/       — парсер + кодогенератор .zippy → JS (Rust)
└── zippy-cli/      — дев-сервер, билд (Bun)
```

## Фичи

| Синтаксис | Описание |
|-----------|----------|
| `{expr}` | Реактивное выражение (signal.val) |
| `@click={fn}` | Обработчик события |
| `bind:value={x}` | Двусторонний binding |
| `{#if show}...{/if}` | Условный рендер |
| `{#each items as item, i}...{/each}` | Цикл с индексом |
| `<Child prop={val} />` | Дочерние компоненты с пропсами |
| `<style>` | Scoped CSS (`[data-z-{hash}]`) |

## Тесты

```bash
cargo test --manifest-path packages/compiler/Cargo.toml
```
