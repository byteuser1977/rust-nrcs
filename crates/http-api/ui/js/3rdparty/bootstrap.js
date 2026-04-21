// Minimal Bootstrap JavaScript for collapse functionality

(function(global, $) {
    'use strict';

    function Collapse(element, options) {
        this.element = element;
        this.options = options || {};
        this.transitioning = false;
        
        if (this.options.toggle) {
            this.toggle();
        }
    }

    Collapse.prototype = {
        show: function() {
            if (this.transitioning || this.element.classList.contains('in')) return;
            
            this.element.classList.remove('collapse');
            this.element.classList.add('collapsing');
            this.element.style.height = '0';
            this.transitioning = true;
            
            var complete = function() {
                this.element.classList.remove('collapsing');
                this.element.classList.add('collapse', 'in');
                this.element.style.height = '';
                this.transitioning = false;
            }.bind(this);
            
            setTimeout(function() {
                this.element.style.height = this.element.scrollHeight + 'px';
                setTimeout(complete, 350);
            }.bind(this), 0);
        },

        hide: function() {
            if (this.transitioning || !this.element.classList.contains('in')) return;
            
            this.element.style.height = this.element.scrollHeight + 'px';
            
            this.element.classList.remove('collapse', 'in');
            this.element.classList.add('collapsing');
            this.transitioning = true;
            
            var complete = function() {
                this.element.classList.remove('collapsing');
                this.element.classList.add('collapse');
                this.transitioning = false;
            }.bind(this);
            
            setTimeout(function() {
                this.element.style.height = '0';
                setTimeout(complete, 350);
            }.bind(this), 0);
        },

        toggle: function() {
            if (this.element.classList.contains('in')) {
                this.hide();
            } else {
                this.show();
            }
        }
    };

    function Plugin(option) {
        return this.each(function() {
            var $this = $(this);
            var data = $this.data('bs.collapse');
            var options = typeof option === 'object' && option;
            
            if (!data) {
                $this.data('bs.collapse', new Collapse(this, options));
            }
            if (typeof option === 'string') {
                data[option]();
            }
        });
    }

    // Event handlers
    $(document).on('click', '[data-toggle="collapse"]', function(e) {
        var target = this.getAttribute('data-target');
        if (target) {
            e.preventDefault();
            var element = document.querySelector(target);
            if (element) {
                var collapse = new Collapse(element, { toggle: true });
                collapse.toggle();
            }
        }
    });

    // Dropdown
    $(document).on('click', '[data-toggle="dropdown"]', function(e) {
        e.preventDefault();
        var parent = this.parentElement;
        parent.classList.toggle('open');
    });

    // Close dropdown when clicking outside
    $(document).on('click', function(e) {
        if (!e.target.closest('.dropdown-toggle') && !e.target.closest('.dropdown-menu')) {
            var dropdowns = document.querySelectorAll('.dropdown.open');
            dropdowns.forEach(function(d) {
                d.classList.remove('open');
            });
        }
    });

    global.Collapse = Collapse;

})(typeof window !== 'undefined' ? window : this, typeof jQuery !== 'undefined' ? jQuery : function() {});
