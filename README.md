<img src="logo.png" alt="Zippy" width="150">

**Zippy** — реактивный фронтенд-фреймворк со своим компилятором на Rust.
Сигналы, scoped CSS, условный рендер, циклы, компоненты, SPA-роутер.

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

# Dev-сервер (автокомпиляция .zippy + bundling)
bun run dev

# Прод-билд
bun run build -- examples/zippy-site
```

## Архитектура

```
packages/
├── runtime/        — сигналы, эффекты, DOM, роутер (TypeScript)
├── compiler/       — парсер + кодогенератор .zippy → JS (Rust)
└── zippy-cli/      — dev-сервер, prod-билд (Bun)
examples/
└── zippy-site/     — SPA-демо (роутер, счётчик, туду-лист)
```

## Фичи

| Синтаксис | Описание |
|-----------|----------|
| `{expr}` | Реактивное выражение (`signal.val`) |
| `@click={fn}` | Обработчик события (в т.ч. внутри `{#each}`) |
| `bind:value={x}` | Двусторонний binding |
| `class:active={cond}` | Условный класс |
| `{#if cond}...{:else if c2}...{:else}...{/if}` | Условный рендер с цепочкой |
| `{#each items as item, i}...{/each}` | Цикл с keyed reconciliation |
| `<Child prop={val}>slot</Child>` | Компоненты + слоты |
| `<slot>` | Слот для контента |
| `onMount(fn)` / `onDestroy(fn)` | Lifecycle-хуки |
| `<style>` | Scoped CSS (`[data-z-{hash}]`) |
| `lang="ts"` | TypeScript в `<script>` |
| `createRouter(routes)` | Hash-based SPA-роутер |

## CLI

```bash
# Dev-сервер: автокомпиляция .zippy, bun-бандлинг, hot-reload
bun run dev

# Prod-билд: компиляция + минификация
bun run build -- [dir]
```

## SPA-демо

```
examples/zippy-site/
  index.html          — точка входа
  src/App.zippy       — корневой компонент с роутером
  src/Home.zippy      — приветственная страница
  src/Counter.zippy   — счётчик (+/-/reset)
  src/Todo.zippy      — туду-лист (добавить/удалить)
```

## Тесты

```bash
cargo test --manifest-path packages/compiler/Cargo.toml
```
