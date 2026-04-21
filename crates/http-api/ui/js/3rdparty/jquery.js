// Minimal jQuery-like implementation for API test page

(function(global) {
    'use strict';

    var $ = function(selector) {
        if (typeof selector === 'string') {
            return new ElementList(document.querySelectorAll(selector));
        } else if (selector instanceof Element) {
            return new ElementList([selector]);
        } else if (selector instanceof ElementList) {
            return selector;
        }
        return new ElementList([]);
    };

    function ElementList(elements) {
        this.elements = Array.from(elements);
        this.length = this.elements.length;
    }

    ElementList.prototype = {
        each: function(fn) {
            this.elements.forEach(fn);
            return this;
        },

        on: function(event, handler) {
            return this.each(function(el) {
                el.addEventListener(event, handler);
            });
        },

        click: function(handler) {
            return this.on('click', handler);
        },

        find: function(selector) {
            var found = [];
            this.each(function(el) {
                var children = el.querySelectorAll(selector);
                found = found.concat(Array.from(children));
            });
            return new ElementList(found);
        },

        text: function(content) {
            if (content === undefined) {
                return this.elements[0] ? this.elements[0].textContent : '';
            }
            return this.each(function(el) {
                el.textContent = content;
            });
        },

        val: function(value) {
            if (value === undefined) {
                return this.elements[0] ? this.elements[0].value : '';
            }
            return this.each(function(el) {
                el.value = value;
            });
        },

        attr: function(name, value) {
            if (value === undefined) {
                return this.elements[0] ? this.elements[0].getAttribute(name) : null;
            }
            return this.each(function(el) {
                el.setAttribute(name, value);
            });
        },

        removeAttr: function(name) {
            return this.each(function(el) {
                el.removeAttribute(name);
            });
        },

        addClass: function(className) {
            return this.each(function(el) {
                el.classList.add(className);
            });
        },

        removeClass: function(className) {
            return this.each(function(el) {
                el.classList.remove(className);
            });
        },

        toggleClass: function(className) {
            return this.each(function(el) {
                el.classList.toggle(className);
            });
        },

        hasClass: function(className) {
            return this.elements[0] ? this.elements[0].classList.contains(className) : false;
        },

        show: function() {
            return this.each(function(el) {
                el.style.display = '';
            });
        },

        hide: function() {
            return this.each(function(el) {
                el.style.display = 'none';
            });
        },

        prop: function(name, value) {
            if (value === undefined) {
                return this.elements[0] ? this.elements[0][name] : undefined;
            }
            return this.each(function(el) {
                el[name] = value;
            });
        },

        data: function(name, value) {
            if (value === undefined) {
                return this.elements[0] ? this.elements[0].dataset[name] : undefined;
            }
            return this.each(function(el) {
                el.dataset[name] = value;
            });
        },

        parent: function() {
            var parents = this.elements.map(function(el) {
                return el.parentElement;
            }).filter(Boolean);
            return new ElementList(parents);
        },

        closest: function(selector) {
            var found = [];
            this.each(function(el) {
                var parent = el.closest(selector);
                if (parent) found.push(parent);
            });
            return new ElementList(found);
        },

        append: function(content) {
            return this.each(function(el) {
                el.insertAdjacentHTML('beforeend', content);
            });
        },

        prepend: function(content) {
            return this.each(function(el) {
                el.insertAdjacentHTML('afterbegin', content);
            });
        },

        serialize: function() {
            var form = this.elements[0];
            if (!form) return '';
            var formData = new FormData(form);
            var params = [];
            formData.forEach(function(value, key) {
                params.push(encodeURIComponent(key) + '=' + encodeURIComponent(value));
            });
            return params.join('&');
        }
    };

    $.ajax = function(options) {
        return new Promise(function(resolve, reject) {
            var xhr = new XMLHttpRequest();
            xhr.open(options.type || 'GET', options.url, true);
            
            if (options.contentType) {
                xhr.setRequestHeader('Content-Type', options.contentType);
            }
            
            xhr.onload = function() {
                if (xhr.status >= 200 && xhr.status < 300) {
                    try {
                        var data = JSON.parse(xhr.responseText);
                        resolve(data);
                    } catch (e) {
                        resolve(xhr.responseText);
                    }
                } else {
                    reject(new Error(xhr.statusText));
                }
            };
            
            xhr.onerror = function() {
                reject(new Error('Network error'));
            };
            
            if (options.data) {
                xhr.send(options.data);
            } else {
                xhr.send();
            }
        });
    };

    $.get = function(url, data, success) {
        if (typeof data === 'function') {
            success = data;
            data = null;
        }
        var fullUrl = url;
        if (data) {
            var params = Object.keys(data).map(function(key) {
                return encodeURIComponent(key) + '=' + encodeURIComponent(data[key]);
            });
            fullUrl = url + '?' + params.join('&');
        }
        return $.ajax({ url: fullUrl, type: 'GET' }).then(success);
    };

    $.post = function(url, data, success) {
        if (typeof data === 'function') {
            success = data;
            data = null;
        }
        return $.ajax({ 
            url: url, 
            type: 'POST', 
            data: data,
            contentType: 'application/x-www-form-urlencoded'
        }).then(success);
    };

    global.jQuery = global.$ = $;

})(typeof window !== 'undefined' ? window : this);
