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
    const tonnageMin = document.querySelector('#ship-tonnage-min');
    const tonnageMax = document.querySelector('#ship-tonnage-max');
    const jumpMin = document.querySelector('#ship-jump-min');
    const jumpMax = document.querySelector('#ship-jump-max');
    const thrustMin = document.querySelector('#ship-thrust-min');
    const thrustMax = document.querySelector('#ship-thrust-max');
    const sortControls = [...document.querySelectorAll('input[name="ship-sort"]')];
    const index = document.querySelector('#catalog-index');
    const entries = [...document.querySelectorAll('[data-catalog-entry]')];
    const status = document.querySelector('#ship-results');
    const empty = document.querySelector('#no-ship-results');
    const rangeMatches = (value, minimum, maximum) => (
      (!minimum.value || value >= Number(minimum.value))
      && (!maximum.value || value <= Number(maximum.value))
    );
    const sortCatalog = () => {
      const selected = sortControls.find((control) => control.checked)?.value || 'catalog';
      const attribute = {
        catalog: 'catalogId',
        tonnage: 'tonnage',
        price: 'price',
        thrust: 'thrust',
        jump: 'jump',
      }[selected];
      entries.sort((left, right) => (
        Number(left.dataset[attribute]) - Number(right.dataset[attribute])
        || Number(left.dataset.catalogId) - Number(right.dataset.catalogId)
      ));
      entries.forEach((entry) => index.append(entry));
    };
    const filterCatalog = () => {
      const terms = shipQuery.value.toLocaleLowerCase().trim().split(/\s+/).filter(Boolean);
      let matches = 0;
      sortCatalog();
      entries.forEach((entry) => {
        const textMatches = terms.every((term) => entry.dataset.search.includes(term));
        const familyMatches = !familyFilter.value || entry.dataset.family === familyFilter.value;
        const pathMatches = !pathFilter.value || entry.dataset.path === pathFilter.value;
        const tonnageMatches = rangeMatches(Number(entry.dataset.tonnage), tonnageMin, tonnageMax);
        const jumpMatches = rangeMatches(Number(entry.dataset.jump), jumpMin, jumpMax);
        const thrustMatches = rangeMatches(Number(entry.dataset.thrust), thrustMin, thrustMax);
        const visible = textMatches && familyMatches && pathMatches
          && tonnageMatches && jumpMatches && thrustMatches;
        entry.hidden = !visible;
        if (visible) matches += 1;
      });
      empty.hidden = matches !== 0;
      const filtered = terms.length || familyFilter.value || pathFilter.value
        || tonnageMin.value || tonnageMax.value || jumpMin.value || jumpMax.value
        || thrustMin.value || thrustMax.value;
      status.textContent = filtered
        ? `${matches} issued entr${matches === 1 ? 'y' : 'ies'} match current filters.`
        : `Showing all ${entries.length} issued entries.`;
    };
    shipQuery.addEventListener('input', filterCatalog);
    [familyFilter, pathFilter, tonnageMin, tonnageMax, jumpMin, jumpMax,
      thrustMin, thrustMax, ...sortControls].forEach((control) => {
      control.addEventListener('change', filterCatalog);
    });
    window.addEventListener('pageshow', filterCatalog);
    filterCatalog();
    activeSearch = shipQuery;
    clearActiveSearch = () => {
      shipQuery.value = '';
      familyFilter.value = '';
      pathFilter.value = '';
      tonnageMin.value = '';
      tonnageMax.value = '';
      jumpMin.value = '';
      jumpMax.value = '';
      thrustMin.value = '';
      thrustMax.value = '';
      sortControls.forEach((control) => {
        control.checked = control.value === 'catalog';
      });
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
