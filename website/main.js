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
let lastScroll = 0;

window.addEventListener('scroll', () => {
  const scrollY = window.scrollY;
  if (scrollY > 50) {
    nav.classList.add('scrolled');
  } else {
    nav.classList.remove('scrolled');
  }
});

// Toggle Mobile Menu
navBurger?.addEventListener('click', () => {
  navLinks?.classList.toggle('active');
  navBurger?.classList.toggle('active');
});

// Smooth Scroll for anchor links
document.querySelectorAll('a[href^="#"]').forEach(anchor => {
  anchor.addEventListener('click', function(e) {
    e.preventDefault();
    if(navLinks?.classList.contains('active')) {
      navLinks.classList.remove('active');
      navBurger.classList.remove('active');
    }
    const target = document.querySelector(this.getAttribute('href'));
    if(target) {
      window.scrollTo({
        top: target.offsetTop - 80,
        behavior: 'smooth'
      });
    }
  });
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
document.addEventListener('mousemove', (e) => {
  const x = e.clientX / window.innerWidth;
  const y = e.clientY / window.innerHeight;
  glows.forEach((glow, index) => {
    const speed = index === 0 ? 30 : -20;
    glow.style.transform = `translate(${x * speed}px, ${y * speed}px)`;
  });
});

// ─── OS Detection & Download Links ───
const heroDownloadBtn = document.getElementById('hero-download-btn');
const installDownloadBtn = document.getElementById('install-download-btn');
const RELEASES_URL = 'https://github.com/IshimweIsaac/Lumina/releases/latest/download';
const FALLBACK_URL = 'https://github.com/IshimweIsaac/Lumina/releases';

function detectOS() {
  const ua = window.navigator.userAgent.toLowerCase();
  
  if (ua.includes('win')) return 'Windows';
  if (ua.includes('mac')) return 'macOS';
  if (ua.includes('linux')) return 'Linux';
  return 'Unknown';
}

function getDownloadLink(os) {
  // Pointing to the newly created GUI installers
  switch (os) {
    case 'Windows': return `${RELEASES_URL}/LuminaSetup.exe`;
    case 'macOS': return `${RELEASES_URL}/lumina-setup-macos.pkg`;
    case 'Linux': return `${RELEASES_URL}/lumina-setup-linux-amd64.deb`;
    default: return FALLBACK_URL;
  }
}

const os = detectOS();
const downloadUrl = getDownloadLink(os);

if (heroDownloadBtn && os !== 'Unknown') {
  heroDownloadBtn.innerHTML = heroDownloadBtn.innerHTML.replace('Download Lumina', `Download for ${os}`);
  heroDownloadBtn.href = downloadUrl;
}

if (installDownloadBtn && os !== 'Unknown') {
  installDownloadBtn.innerHTML = `Download for ${os}`;
  installDownloadBtn.href = downloadUrl;
}

// ─── Documentation Logic ───

// Basic Syntax Highlighter for Lumina
function highlightLumina(code) {
  const tokens = [];
  let ti = 0;

  // Escape HTML entities first
  let h = code
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');

  // Protect strings from further regex matches
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
  h = h.replace(/\b(entity|external|sync on|rule|when|becomes|then|update|to|show|alert|severity|source|message|code|payload|on clear|every|for|let|fn|import|aggregate|over|cooldown|if|else|not|and|or)\b/g, '<span class="kw">$1</span>');

  // Types
  h = h.replace(/\b(Number|Text|Boolean)\b/g, '<span class="type">$1</span>');

  // Built-ins
  h = h.replace(/\b(len|min|max|sum|append|head|tail|at|count|avg|any|all|prev)\b(?=\()/g, '<span class="builtin">$1</span>');

  // Operators — use HTML-escaped forms for < and >
  h = h.replace(/(:=|&lt;=|&gt;=|&lt;|&gt;|-&gt;|@doc|@range|@affects|==|!=)/g, '<span class="op">$1</span>');

  // Numbers
  h = h.replace(/\b(\d+(?:\.\d+)?)\b/g, '<span class="num">$1</span>');

  // Booleans
  h = h.replace(/\b(true|false)\b/g, '<span class="bool">$1</span>');

  // Restore protected tokens
  tokens.forEach(({ ph, html }) => {
    h = h.split(ph).join(html);
  });

  return h;
}

// Auto-highlight static Lumina code blocks in the HTML
document.querySelectorAll('code.language-lumina').forEach(block => {
  block.innerHTML = highlightLumina(block.textContent);
});

const docsNav = document.getElementById('docs-nav');
const docsContent = document.getElementById('docs-content');

function renderDocTab(tabKey) {
  const docData = DOCS[tabKey];
  if (!docData) return;

  // Update active tab button
  document.querySelectorAll('.docs-nav__tab').forEach(btn => {
    btn.classList.toggle('active', btn.dataset.tab === tabKey);
  });

  // Hide or show the examples section based on the active tab
  const examplesSection = document.getElementById('examples');
  if (examplesSection) {
    examplesSection.style.display = (tabKey === 'getting_started') ? 'block' : 'none';
  }

  // Render content
  let html = `<div class="docs-pane fade-in"><h3 class="docs-pane__title">${docData.title}</h3>`;
  
  docData.sections.forEach(sec => {
    html += `<div class="docs-section-card">`;
    html += `<h4 class="docs-section-card__title">${sec.heading} ${sec.badge ? `<span class="badge">${sec.badge}</span>` : ''}</h4>`;
    if (sec.text) html += `<p class="docs-section-card__text">${sec.text}</p>`;
    
    // Render Table if present
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

    // Render Code window if present
    if (sec.code) {
      html += `
        <div class="code-window code-window--doc">
          ${sec.file ? `<div class="code-window__header"><div class="code-window__dots"><img src="/logo.png" alt="Lumina" width="14" height="14" /></div><span class="code-window__filename">${sec.file}</span></div>` : ''}
          <pre class="code-window__body"><code>${highlightLumina(sec.code)}</code></pre>
        </div>
      `;
    }
    html += `</div>`;
  });
  
  html += `</div>`;
  docsContent.innerHTML = html;
}

// Attach listeners to tabs
if (docsNav) {
  document.querySelectorAll('.docs-nav__tab[data-tab]').forEach(btn => {
    btn.addEventListener('click', () => {
      renderDocTab(btn.dataset.tab);
    });
  });
  // Initial render
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
  
  // Re-observe newly injected elements
  examplesGrid.querySelectorAll('.reveal').forEach((el) => {
    revealObserver.observe(el);
  });
}

renderExamples();
