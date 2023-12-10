import init, { Counter } from "../wasm/counter.js";

async function app() {
  const env_hello = proces.env.ENV_HELLO;
  console.log("env: ", env_hello);

  console.log("starting init ...");
  await init();
  console.log("init done");

  const counter = new Counter();
  console.log(counter.get_count());
}
await app();
