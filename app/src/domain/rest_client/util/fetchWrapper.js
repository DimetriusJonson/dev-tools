const RC_BASE_URL = '${RC_BASE_URL}';

// ****** INTERCEPT FETCH
const originalFetch = window.fetch;

window.fetch = async function (...args) {
    let [resource, config] = args;

    if (typeof resource === 'string') {
        resource = resource + (resource.includes('?') ? "&" : "?") + "rc_base_url=" + encodeURIComponent(RC_BASE_URL);
    } else if (resource instanceof Request) {
        const newUrl = resource.url + (resource.url.includes('?') ? "&" : "?") + "rc_base_url=" + encodeURIComponent(RC_BASE_URL);
        resource = new Request(newUrl, resource);
    }

    return await originalFetch(resource, config);
};

// ****** INTERCEPT XMLHttpRequest

const originalOpen = XMLHttpRequest.prototype.open;
const originalSend = XMLHttpRequest.prototype.send;

XMLHttpRequest.prototype.open = function (method, url, ...args) {
    this._url = url;
    this._method = method;
    let targetUrl = url + (url.includes('?') ? "&" : "?") + "rc_base_url=" + encodeURIComponent(RC_BASE_URL);

    return originalOpen.apply(this, [method, targetUrl, ...args]);
};

XMLHttpRequest.prototype.send = function (body) {
    return originalSend.apply(this, [body]);
};

// ***** INTERCEPT CHANGE DOM
const iframe = window.parent.document.getElementById('html-previewer-frame');
const iframeDoc = iframe.contentWindow.document;

const observer = new MutationObserver((mutations) => {
    mutations.forEach((mutation) => {
        mutation.addedNodes.forEach((node) => {
            if (node.tagName === 'IMG') {
                node.src = node.src + (node.src.includes('?') ? "&" : "?") + "rc_base_url=" + encodeURIComponent(RC_BASE_URL);
            }
        });
    });
});

observer.observe(iframeDoc.documentElement, {
    childList: true,
    subtree: true
});
