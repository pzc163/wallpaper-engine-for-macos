const test = require('node:test');
const assert = require('node:assert/strict');
const vm = require('node:vm');
const fs = require('node:fs');
const path = require('node:path');

function fixture() {
  const frames = new Map(), messages = [], calls = [], events = [], listeners = {};
  let next = 0;
  const config = {properties: {zoom: {value: 100}, mode: {value: 'single'}}, fps: 60, paused: false, volume: 0.5, muted: false};
  const document = {
    querySelectorAll: () => [],
    elementFromPoint: () => ({dispatchEvent: event => events.push(event)}),
    documentElement: {toggleAttribute() {}},
    createElement: () => ({}), head: {appendChild() {}},
  };
  const context = vm.createContext({document, innerWidth: 1280, innerHeight: 800,
    MutationObserver: class {observe() {}}, MouseEvent: class {constructor(type, init) {Object.assign(this, {type}, init);}},
    requestAnimationFrame: fn => {const id = ++next; frames.set(id, fn); return id;},
    cancelAnimationFrame: id => frames.delete(id),
    addEventListener: (name, fn) => {listeners[name] = fn;},
    webkit: {messageHandlers: {wallpaperHost: {postMessage: message => messages.push(message)}}},
  });
  context.window = context;
  const source = fs.readFileSync(path.join(__dirname, '../src/web/host.js'), 'utf8');
  vm.runInContext(source.replace('__WALLPAPER_CONFIG__', JSON.stringify(config)), context);
  context.wallpaperPropertyListener = {
    applyUserProperties: value => calls.push(['properties', JSON.parse(JSON.stringify(value))]),
    applyGeneralProperties: value => calls.push(['general', JSON.parse(JSON.stringify(value))]),
    setPaused: value => calls.push(['paused', value]),
  };
  return {context, config, messages, calls, events, listeners, tick(time) {
    const batch = [...frames.values()]; frames.clear(); batch.forEach(fn => fn(time));
  }};
}

test('default properties arrive once, updates carry only changed values, reset restores defaults', () => {
  const f = fixture(); f.context.__wallpaperHost.initialize();
  assert.deepEqual(f.calls, [['properties', f.config.properties], ['general', {fps: 60}]]);
  f.context.__wallpaperHost.apply({...f.config, properties: {...f.config.properties, zoom: {value: 200}}});
  assert.deepEqual(f.calls.at(-1), ['properties', {zoom: {value: 200}}]);
  f.context.__wallpaperHost.apply(f.config);
  assert.deepEqual(f.calls.at(-1), ['properties', {zoom: {value: 100}}]);
});

test('animation respects fps, pause, resume and cancellation without dropping scheduled work', () => {
  const f = fixture(); f.context.__wallpaperHost.initialize();
  let draws = 0;
  const draw = () => {draws++; f.context.requestAnimationFrame(draw);};
  f.context.requestAnimationFrame(draw);
  const cancelled = f.context.requestAnimationFrame(() => assert.fail('cancelled callback ran'));
  f.context.cancelAnimationFrame(cancelled);
  f.tick(0); f.tick(5); f.tick(17); assert.equal(draws, 2);
  f.context.__wallpaperHost.apply({...f.config, paused: true});
  f.tick(100); assert.equal(draws, 2);
  f.context.__wallpaperHost.apply({...f.config, fps: 30});
  f.tick(110); f.tick(120); f.tick(144); assert.equal(draws, 4);
  assert.equal(f.messages.filter(m => m.kind === 'frame').length, 1);
  assert.deepEqual(f.calls.filter(c => c[0] === 'paused'), [['paused', true], ['paused', false]]);
});

test('secondary display mouse uses local logical viewport coordinates and DOM button masks', () => {
  const f = fixture();
  f.context.__wallpaperHost.input({type: 'mousemove', x: 0.25, y: 0.75});
  assert.equal(f.events.at(-1).clientX, 320); assert.equal(f.events.at(-1).clientY, 600);
  f.context.__wallpaperHost.input({type: 'mousedown', button: 1});
  assert.equal(f.events.at(-1).button, 2);
  assert.equal(f.events.at(-1).buttons, 2);
  f.context.__wallpaperHost.input({type: 'mouseup', button: 1});
  assert.equal(f.events.at(-1).buttons, 0);
  f.context.__wallpaperHost.input({type: 'mousedown', button: 2});
  assert.equal(f.events.at(-1).button, 1);
  assert.equal(f.events.at(-1).buttons, 4);
});

test('script errors prevent load readiness and are reported with their cause', () => {
  const f = fixture(); f.listeners.unhandledrejection({reason: new Error('WebGL shader failed')});
  assert.throws(() => f.context.__wallpaperHost.initialize(), /WebGL shader failed/);
  assert.equal(f.messages[0].kind, 'error');
});
