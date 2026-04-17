/* ────────────────────────────────────────────────────
   Lumina Website — Interactions & Animations
   ──────────────────────────────────────────────────── */

import './style.css';
import { DOCS, EXAMPLES } from './docs.js';

// ─── Scroll-Triggered Reveal Animations ───
const observerOptions = {
  threshold: 0.1,
  rootMargin: '0px 0px -40px 0px',
};

const revealObserver = new IntersectionObserver((entries) => {
  entries.forEach((entry) => {
    if (entry.isIntersecting) {
      entry.target.classList.add('visible');
      revealObserver.unobserve(entry.target);
    }
  });
}, observerOptions);

document.querySelectorAll('.reveal').forEach((el) => {
  revealObserver.observe(el);
});

// ─── Navbar Scroll Effect & Mobile Menu ───
const nav = document.getElementById('nav');
const navLinks = document.getElementById('nav-links');
const navBurger = document.getElementById('nav-burger');

window.addEventListener('scroll', () => {
  if (window.scrollY > 50) {
    nav?.classList.add('scrolled');
  } else {
    nav?.classList.remove('scrolled');
  }
});

// Toggle Mobile Menu — uses .active class (matches CSS)
navBurger?.addEventListener('click', () => {
  navLinks?.classList.toggle('active');
  navBurger?.classList.toggle('active');
});

// Close mobile menu when clicking a link
navLinks?.querySelectorAll('.nav__link').forEach(link => {
  link.addEventListener('click', () => {
    navLinks?.classList.remove('active');
    navBurger?.classList.remove('active');
  });
});

// Close mobile menu on outside click
document.addEventListener('click', (e) => {
  if (navLinks?.classList.contains('active') &&
      !navLinks.contains(e.target) &&
      !navBurger?.contains(e.target)) {
    navLinks.classList.remove('active');
    navBurger?.classList.remove('active');
  }
});

// ─── Smooth Scroll ───
function scrollToHashTarget(hash) {
  const target = document.querySelector(hash);
  if (target) {
    window.scrollTo({
      top: target.offsetTop - 80,
      behavior: 'smooth'
    });
    window.history.replaceState(null, null, window.location.pathname);
  }
}
// Expose globally for any inline onclick handlers
window.scrollToHashTarget = scrollToHashTarget;

document.querySelectorAll('a').forEach(anchor => {
  anchor.addEventListener('click', function(e) {
    const href = this.getAttribute('href');
    if (!href) return;

    const hashMatches = href.match(/(#.*)$/);
    if (!hashMatches) return;

    const hash = hashMatches[1];
    const isRoot = window.location.pathname === '/' || window.location.pathname === '/index.html';
    if (href.startsWith('/#') && !isRoot) return;
    if (href.startsWith('/docs.html#') && window.location.pathname !== '/docs.html') return;

    e.preventDefault();
    if (navLinks?.classList.contains('active')) {
      navLinks.classList.remove('active');
      navBurger?.classList.remove('active');
    }
    scrollToHashTarget(hash);
  });
});

// ─── OS Detection & Download ───
function detectOS() {
  const ua = window.navigator.userAgent.toLowerCase();
  if (ua.includes('win')) return 'Windows';
  if (ua.includes('mac')) return 'macOS';
  if (ua.includes('linux')) return 'Linux';
  return 'Unknown';
}

function getDownloadLink(os) {
  switch (os) {
    case 'Windows': return `/LuminaSetup.exe`;
    case 'macOS': return `/lumina-setup-macos.pkg`;
    case 'Linux': return `/lumina-setup-linux-amd64.deb`;
    default: return `/install.sh`;
  }
}

function getOSIcon(os) {
  switch (os) {
    case 'Windows': return '⊞';
    case 'macOS': return '⌘';
    case 'Linux': return '🐧';
    default: return '↓';
  }
}

function highlightDetectedOS() {
  const os = detectOS();
  const keyMap = { 'windows': 'windows', 'macos': 'macos', 'linux': 'linux' };
  const btnId = keyMap[os.toLowerCase()];
  const btn = document.getElementById(`download-${btnId}`);
  if (btn) {
    btn.classList.remove('btn--outline');
    btn.classList.add('btn--primary');
    if (!btn.querySelector('.os-recommended')) {
      const label = document.createElement('span');
      label.className = 'os-recommended';
      label.innerText = ' (Recommended)';
      label.style.cssText = 'font-size:0.78rem;opacity:0.8;margin-left:0.4rem;';
      btn.appendChild(label);
    }
  }
}

// Set hero download button — runs immediately
const heroDownloadBtn = document.getElementById('hero-download-btn');
const os = detectOS();
const downloadUrl = getDownloadLink(os);

// Immediately set the hero button
if (heroDownloadBtn && os !== 'Unknown') {
  heroDownloadBtn.innerHTML = `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>Download for Desktop`;
  heroDownloadBtn.href = downloadUrl;
}

// Update mobile CTA
const mobileCta = document.querySelector('.nav__mobile-cta');
if (mobileCta && os !== 'Unknown') {
  mobileCta.innerHTML = `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>Download for Desktop`;
  mobileCta.href = downloadUrl;
}

window.addEventListener('DOMContentLoaded', () => {
  highlightDetectedOS();

  if (window.location.hash) {
    setTimeout(() => scrollToHashTarget(window.location.hash), 100);
  }
});

// ─── Copy to Clipboard ───
document.querySelectorAll('.install__copy').forEach(btn => {
  btn.addEventListener('click', () => {
    const text = btn.getAttribute('data-copy');
    navigator.clipboard.writeText(text).then(() => {
      const originalHTML = btn.innerHTML;
      btn.innerHTML = '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#22c55e" stroke-width="2"><polyline points="20 6 9 17 4 12"></polyline></svg>';
      setTimeout(() => btn.innerHTML = originalHTML, 2000);
    });
  });
});

// ─── Parallax Glow ───
const glows = document.querySelectorAll('.hero__glow');
if (glows.length && window.matchMedia('(min-width: 768px)').matches) {
  document.addEventListener('mousemove', (e) => {
    const x = e.clientX / window.innerWidth;
    const y = e.clientY / window.innerHeight;
    glows.forEach((glow, index) => {
      const speed = index === 0 ? 30 : -20;
      glow.style.transform = `translate(${x * speed}px, ${y * speed}px)`;
    });
  });
}

// ─── Documentation Logic ───

function parseMarkdown(md) {
  if (!md) return '';
  let html = md
    .replace(/^### (.*$)/gm, '<h5 id="$1" class="docs-section-h5">$1</h5>')
    .replace(/^## (.*$)/gm, '<h4 id="$1" class="docs-section-h4">$1</h4>')
    .replace(/^# (.*$)/gm, '<h3 class="docs-section-h3">$1</h3>')
    .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
    .replace(/\*(.*?)\*/g, '<em>$1</em>')
    .replace(/`(.*?)`/g, '<code>$1</code>');

  // Parse lists: group consecutive lines starting with '- '
  html = html.replace(/(?:^- .*(?:\n|$))+/gm, match => {
    const items = match.trim().split('\n').map(line => `<li>${line.replace(/^- /, '')}</li>`).join('');
    return `<ul class="docs-list">${items}</ul>`;
  });

  html = html
    .replace(/\n\n/g, '</p><p>')
    .replace(/\n/g, '<br/>')
    // Remove unwanted <br/> tags around or inside block elements
    .replace(/<br\/>(?=<ul|<h|<p|<\/ul)/g, '')
    .replace(/(<\/ul>|<\/h3>|<\/h4>|<\/h5>|<\/p>)<br\/>/g, '$1');

  return html;
}

// Syntax Highlighter for Lumina
function highlightLumina(code) {
  if (!code) return '';
  const tokens = [];
  let ti = 0;

  let h = code
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');

  // Protect strings
  h = h.replace(/"((?:\\.|[^"])*)"/g, (m) => {
    const ph = `\x00S${ti}\x00`;
    tokens.push({ ph, html: `<span class="str">${m}</span>` });
    ti++;
    return ph;
  });

  // Protect comments
  h = h.replace(/(--.*$)/gm, (m) => {
    const ph = `\x00C${ti}\x00`;
    tokens.push({ ph, html: `<span class="comment">${m}</span>` });
    ti++;
    return ph;
  });

  // Keywords
  h = h.replace(/\b(entity|external|sync on|rule|when|becomes|then|update|to|show|alert|severity|source|message|code|payload|on clear|every|for|let|fn|import|aggregate|over|cooldown|if|else|not|and|or|ref|write|times within)\b/g, '<span class="kw">$1</span>');

  // Types
  h = h.replace(/\b(Number|Text|Boolean|Timestamp)\b/g, '<span class="type">$1</span>');

  // Built-ins
  h = h.replace(/\b(len|min|max|sum|append|head|tail|at|count|avg|any|all|prev|now)\b(?=\()/g, '<span class="builtin">$1</span>');

  // Operators
  h = h.replace(/(:=|&lt;=|&gt;=|&lt;|&gt;|-&gt;|@doc|@range|@affects|==|!=)/g, '<span class="op">$1</span>');

  // Numbers
  h = h.replace(/\b(\d+(?:\.\d+)?)\b/g, '<span class="num">$1</span>');

  // Booleans
  h = h.replace(/\b(true|false)\b/g, '<span class="bool">$1</span>');

  // Restore
  tokens.forEach(({ ph, html }) => {
    h = h.split(ph).join(html);
  });

  return h;
}

// Auto-highlight static Lumina code blocks
document.querySelectorAll('code.language-lumina').forEach(block => {
  block.innerHTML = highlightLumina(block.textContent);
});

// Icon Library for Getting Started
function getIcon(name) {
  const icons = {
    'trending-up': '<polyline points="23 6 13.5 15.5 8.5 10.5 1 18"></polyline><polyline points="17 6 23 6 23 12"></polyline>',
    'cpu': '<rect x="4" y="4" width="16" height="16" rx="2" ry="2"></rect><rect x="9" y="9" width="6" height="6"></rect><line x1="9" y1="1" x2="9" y2="4"></line><line x1="15" y1="1" x2="15" y2="4"></line><line x1="9" y1="20" x2="9" y2="23"></line><line x1="15" y1="20" x2="15" y2="23"></line><line x1="20" y1="9" x2="23" y2="9"></line><line x1="20" y1="15" x2="23" y2="15"></line><line x1="1" y1="9" x2="4" y2="9"></line><line x1="1" y1="15" x2="4" y2="15"></line>',
    'server': '<rect x="2" y="2" width="20" height="8" rx="2" ry="2"></rect><rect x="2" y="14" width="20" height="8" rx="2" ry="2"></rect><line x1="6" y1="6" x2="6.01" y2="6"></line><line x1="6" y1="18" x2="6.01" y2="18"></line>',
    'shield': '<path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"></path>',
    'box': '<path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"></path><polyline points="3.27 6.96 12 12.01 20.73 6.96"></polyline><line x1="12" y1="22.08" x2="12" y2="12"></line>',
    'database': '<ellipse cx="12" cy="5" rx="9" ry="3"></ellipse><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"></path><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"></path>',
    'zap': '<polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon>',
    'bell': '<path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"></path><path d="M13.73 21a2 2 0 0 1-3.46 0"></path>',
    'truck': '<rect x="1" y="3" width="15" height="13"></rect><polygon points="16 8 20 8 23 11 23 16 16 16 16 8"></polygon><circle cx="5.5" cy="18.5" r="2.5"></circle><circle cx="18.5" cy="18.5" r="2.5"></circle>',
    'factory': '<path d="M2 20a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V8l-7 5V8l-7 5V4a2 2 0 0 0-2-2H4a2 2 0 0 0-2 2Z"></path><path d="M17 18h1"></path><path d="M12 18h1"></path><path d="M7 18h1"></path>',
    'building': '<rect x="4" y="2" width="16" height="20" rx="2" ry="2"></rect><line x1="9" y1="22" x2="9" y2="2"></line><line x1="15" y1="22" x2="15" y2="2"></line><line x1="4" y1="6" x2="20" y2="6"></line><line x1="4" y1="10" x2="20" y2="10"></line><line x1="4" y1="14" x2="20" y2="14"></line><line x1="4" y1="18" x2="20" y2="18"></line>',
    'sprout': '<path d="m12 22 4-4-3-3"></path><path d="M9 18c-4.51 2-5-3-7-4"></path><path d="M19 14c2.09 1 2.5-4 7-6"></path><path d="M9 13a5 5 0 0 1 5-5c2 0 2-4 2-4"></path><path d="M5 11c0-1.28.2-2.5 1-3.5C7.64 5.5 10 3 10 3s1.5 2 1.5 5.5c0 1.28-.2 2.5-1 3.5C8.36 13.5 6 16 6 16s-1.5-2-1.5-5.5Z"></path>',
    'play': '<circle cx="12" cy="12" r="10"></circle><polygon points="10 8 16 12 10 16 10 8"></polygon>',
    'book': '<path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"></path><path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"></path>',
    'terminal': '<polyline points="4 17 10 11 4 5"></polyline><line x1="12" y1="19" x2="20" y2="19"></line>',
    'message-square': '<path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path>'
  };

  const path = icons[name] || '';
  return `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">${path}</svg>`;
}

// Render a single diagnostic object (error/warning) into a rich HTML card
function renderDiagnosticCard(diag) {
  return `
    <div class="diagnostic-card ${diag.code.startsWith('R') ? 'diagnostic-card--runtime' : ''}">
      <div class="diagnostic-card__header">
        <span class="diagnostic-card__code">${diag.code}</span>
        <span class="diagnostic-card__msg">${diag.message}</span>
      </div>
      <div class="diagnostic-card__loc">
        Line ${diag.location.line}, Col ${diag.location.col}
      </div>
      ${diag.source_line ? `
      <div class="diagnostic-card__snippet">
        <pre><code>${diag.source_line}\n${' '.repeat(diag.location.col - 1) + '^'.repeat(diag.location.len)}</code></pre>
      </div>` : ''}
      ${diag.help ? `
      <div class="diagnostic-card__help">
        <strong>💡 Tip:</strong> ${diag.help}
      </div>` : ''}
    </div>
  `;
}

// Debounce helper
function debounce(fn, ms) {
  let timeout;
  return function() {
    clearTimeout(timeout);
    timeout = setTimeout(() => fn.apply(this, arguments), ms);
  };
}

const docsContent = document.getElementById('docs-content');
const docsBreadcrumbs = document.getElementById('docs-breadcrumbs');
const docsSidebar = document.getElementById('docs-sidebar');
const docsTrigger = document.getElementById('docs-trigger-sidebar');
const tocLinks = document.getElementById('toc-links');
const footerNav = document.getElementById('docs-footer-nav');
const searchInput = document.getElementById('docs-search-input');

// ── Sidebar Overlay (mobile docs) ──
const sidebarOverlay = document.createElement('div');
sidebarOverlay.className = 'docs-sidebar-overlay';
sidebarOverlay.id = 'docs-sidebar-overlay';
if (docsSidebar) {
  docsSidebar.parentElement.insertBefore(sidebarOverlay, docsSidebar);
}

function openDocsSidebar() {
  docsSidebar?.classList.add('active');
  sidebarOverlay?.classList.add('active');
  document.body.style.overflow = 'hidden';
}

function closeDocsSidebar() {
  docsSidebar?.classList.remove('active');
  sidebarOverlay?.classList.remove('active');
  document.body.style.overflow = '';
}

docsTrigger?.addEventListener('click', openDocsSidebar);
sidebarOverlay?.addEventListener('click', closeDocsSidebar);

// Render Table of Contents
function renderTOC() {
  if (!tocLinks) return;
  const headings = docsContent.querySelectorAll('h4, h5');
  let html = '';
  headings.forEach(h => {
    const id = h.innerText.replace(/\s+/g, '-').toLowerCase();
    h.id = id;
    html += `<a href="#${id}" class="docs-toc__link">${h.innerText}</a>`;
  });
  tocLinks.innerHTML = html;
}

// Render Breadcrumbs
function renderBreadcrumbs(tabKey) {
  if (!docsBreadcrumbs) return;
  const data = DOCS[tabKey];
  if (!data) return;
  docsBreadcrumbs.innerHTML = `
    <span class="breadcrumb__item">Docs</span>
    <span class="breadcrumb__separator">/</span>
    <span class="breadcrumb__item active">${data.title}</span>
  `;
}

// Render Footer Navigation
function renderFooterNav(tabKey) {
  if (!footerNav) return;
  const keys = Object.keys(DOCS);
  const idx = keys.indexOf(tabKey);

  const prev = idx > 0 ? keys[idx-1] : null;
  const next = idx < keys.length - 1 ? keys[idx+1] : null;

  let html = '';
  if (prev) {
    html += `
      <div class="docs-footer-btn" onclick="document.querySelector('[data-tab=${prev}]').click()">
        <span class="docs-footer-btn__label">← Previous</span>
        <span class="docs-footer-btn__title">${DOCS[prev].title}</span>
      </div>
    `;
  } else {
    html += '<div></div>';
  }

  if (next) {
    html += `
      <div class="docs-footer-btn" onclick="document.querySelector('[data-tab=${next}]').click()">
        <span class="docs-footer-btn__label">Next →</span>
        <span class="docs-footer-btn__title">${DOCS[next].title}</span>
      </div>
    `;
  }
  footerNav.innerHTML = html;
}

function renderDocTab(tabKey) {
  const docData = DOCS[tabKey];
  if (!docData) return;

  // Update active tab button
  document.querySelectorAll('.docs-nav__tab').forEach(btn => {
    btn.classList.toggle('active', btn.dataset.tab === tabKey);
  });

  // Mobile drawer close
  closeDocsSidebar();
  const mobileTabTitle = document.getElementById('docs-mobile-active-tab');
  if (mobileTabTitle) mobileTabTitle.innerText = docData.title;

  // Hide or show examples section
  const examplesSection = document.getElementById('examples');
  if (examplesSection) {
    examplesSection.style.display = (tabKey === 'getting_started') ? 'block' : 'none';
  }

  // Render content
  let html = `<div class="docs-pane"><h3 class="docs-pane__title">${docData.title}</h3>`;

  if (docData.intro) {
    html += `<p class="docs-pane__intro">${docData.intro}</p>`;
  }

  docData.sections.forEach(sec => {
    html += `<div class="docs-section-card${sec.tagline ? ' gs-hook' : ''}">`;
    html += `<h4 class="docs-section-card__title">${sec.heading} ${sec.badge ? `<span class="badge">${sec.badge}</span>` : ''}</h4>`;
    if (sec.tagline) html += `<p class="gs-tagline">${sec.tagline}</p>`;
    if (sec.text) html += `<div class="docs-section-card__text">${parseMarkdown(sec.text)}</div>`;

    // Render Table
    if (sec.table && sec.table.length > 0) {
      html += `<div class="docs-table-wrapper"><table class="docs-table">`;
      sec.table.forEach((row, rIdx) => {
        html += `<tr>`;
        row.forEach(cell => {
          html += rIdx === 0 ? `<th>${cell}</th>` : `<td>${cell}</td>`;
        });
        html += `</tr>`;
      });
      html += `</table></div>`;
    }

    // Render Code window
    if (sec.code) {
      const isInteractive = sec.file && sec.file.endsWith('.lum');

      if (isInteractive) {
        html += `
          <div class="code-window code-window--doc interactive-cell">
            <div class="code-window__header">
              <div class="code-window__dots"><img src="/logo.png" alt="Lumina" width="14" height="14" /></div>
              <span class="code-window__filename">${sec.file}</span>
              <button class="btn-run-cell">▶ Run Live</button>
            </div>
            <div class="interactive-editor-wrap">
              <textarea class="interactive-textarea" spellcheck="false">${sec.code.replace(/</g, '&lt;').replace(/>/g, '&gt;')}</textarea>
              <pre class="interactive-highlight"><code>${highlightLumina(sec.code)}</code></pre>
            </div>
            <div class="interactive-output" style="display: none;"></div>
          </div>
        `;
      } else {
        html += `
          <div class="code-window code-window--doc">
            ${sec.file ? `<div class="code-window__header"><div class="code-window__dots"><img src="/logo.png" alt="Lumina" width="14" height="14" /></div><span class="code-window__filename">${sec.file}</span></div>` : ''}
            <pre class="code-window__body"><code>${highlightLumina(sec.code)}</code></pre>
          </div>
        `;
      }
    }

    // ── Getting Started: Install CTA ──
    if (sec.installBlock) {
      html += `<div class="gs-install">
        <div class="gs-install__cmd-wrap">
          <div class="gs-install__label">Terminal</div>
          <div class="gs-install__cmd">
            <code>curl -fsSL https://lumina-lang.dev/install.sh | sh</code>
            <button class="gs-install__copy" onclick="navigator.clipboard.writeText('curl -fsSL https://lumina-lang.dev/install.sh | sh').then(()=>{this.textContent='✓ Copied';setTimeout(()=>this.textContent='📋',2000)})">📋</button>
          </div>
        </div>
        <div class="gs-install__divider"><span>or</span></div>
        <a href="/" class="btn btn--primary btn--lg gs-install__playground">🎮 Try it instantly in the Playground</a>
      </div>`;
    }

    // ── Getting Started: Concepts Grid ──
    if (sec.concepts) {
      html += `<div class="gs-concepts">`;
      sec.concepts.forEach(c => {
        html += `<div class="gs-concept"><span class="gs-concept__icon">${getIcon(c.icon)}</span><span class="gs-concept__term">${c.term}</span><span class="gs-concept__desc">= ${c.desc}</span></div>`;
      });
      html += `</div>`;
    }

    // ── Getting Started: Guided Steps ──
    if (sec.guidedSteps) {
      html += `<div class="gs-steps">`;
      sec.guidedSteps.forEach(s => {
        const isStepInteractive = s.file && s.file.endsWith('.lum');
        html += `<div class="gs-step"><div class="gs-step__header"><span class="gs-step__number">0${s.step}</span><div><h5 class="gs-step__title">${s.title}</h5><p class="gs-step__desc">${s.desc}</p></div></div>`;
        if (isStepInteractive) {
          html += `<div class="code-window code-window--doc interactive-cell"><div class="code-window__header"><div class="code-window__dots"><img src="/logo.png" alt="Lumina" width="14" height="14" /></div><span class="code-window__filename">${s.file}</span><button class="btn-run-cell">▶ Run Live</button></div><div class="interactive-editor-wrap"><textarea class="interactive-textarea" spellcheck="false">${s.code.replace(/</g,'&lt;').replace(/>/g,'&gt;')}</textarea><pre class="interactive-highlight"><code>${highlightLumina(s.code)}</code></pre></div><div class="interactive-output" style="display:none;"></div></div>`;
        } else {
          html += `<div class="code-window code-window--doc"><pre class="code-window__body"><code>${highlightLumina(s.code)}</code></pre></div>`;
        }
        html += `</div>`;
      });
      html += `</div>`;
    }

    // ── Getting Started: Comparison ──
    if (sec.comparison) {
      html += `<div class="gs-comparison">
        <div class="gs-comparison__col gs-comparison__col--old"><h5>${sec.comparison.traditional.title}</h5><pre><code>${sec.comparison.traditional.code.replace(/</g,'&lt;').replace(/>/g,'&gt;')}</code></pre></div>
        <div class="gs-comparison__vs">vs</div>
        <div class="gs-comparison__col gs-comparison__col--new"><h5>${sec.comparison.lumina.title}</h5><pre><code>${highlightLumina(sec.comparison.lumina.code)}</code></pre></div>
      </div>`;
    }

    // ── Getting Started: Use Cases ──
    if (sec.usecases) {
      html += `<div class="gs-usecases">`;
      sec.usecases.forEach(uc => {
        html += `<div class="gs-usecase"><span class="gs-usecase__icon">${getIcon(uc.icon)}</span><div class="gs-usecase__info"><strong>${uc.title}</strong><span>${uc.desc}</span></div></div>`;
      });
      html += `</div>`;
    }

    // ── Getting Started: Next Steps CTA ──
    if (sec.nextSteps) {
      html += `<div class="gs-nextsteps">`;
      sec.nextSteps.forEach(ns => {
        if (ns.action) {
          html += `<button class="gs-nextstep" onclick="document.querySelector('[data-tab=${ns.action}]').click()"><span class="gs-nextstep__icon">${getIcon(ns.icon)}</span><strong>${ns.title}</strong><span>${ns.desc}</span></button>`;
        } else {
          html += `<a href="${ns.href}" class="gs-nextstep" ${ns.href.startsWith('http')?'target="_blank"':''}><span class="gs-nextstep__icon">${getIcon(ns.icon)}</span><strong>${ns.title}</strong><span>${ns.desc}</span></a>`;
        }
      });
      html += `</div>`;
    }

    html += `</div>`;
  });

  html += `</div>`;
  docsContent.innerHTML = html;

  // ── Initialize Interactive Playgrounds ──
  document.querySelectorAll('.interactive-cell').forEach(cell => {
    const textarea = cell.querySelector('.interactive-textarea');
    const highlight = cell.querySelector('.interactive-highlight code');
    const highlightPre = cell.querySelector('.interactive-highlight');
    const runBtn = cell.querySelector('.btn-run-cell');
    const outputDiv = cell.querySelector('.interactive-output');

    if (textarea && highlight) {
      // Sync syntax styling live
      textarea.addEventListener('input', () => {
        highlight.innerHTML = highlightLumina(textarea.value);
      });
      // Sync scroll bars
      textarea.addEventListener('scroll', () => {
        highlightPre.scrollTop = textarea.scrollTop;
        highlightPre.scrollLeft = textarea.scrollLeft;
      });

      textarea.addEventListener('keydown', (e) => {
        if(e.key === 'Tab') {
          e.preventDefault();
          const start = textarea.selectionStart;
          textarea.value = textarea.value.substring(0, start) + "  " + textarea.value.substring(textarea.selectionEnd);
          textarea.selectionStart = textarea.selectionEnd = start + 2;
          highlight.innerHTML = highlightLumina(textarea.value);
        }
      });

      // --- Advanced Real-Time Validation ---
      const validate = async () => {
        try {
          const { runLuminaScript } = await import('/playground.js');
          const result = await runLuminaScript(textarea.value);
          
          if (result.error && Array.isArray(result.error)) {
             cell.classList.add('cell--has-error');
             cell.classList.remove('cell--valid');
          } else {
             cell.classList.remove('cell--has-error');
             cell.classList.add('cell--valid');
          }
        } catch(e) {}
      };
      
      const debouncedValidate = debounce(validate, 500);
      textarea.addEventListener('input', () => {
        highlight.innerHTML = highlightLumina(textarea.value);
        debouncedValidate();
      });
    }

    if (runBtn && outputDiv && textarea) {
      runBtn.addEventListener('click', async () => {
        runBtn.innerText = "Running...";
        runBtn.disabled = true;
        outputDiv.style.display = "block";
        outputDiv.innerHTML = '<div class="loader-wrap"><div class="lumina-pulse"></div><span>Analyzing Reactive Graph...</span></div>';
        
        try {
          const { runLuminaScript } = await import('/playground.js');
          const result = await runLuminaScript(textarea.value);
          
          if (result.error) {
            if (Array.isArray(result.error)) {
              let errHTML = `<div class="terminal-error-header">Found ${result.error.length} issue${result.error.length > 1 ? 's' : ''} during analysis:</div>`;
              result.error.forEach(diag => {
                errHTML += renderDiagnosticCard(diag);
              });
              outputDiv.innerHTML = errHTML;
            } else {
              outputDiv.innerHTML = `<div class="terminal-error">Unexpected engine error: ${result.error}</div>`;
            }
          } else {
            let outHTML = '';
            if (result.output) {
               const logs = result.output.split('\n').map(l => `<div class="log-line"><span class="log-bullet">»</span> ${l}</div>`).join('');
               outHTML += `<div class="terminal-output"><strong>Execution Logs (${result.ticks} cycle settling):</strong><br/>${logs}</div>`;
            }
            
            if (result.state) {
               outHTML += `<div class="terminal-state"><strong>Resolved State Tree:</strong><br/><pre class="state-json">${result.state}</pre></div>`;
            }
            
            if (!outHTML) outHTML = `<div class="terminal-success">✓ Script settled with zero alerts or logs. System state is stable.</div>`;
            
            outputDiv.innerHTML = outHTML;
          }
        } catch (e) {
          outputDiv.innerHTML = `<div class="terminal-error">Workbench failure: ${e}</div>`;
        }
        runBtn.innerText = "▶ Run Live";
        runBtn.disabled = false;
      });
    }
  });

  renderBreadcrumbs(tabKey);
  renderTOC();
  renderFooterNav(tabKey);
  window.scrollTo(0, 0);
}

// Search Logic
searchInput?.addEventListener('input', (e) => {
  const query = e.target.value.toLowerCase().trim();
  if (!query) return;

  for (const [key, category] of Object.entries(DOCS)) {
    if (category.title.toLowerCase().includes(query)) {
      renderDocTab(key);
      break;
    }
    const match = category.sections.find(s => s.heading.toLowerCase().includes(query));
    if (match) {
      renderDocTab(key);
      setTimeout(() => {
        const id = match.heading.replace(/\s+/g, '-').toLowerCase();
        const el = document.getElementById(id);
        if (el) el.scrollIntoView({ behavior: 'smooth' });
      }, 100);
      break;
    }
  }
});

// Attach listeners to tabs
if (docsContent) {
  document.querySelectorAll('.docs-nav__tab[data-tab]').forEach(btn => {
    btn.addEventListener('click', () => {
      renderDocTab(btn.dataset.tab);
    });
  });
  renderDocTab('getting_started');
}

// ─── Examples Render ───
const examplesGrid = document.getElementById('examples-grid');

function renderExamples() {
  if (!examplesGrid) return;
  let html = '';
  EXAMPLES.forEach(ex => {
    html += `
      <div class="example-card reveal">
        <div class="example-card__info">
          <div class="example-card__tags">
            ${ex.tags.map(t => `<span class="tag">${t}</span>`).join('')}
          </div>
          <h3 class="example-card__title">${ex.title}</h3>
          <p class="example-card__desc">${ex.desc}</p>
        </div>
        <div class="code-window code-window--sm">
          <div class="code-window__header">
            <span class="code-window__filename">${ex.file}</span>
          </div>
          <pre class="code-window__body"><code>${highlightLumina(ex.code)}</code></pre>
        </div>
      </div>
    `;
  });
  examplesGrid.innerHTML = html;

  document.querySelectorAll('.reveal').forEach((el) => {
    revealObserver.observe(el);
  });
}

renderExamples();
