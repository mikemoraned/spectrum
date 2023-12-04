// import init from "../counter/pkg/counter_bg.wasm?init";

// init().then((instance) => {
//   console.log("inited");
//   // instance.exports.test();
// });

// import init from "../counter/pkg/counter";
import wasmUrl from "./wasm/counter_bg.wasm?url";

export const load = async () => {
  console.log("fetching");
  const response = await fetch(wasmUrl);
  console.log("status", response.status);
  console.log("instantiate");
  const { module, instance } = await WebAssembly.instantiateStreaming(response);
  console.log("instantiated");
  console.dir(module);
  console.dir(instance);
  /* ... */
};

export function setupCounter(element) {
  let counter = 0;
  const setCounter = (count) => {
    counter = count;
    element.innerHTML = `count is ${counter}`;
  };
  element.addEventListener("click", () => setCounter(counter + 1));
  setCounter(0);
}
