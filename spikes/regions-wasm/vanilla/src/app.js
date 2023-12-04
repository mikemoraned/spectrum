import init, { Counter } from "../wasm/counter.js";

async function app() {
  console.log("starting init ...");
  await init();
  console.log("init done");

  const counter = new Counter();
  console.log(counter.get_count());
}
await app();
