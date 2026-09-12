function convertAbsoluteUrl(url) {
    if (url.startsWith("http://") || url.startsWith("https://")) {
        let parsedUrl = URL.parse(url);
        return parsedUrl.pathname + "?__rc_src_url=" + encodeURIComponent(url);
    }
    return url;
}

// ****** INTERCEPT FETCH
const originalFetch = window.fetch;

window.fetch = async function (...args) {
    let [resource, config] = args;

    if (typeof resource === 'string') {
        resource = convertAbsoluteUrl(resource);
    } else if (resource instanceof Request) {
        resource = new Request(convertAbsoluteUrl(resource.url), resource);
    }

    return await originalFetch(resource, config);
};

// ****** INTERCEPT XMLHttpRequest

const originalOpen = XMLHttpRequest.prototype.open;
const originalSend = XMLHttpRequest.prototype.send;

XMLHttpRequest.prototype.open = function (method, url, ...args) {
    this._url = url;
    this._method = method;
    let targetUrl = convertAbsoluteUrl(url);

    return originalOpen.apply(this, [method, targetUrl, ...args]);
};

XMLHttpRequest.prototype.send = function (body) {
    return originalSend.apply(this, [body]);
};
