// Automatically generate a table of contents in the right sidebar
(function() {
    'use strict';

    // Create the TOC container
    var tocContainer = document.createElement('div');
    tocContainer.className = 'page-toc';
    tocContainer.innerHTML = '<h3>On this page</h3><nav id="toc-nav"></nav>';
    
    // Insert the TOC container
    var content = document.querySelector('.content');
    if (content) {
        content.appendChild(tocContainer);
    }

    // Generate TOC from headings
    var tocNav = document.getElementById('toc-nav');
    if (tocNav) {
        var main = document.querySelector('main');
        if (main) {
            var headings = main.querySelectorAll('h1, h2, h3, h4');
            var tocList = document.createElement('ul');
            
            headings.forEach(function(heading) {
                if (!heading.id) {
                    // Generate an ID if one doesn't exist
                    heading.id = heading.textContent.toLowerCase()
                        .replace(/[^\w\s-]/g, '')
                        .replace(/\s+/g, '-');
                }
                
                var listItem = document.createElement('li');
                listItem.className = 'toc-' + heading.tagName.toLowerCase();
                
                var link = document.createElement('a');
                link.href = '#' + heading.id;
                link.textContent = heading.textContent;
                
                listItem.appendChild(link);
                tocList.appendChild(listItem);
            });
            
            tocNav.appendChild(tocList);
        }
    }

    // Highlight current section
    function highlightCurrentSection() {
        var links = document.querySelectorAll('.page-toc a');
        var fromTop = window.scrollY + 150;
        
        // Check if we're at the bottom of the page
        var isAtBottom = (window.innerHeight + window.scrollY) >= document.body.offsetHeight - 50;
        
        var currentSection = null;
        
        // Find the current section based on scroll position
        links.forEach(function(link) {
            var section = document.querySelector(link.hash);
            if (section) {
                var sectionTop = section.offsetTop;
                
                if (fromTop >= sectionTop) {
                    currentSection = link;
                }
            }
        });
        
        // If at bottom and no section is active, activate the last link
        if (isAtBottom && (!currentSection || links.length > 0)) {
            currentSection = links[links.length - 1];
        }
        
        // Remove active from all, then add to current
        links.forEach(function(link) {
            link.classList.remove('active');
        });
        
        if (currentSection) {
            currentSection.classList.add('active');
        }
    }

    window.addEventListener('scroll', highlightCurrentSection);
    highlightCurrentSection();

    // Add chapter names to navigation buttons
    function addChapterNames() {
        var navWrapper = document.querySelector('.nav-wrapper');
        if (!navWrapper) {
            setTimeout(addChapterNames, 100);
            return;
        }
        
        // Find all navigation links
        var allNavLinks = navWrapper.querySelectorAll('a');
        
        allNavLinks.forEach(function(link) {
            if (link.dataset.enhanced) return;
            link.dataset.enhanced = 'true';
            
            var linkHref = link.getAttribute('href');
            if (!linkHref) return;
            
            // Find matching sidebar link
            var sidebarLink = document.querySelector('.sidebar a[href="' + linkHref + '"]');
            if (sidebarLink) {
                var chapterName = sidebarLink.textContent.trim();
                var nameSpan = document.createElement('span');
                nameSpan.className = 'chapter-name';
                nameSpan.textContent = chapterName;
                
                // Check if it's the next button (floated right) or previous button (floated left)
                var isNextButton = link.classList.contains('next') || 
                                  link.getAttribute('rel') === 'next' ||
                                  window.getComputedStyle(link).float === 'right';
                
                if (isNextButton) {
                    // For next button, insert text before the arrow
                    link.insertBefore(nameSpan, link.firstChild);
                } else {
                    // For previous button, append text after the arrow
                    link.appendChild(nameSpan);
                }
            }
        });
    }
    
    // Try multiple times to add chapter names
    setTimeout(addChapterNames, 100);
    setTimeout(addChapterNames, 500);
    setTimeout(addChapterNames, 1000);
})();
