(() => {
    'use strict';
    const nativeRAF = window.requestAnimationFrame.bind(window);
    const nativeCancelRAF = window.cancelAnimationFrame.bind(window);
    const post = (kind, message = '') => window.webkit.messageHandlers.wallpaperHost.postMessage({kind, message});
    let config = __WALLPAPER_CONFIG__;
    let initialized = false, paused = false, stopped = false, firstFrame = false;
    let failure = null, pending = new Map(), nextId = 1, scheduled = 0, lastFrame = -Infinity;
    let mouse = {x: 0.5, y: 0.5, buttons: 0};
    const pausedMedia = new Set();
    const report = error => {
        failure = String(error?.stack || error?.message || error);
        post('error', failure);
    };
    window.addEventListener('error', event => {
        if (event.error || event.message) report(event.error || event.message);
        else if (event.target?.src) report('Failed to load resource: ' + event.target.src);
    }, true);
    window.addEventListener('unhandledrejection', event => report(event.reason));
    function schedule() {
        if (!scheduled && !paused && !stopped && pending.size) scheduled = nativeRAF(pump);
    }
    function pump(time) {
        scheduled = 0;
        if (paused || stopped) return;
        const interval = 1000 / Math.max(1, config.fps);
        if (time - lastFrame + 0.5 >= interval) {
            lastFrame = time;
            const callbacks = pending;
            pending = new Map();
            for (const callback of callbacks.values()) {
                try { callback(time); } catch (error) { report(error); }
            }
            if (initialized && !firstFrame && !failure) {
                firstFrame = true;
                post('frame');
            }
        }
        schedule();
    }
    window.requestAnimationFrame = callback => {
        const id = nextId++;
        pending.set(id, callback);
        schedule();
        return id;
    };
    window.cancelAnimationFrame = id => pending.delete(id);
    function applyMedia() {
        for (const media of document.querySelectorAll('audio, video')) {
            media.volume = config.volume;
            media.muted = config.muted;
            if (paused && !media.paused) { pausedMedia.add(media); media.pause(); }
        }
    }
    function setPaused(value) {
        if (paused === value) return;
        paused = value;
        document.documentElement.toggleAttribute('data-wallpaper-paused', paused);
        if (paused && scheduled) { nativeCancelRAF(scheduled); scheduled = 0; }
        window.wallpaperPropertyListener?.setPaused?.(paused);
        applyMedia();
        if (!paused) {
            for (const media of pausedMedia) media.play().catch(() => {});
            pausedMedia.clear();
            lastFrame = -Infinity;
            schedule();
        }
    }
    function apply(next) {
        const previous = config;
        config = next;
        if (!initialized) return;
        const listener = window.wallpaperPropertyListener;
        const changed = Object.fromEntries(Object.entries(config.properties).filter(([key, value]) =>
            JSON.stringify(value) !== JSON.stringify(previous.properties[key])));
        if (Object.keys(changed).length) listener?.applyUserProperties?.(changed);
        if (config.fps !== previous.fps) listener?.applyGeneralProperties?.({fps: config.fps});
        setPaused(config.paused);
        applyMedia();
    }
    function input(event) {
        // AppKit numbers right=1 and middle=2; DOM uses middle=1 and right=2.
        const button = event.button === 1 ? 2 : event.button === 2 ? 1 : (event.button || 0);
        if (event.type === 'mousemove') { mouse.x = event.x; mouse.y = event.y; }
        if (event.type === 'mousedown') mouse.buttons |= (1 << button);
        if (event.type === 'mouseup') mouse.buttons &= ~(1 << button);
        const buttons = (mouse.buttons & 1) | ((mouse.buttons & 2) << 1) | ((mouse.buttons & 4) >> 1);
        const init = {clientX: mouse.x * innerWidth, clientY: mouse.y * innerHeight,
            button, buttons, bubbles: true};
        const target = document.elementFromPoint(init.clientX, init.clientY) || window;
        target.dispatchEvent(new MouseEvent(event.type, init));
    }
    window.__wallpaperHost = {
        apply, input,
        initialize() {
            if (failure) throw new Error(failure);
            if (initialized) return;
            initialized = true;
            const style = document.createElement('style');
            style.textContent = '[data-wallpaper-paused] * { animation-play-state: paused !important; }';
            document.head.appendChild(style);
            const listener = window.wallpaperPropertyListener;
            listener?.applyUserProperties?.(config.properties);
            listener?.applyGeneralProperties?.({fps: config.fps});
            setPaused(config.paused);
            applyMedia();
            new MutationObserver(applyMedia).observe(document.documentElement, {childList: true, subtree: true});
            // Static documents have no animation callback; acknowledge their first paint too.
            nativeRAF(() => nativeRAF(() => {
                if (!firstFrame && !failure && !stopped) { firstFrame = true; post('frame'); }
            }));
        },
        stop() { stopped = true; if (scheduled) nativeCancelRAF(scheduled); pending.clear(); }
    };
})();
