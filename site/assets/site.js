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

  const query = document.querySelector('#help-query');
  if (!query) return;

  const topics = [...document.querySelectorAll('[data-help-topic]')];
  const groups = [...document.querySelectorAll('[data-help-group]')];
  const categories = [...document.querySelectorAll('[data-help-category]')];
  const status = document.querySelector('#help-results');
  const empty = document.querySelector('#no-help-results');

  const filter = () => {
    const terms = query.value.toLocaleLowerCase().trim().split(/\s+/).filter(Boolean);
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
      ? `${matches} topic${matches === 1 ? '' : 's'} match “${query.value.trim()}”.`
      : `Showing all ${topics.length} topics.`;
  };

  query.addEventListener('input', filter);
  document.addEventListener('keydown', (event) => {
    if (event.key === '/' && document.activeElement !== query) {
      event.preventDefault();
      query.focus();
    }
    if (event.key === 'Escape' && document.activeElement === query) {
      query.value = '';
      filter();
      query.blur();
    }
  });
})();
