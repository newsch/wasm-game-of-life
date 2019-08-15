import {Universe} from "wasm-game-of-life";

var is_running = true;

const togglebtn = document.getElementById("toggle");
togglebtn.addEventListener("click", () => {
    is_running = !is_running;
    if (is_running) {
        requestAnimationFrame(renderLoop);
    }
});

const pre = document.getElementById("game-of-life-canvas");
const universe = Universe.new();

const renderLoop = () => {
    pre.textContent = universe.render();
    universe.tick();

    if (is_running) {
        requestAnimationFrame(renderLoop);
    }
}

requestAnimationFrame(renderLoop);
