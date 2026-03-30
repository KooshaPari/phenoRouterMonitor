/*
 * htmx SSE extension — minimal implementation for AgilePlus dashboard.
 *
 * Registers the "sse" extension with htmx. When an element carries
 *   hx-ext="sse"  sse-connect="<url>"
 * this extension opens an EventSource and, for each child element that has
 *   sse-swap="<event-name>"
 * or triggers an htmx request when
 *   hx-trigger="sse:<event-name>"
 * is present on a descendant, it fires the custom event on the element so
 * htmx picks it up naturally.
 *
 * This is a lightweight shim. For production use replace with the official
 * htmx/ext/sse.js from the htmx CDN or npm package.
 */
(function () {
  if (typeof htmx === 'undefined') return;

  htmx.defineExtension('sse', {
    init: function (api) {
      this.api = api;
    },

    onEvent: function (name, evt) {
      if (name !== 'htmx:afterProcessNode') return;

      var el = evt.detail.elt;
      if (!el || !el.getAttribute) return;

      var sseUrl = el.getAttribute('sse-connect');
      if (!sseUrl) return;

      // Avoid duplicate connections on the same element.
      if (el.__sseSource) return;

      var source = new EventSource(sseUrl);
      el.__sseSource = source;

      source.onerror = function () {
        // Silently reconnect — EventSource handles this automatically.
      };

      // Listen for named events and re-dispatch them so htmx trigger
      // expressions like "sse:feature_updated" work.
      var eventNames = new Set();

      // Collect sse:<event> triggers from all descendants.
      el.querySelectorAll('[hx-trigger]').forEach(function (child) {
        var trigger = child.getAttribute('hx-trigger') || '';
        trigger.split(',').forEach(function (part) {
          var m = part.trim().match(/^sse:(\S+)/);
          if (m) eventNames.add(m[1]);
        });
      });

      // Also handle sse-swap attributes.
      el.querySelectorAll('[sse-swap]').forEach(function (child) {
        eventNames.add(child.getAttribute('sse-swap'));
      });

      eventNames.forEach(function (eventName) {
        source.addEventListener(eventName, function (sseEvt) {
          // Fire a custom DOM event so htmx trigger="sse:<name>" picks it up.
          var domEvt = new CustomEvent('sse:' + eventName, {
            bubbles: true,
            detail: { data: sseEvt.data },
          });
          el.dispatchEvent(domEvt);
        });
      });

      // Clean up when the element is removed from the DOM.
      var observer = new MutationObserver(function () {
        if (!document.contains(el)) {
          source.close();
          observer.disconnect();
        }
      });
      observer.observe(document.body, { childList: true, subtree: true });
    },
  });
})();
