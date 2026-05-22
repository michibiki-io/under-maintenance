<script lang="ts">
  import heroEn from '../under-meintenance.en.svg?url';
  import heroJp from '../under-meintenance.jp.svg?url';

  type Locale = 'en' | 'ja';

  const storageKey = 'under-maintenance-locale';
  const heroes: Record<Locale, { alt: string; next: Locale; src: string; switchLabel: string; toggleLabel: string }> = {
    en: {
      alt: 'Under maintenance',
      next: 'ja',
      src: heroEn,
      switchLabel: 'Switch to Japanese',
      toggleLabel: '日本語'
    },
    ja: {
      alt: 'メンテナンス中です',
      next: 'en',
      src: heroJp,
      switchLabel: '英語に切り替え',
      toggleLabel: 'English'
    }
  };

  function initialLocale(): Locale {
    if (typeof window === 'undefined') return 'en';

    const saved = window.localStorage.getItem(storageKey);
    if (saved === 'en' || saved === 'ja') return saved;

    return window.navigator.language.toLowerCase().startsWith('ja') ? 'ja' : 'en';
  }

  let locale = initialLocale();
  $: current = heroes[locale];

  function toggleLocale() {
    locale = current.next;
    window.localStorage.setItem(storageKey, locale);
    document.documentElement.lang = locale;
  }

  $: if (typeof document !== 'undefined') {
    document.documentElement.lang = locale;
  }
</script>

<svelte:head>
  <title>{current.alt}</title>
  <meta name="description" content={current.alt} />
</svelte:head>

<main class="page" aria-label={current.alt}>
  <button
    class="language-button"
    type="button"
    aria-label={current.switchLabel}
    title={current.switchLabel}
    on:click={toggleLocale}
  >
    {current.toggleLabel}
  </button>

  <div class="hero-frame">
    <img class="hero" src={current.src} alt={current.alt} width="1200" height="800" decoding="async" />
  </div>
</main>
