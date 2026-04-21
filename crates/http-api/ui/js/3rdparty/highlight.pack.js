// Minimal highlight.js for JSON

var hljs = (function() {
    'use strict';

    function escape(value) {
        return value
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;')
            .replace(/"/g, '&quot;')
            .replace(/'/g, '&#39;');
    }

    function highlightJSON(json) {
        json = json.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
        return json.replace(/("(\\u[a-zA-Z0-9]{4}|\\[^u]|[^\\"])*"(\s*:)?|\b(true|false|null)\b|-?\d+(?:\.\d*)?(?:[eE][+\-]?\d+)?)/g, function(match) {
            var cls = 'number';
            if (/^"/.test(match)) {
                if (/:$/.test(match)) {
                    cls = 'attr';
                } else {
                    cls = 'string';
                }
            } else if (/true|false/.test(match)) {
                cls = 'literal';
            } else if (/null/.test(match)) {
                cls = 'literal';
            }
            return '<span class="hljs-' + cls + '">' + match + '</span>';
        });
    }

    function highlightBlock(block) {
        var text = block.textContent || block.innerText;
        var result;
        
        try {
            var json = JSON.parse(text);
            result = highlightJSON(JSON.stringify(json, null, 2));
        } catch (e) {
            result = highlightJSON(text);
        }
        
        block.innerHTML = result;
        block.classList.add('hljs');
    }

    function highlightAll() {
        var blocks = document.querySelectorAll('pre code');
        blocks.forEach(function(block) {
            highlightBlock(block);
        });
    }

    return {
        highlightBlock: highlightBlock,
        highlightAll: highlightAll,
        json: highlightJSON
    };
})();

if (typeof module !== 'undefined' && module.exports) {
    module.exports = hljs;
}
