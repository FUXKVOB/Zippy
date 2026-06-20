import Counter from "./Counter.js";

const app = document.getElementById("app");
const cmp = Counter({ start: 3 });
cmp.mount(app);
