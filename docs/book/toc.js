// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded "><a href="overview/index.html"><strong aria-hidden="true">1.</strong> Overview</a></li><li class="chapter-item expanded "><a href="getting-started/installation.html"><strong aria-hidden="true">2.</strong> Installation</a></li><li class="chapter-item expanded "><a href="getting-started/quickstart.html"><strong aria-hidden="true">3.</strong> Quick Start</a></li><li class="chapter-item expanded "><a href="getting-started/configuration.html"><strong aria-hidden="true">4.</strong> Configuration</a></li><li class="chapter-item expanded "><a href="features/ai-features.html"><strong aria-hidden="true">5.</strong> AI Features</a></li><li class="chapter-item expanded "><a href="features/security.html"><strong aria-hidden="true">6.</strong> Security Features</a></li><li class="chapter-item expanded "><a href="features/performance.html"><strong aria-hidden="true">7.</strong> Performance</a></li><li class="chapter-item expanded "><a href="features/collaboration.html"><strong aria-hidden="true">8.</strong> Collaboration</a></li><li class="chapter-item expanded "><a href="user-guide/index.html"><strong aria-hidden="true">9.</strong> Getting Started</a></li><li class="chapter-item expanded "><a href="user-guide/BASIC_USAGE.html"><strong aria-hidden="true">10.</strong> Basic Usage</a></li><li class="chapter-item expanded "><a href="user-guide/troubleshooting.html"><strong aria-hidden="true">11.</strong> Troubleshooting</a></li><li class="chapter-item expanded "><a href="development/contributing.html"><strong aria-hidden="true">12.</strong> Contributing Guide</a></li><li class="chapter-item expanded "><a href="development/ARCHITECTURE.html"><strong aria-hidden="true">13.</strong> Architecture</a></li><li class="chapter-item expanded "><a href="development/CODING_STANDARDS.html"><strong aria-hidden="true">14.</strong> Coding Standards</a></li><li class="chapter-item expanded "><a href="development/TESTING.html"><strong aria-hidden="true">15.</strong> Testing Guide</a></li><li class="chapter-item expanded "><a href="development/plugins.html"><strong aria-hidden="true">16.</strong> Plugin Development</a></li><li class="chapter-item expanded "><a href="api/index.html"><strong aria-hidden="true">17.</strong> API Overview</a></li><li class="chapter-item expanded "><a href="api/CORE_API.html"><strong aria-hidden="true">18.</strong> Core API</a></li><li class="chapter-item expanded "><a href="api/AI_API.html"><strong aria-hidden="true">19.</strong> AI API</a></li><li class="chapter-item expanded "><a href="api/PLUGIN_API.html"><strong aria-hidden="true">20.</strong> Plugin API</a></li><li class="chapter-item expanded "><a href="enterprise/DEPLOYMENT.html"><strong aria-hidden="true">21.</strong> Deployment</a></li><li class="chapter-item expanded "><a href="enterprise/SCALING.html"><strong aria-hidden="true">22.</strong> Scaling</a></li><li class="chapter-item expanded "><a href="enterprise/MONITORING.html"><strong aria-hidden="true">23.</strong> Monitoring</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split("#")[0].split("?")[0];
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
