# three.wasm

The future of 3D on the web is here.

After years of pushing JavaScript to its limits, we've decided to rewrite three.js entirely in WebAssembly. The result? **three.wasm** — a 10KB binary that renders 3D graphics at 480+ fps.

No more garbage collection pauses. No more JIT warm-up. No more `"use strict"`. Just raw, uncompromising performance.

## Getting Started

```html
<script>
const { instance } = await WebAssembly.instantiateStreaming(
    fetch( 'three.wasm' ), { env: { abort: () => {} } }
);

instance.exports.init();
instance.exports.update( time, aspect );
</script>
```

## Features

- 10KB total. Down from 175KB. You're welcome.
- No dependencies. Not even JavaScript.
- 480+ fps on a spinning cube. Imagine what it could do with two.
- Matrix math computed at native speed.
- WebGL2 rendering pipeline.

## FAQ

**Q: Is this real?**

The `.wasm` file is a real WebAssembly binary. Run `file three.wasm` and see for yourself.

**Q: Can it do everything three.js does?**

It can spin a cube. What more do you need?

**Q: What about TypeScript support?**

WebAssembly is typed at the instruction level. Every type is either `i32`, `i64`, `f32`, or `f64`. No `any`. No `unknown`. No `as const satisfies readonly`. Just math.

**Q: Should I migrate my production app?**

Absolutely. What could go wrong?

## Live Demo

Open `index.html` with a local server:

```bash
npx serve .
```

## License

MIT
