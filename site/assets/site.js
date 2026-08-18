(() => {
  const toggle = document.querySelector('.nav-toggle');
  const nav = document.querySelector('#site-nav');
  if (toggle && nav) {
    toggle.addEventListener('click', () => {
      const open = toggle.getAttribute('aria-expanded') === 'true';
      toggle.setAttribute('aria-expanded', String(!open));
      nav.classList.toggle('is-open', !open);
    });
  }

  const helpQuery = document.querySelector('#help-query');
  let activeSearch = null;
  let clearActiveSearch = null;

  if (helpQuery) {
    const topics = [...document.querySelectorAll('[data-help-topic]')];
    const groups = [...document.querySelectorAll('[data-help-group]')];
    const categories = [...document.querySelectorAll('[data-help-category]')];
    const status = document.querySelector('#help-results');
    const empty = document.querySelector('#no-help-results');
    const filterHelp = () => {
      const terms = helpQuery.value.toLocaleLowerCase().trim().split(/\s+/).filter(Boolean);
      let matches = 0;
      topics.forEach((topic) => {
        const visible = terms.every((term) => topic.dataset.search.includes(term));
        topic.hidden = !visible;
        if (visible) matches += 1;
      });
      groups.forEach((group) => {
        group.hidden = !group.querySelector('[data-help-topic]:not([hidden])');
      });
      categories.forEach((category) => {
        category.hidden = !category.querySelector('[data-help-topic]:not([hidden])');
      });
      empty.hidden = matches !== 0;
      status.textContent = terms.length
        ? `${matches} topic${matches === 1 ? '' : 's'} match “${helpQuery.value.trim()}”.`
        : `Showing all ${topics.length} topics.`;
    };
    helpQuery.addEventListener('input', filterHelp);
    activeSearch = helpQuery;
    clearActiveSearch = () => {
      helpQuery.value = '';
      filterHelp();
    };
  }

  const shipQuery = document.querySelector('#ship-query');
  if (shipQuery) {
    const familyFilter = document.querySelector('#ship-family');
    const pathFilter = document.querySelector('#ship-path');
    const entries = [...document.querySelectorAll('[data-catalog-entry]')];
    const status = document.querySelector('#ship-results');
    const empty = document.querySelector('#no-ship-results');
    const filterCatalog = () => {
      const terms = shipQuery.value.toLocaleLowerCase().trim().split(/\s+/).filter(Boolean);
      let matches = 0;
      entries.forEach((entry) => {
        const textMatches = terms.every((term) => entry.dataset.search.includes(term));
        const familyMatches = !familyFilter.value || entry.dataset.family === familyFilter.value;
        const pathMatches = !pathFilter.value || entry.dataset.path === pathFilter.value;
        const visible = textMatches && familyMatches && pathMatches;
        entry.hidden = !visible;
        if (visible) matches += 1;
      });
      empty.hidden = matches !== 0;
      const filtered = terms.length || familyFilter.value || pathFilter.value;
      status.textContent = filtered
        ? `${matches} issued entr${matches === 1 ? 'y' : 'ies'} match current filters.`
        : `Showing all ${entries.length} issued entries.`;
    };
    shipQuery.addEventListener('input', filterCatalog);
    familyFilter.addEventListener('change', filterCatalog);
    pathFilter.addEventListener('change', filterCatalog);
    activeSearch = shipQuery;
    clearActiveSearch = () => {
      shipQuery.value = '';
      familyFilter.value = '';
      pathFilter.value = '';
      filterCatalog();
    };
  }

  if (!activeSearch) return;
  document.addEventListener('keydown', (event) => {
    const editing = ['INPUT', 'SELECT', 'TEXTAREA'].includes(document.activeElement.tagName);
    if (event.key === '/' && !editing) {
      event.preventDefault();
      activeSearch.focus();
    }
    if (event.key === 'Escape' && editing) {
      clearActiveSearch();
      document.activeElement.blur();
    }
  });
})();
