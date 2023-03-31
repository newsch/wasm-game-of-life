import init, { Universe, Cell, EdgeBehavior } from "./pkg/wasm_game_of_life.js";
import { Renderer } from "./utils.js";
import ctrl, { speedToMsPerTick } from "./controls.js";

const { memory }  = await init();

const CELL_SIZE = 5;  // width/height in pixels
const GRID_COLOR = "#EEEEEE";
// const GRID_COLOR = "#FFFFFF";
const DEAD_COLOR = "#FFFFFF";
const ALIVE_COLOR = "#000000";

const canvas = document.getElementById("game-of-life-canvas");

const ctx = canvas.getContext("2d");

function play() {
    ctrl.playPauseBtn.textContent = "⏸";
    // playPauseBtn.textContent = "⏯";
    ctrl.stepBtn.enabled = false;
    renderer.loop();
};

function pause() {
    ctrl.playPauseBtn.textContent = "▶";
    // playPauseBtn.textContent = "⏯";
    renderer.cancel();
    ctrl.stepBtn.enabled = true;
}

const isPaused = () => {
    return !renderer.isRunning();
};

function reset(pattern) {
    if (pattern !== "custom" && (ctrl.widthEl.value !== width || ctrl.heightEl.value !== height)) {
        // resize based on dimensions input
        universe.width = ctrl.widthEl.value;
        universe.height = ctrl.heightEl.value;
    }

    switch(pattern) {
        case "fancy":
            universe.reset_fancy();
            break;
        case "random":
            universe.reset_random();
            break;
        case "blank":
            universe.reset_blank();
            break;
        case "custom":
            try {
                const file = new TextEncoder().encode(ctrl.customTxt.value);
                universe.reset_from_file(file);
            } catch(e) {
                console.error(e);
                ctrl.customTxt.setCustomValidity('Parse error: ' + e);
                ctrl.customTxt.reportValidity();
            }
            ctrl.customTxt.setCustomValidity('');
            break;
        default:
            throw "unknown pattern: " + pattern;
    }

    // TODO: update or persist edge behavior
    ctrl.edgeBehaviorSlt.value = "wrap";

    resize_canvas();
    drawGrid();
    drawCells();
}

ctrl.playPauseBtn.addEventListener("click", () => {
    if (isPaused()) {
        play();
    } else {
        pause();
    }
});

ctrl.stepBtn.addEventListener("click", () => {
    renderer.step();
});

ctrl.patternSlt.addEventListener("change", event => {
    const pattern = event.target.value;
    // customTxt.disabled = (pattern !== "custom");
    reset(pattern);
});

ctrl.edgeBehaviorSlt.addEventListener("change", event => {
    const behavior = event.target.value;
    let b
    switch (behavior) {
        case "wrap":
            b = EdgeBehavior.Wrap;
            break;
        case "dead":
            b = EdgeBehavior.Dead;
            break;
        case "alive":
            b = EdgeBehavior.Alive;
            break;
        default:
            throw new TypeError("Unknown edge behavior: " + behavior);
    }
    universe.edge_behavior = b;
});

ctrl.resetBtn.addEventListener("click", () => {
    const pattern = ctrl.patternSlt.value;
    reset(pattern);
});

ctrl.speedNum.addEventListener("change", event => {
    const speed = event.target.value;
    renderer.goalMsPerTick = speedToMsPerTick(speed);
    if (renderer.isRunning()) {
        renderer.cancel();
        renderer.loop();
    }
});

ctrl.customUrlBtn.addEventListener("click", async function (event) {
    const url = ctrl.customUrlTxt.value;
    try {
        const resp = await fetch(url);
        const text = resp.text();
            // .then(r => r.arrayBuffer())
            // .then(b => new Uint8Array(b));
         ctrl.customTxt.value = text;
    } catch(e) {
        console.error(e);
        ctrl.customUrlTxt.setCustomValidity('Error fetching url: ' + e);
        ctrl.customUrlTxt.reportValidity();
    }
    ctrl.customUrlTxt.setCustomValidity('');
});

const universe = Universe.new(64, 64);
universe.reset_fancy();

let width;
let height;

function resize_canvas() {
    const new_width = universe.width;
    const new_height = universe.height;
    if (new_width !== width || new_height !== height) {
        width = new_width;
        height = new_height;
        canvas.height = (CELL_SIZE + 1) * height + 1;
        canvas.width = (CELL_SIZE + 1) * width + 1;
    }
}

resize_canvas();

canvas.addEventListener("click", event => {
    const boundingRect = canvas.getBoundingClientRect();

    const scaleX = canvas.width / boundingRect.width;
    const scaleY = canvas.height / boundingRect.height;

    const canvasLeft = (event.clientX - boundingRect.left) * scaleX;
    const canvasTop = (event.clientY - boundingRect.top) * scaleY;

    const row = Math.min(Math.floor(canvasTop / (CELL_SIZE + 1)), height - 1);
    const col = Math.min(Math.floor(canvasLeft / (CELL_SIZE + 1)), width - 1);

    universe.toggle_cell(row, col);
    renderer.redraw();
});

function drawGrid() {
    ctx.beginPath();
    ctx.strokeStyle = GRID_COLOR;

    // Vertical lines;
    for (let i = 0; i <= width; i++) {
        ctx.moveTo(i * (CELL_SIZE + 1) + 1, 0);
        ctx.lineTo(i * (CELL_SIZE + 1) + 1, (CELL_SIZE + 1) * height + 1);
    }

    // Horizontal lines.
    for (let j = 0; j <= height; j++) {
        ctx.moveTo(0,                           j * (CELL_SIZE + 1) + 1);
        ctx.lineTo((CELL_SIZE + 1) * width + 1, j * (CELL_SIZE + 1) + 1);
    }

    ctx.stroke();
}


function getIndex(row, column) {
    return row * width + column;
}

function drawCells() {
    const cellsPtr = universe.cells();
    const cells = new Uint8Array(memory.buffer, cellsPtr, width * height);

    ctx.beginPath();

    // Alive cells
    ctx.fillStyle = ALIVE_COLOR;
    for (let row = 0; row < height; row++) {
      for (let col = 0; col < width; col++) {
        const idx = getIndex(row, col);

        if (cells[idx] !== Cell.Alive) {
            continue;
        }

        ctx.fillRect(
          col * (CELL_SIZE + 1) + 1,
          row * (CELL_SIZE + 1) + 1,
          CELL_SIZE,
          CELL_SIZE
        );
      }
    }

    // Dead cells
    ctx.fillStyle = DEAD_COLOR;
    for (let row = 0; row < height; row++) {
      for (let col = 0; col < width; col++) {
        const idx = getIndex(row, col);

        if (cells[idx] !== Cell.Dead) {
            continue;
        }

        ctx.fillRect(
          col * (CELL_SIZE + 1) + 1,
          row * (CELL_SIZE + 1) + 1,
          CELL_SIZE,
          CELL_SIZE
        );
      }
    }

    ctx.stroke();
}

function drawCellsDelta() {
    const bornPtr = universe.cells_born();
    const numBorn = universe.cells_born_count();
    const bornCells = new Uint32Array(memory.buffer, bornPtr, numBorn * 2);

    const diedPtr = universe.cells_died();
    const numDied = universe.cells_died_count();
    const diedCells = new Uint32Array(memory.buffer, diedPtr, numDied * 2);

    // Alive cells
    ctx.fillStyle = ALIVE_COLOR;
    for (let i = 0; i < numBorn; i++) {
        const row = bornCells[i*2];
        const col = bornCells[i*2+1];
        const idx = getIndex(row, col);

        ctx.fillRect(
          col * (CELL_SIZE + 1) + 1,
          row * (CELL_SIZE + 1) + 1,
          CELL_SIZE,
          CELL_SIZE
        );
    }

    // Dead cells
    ctx.fillStyle = DEAD_COLOR;
    for (let i = 0; i < numDied; i++) {
        const row = diedCells[i*2];
        const col = diedCells[i*2+1];

        ctx.fillRect(
          col * (CELL_SIZE + 1) + 1,
          row * (CELL_SIZE + 1) + 1,
          CELL_SIZE,
          CELL_SIZE
        );
    }
}

class GoLRenderer extends Renderer {
    method = "delta";

    calculate() {
        switch (this.method) {
            case "full":
                universe.tick();
                break;
            case "delta":
                universe.tick_delta();
                break;
            default:
                throw `Unknown method: ${this.method}`;
        }
    }

    redraw() {
        switch (this.method) {
            case "full":
                drawCells();
                break;
            case "delta":
                drawCellsDelta();
                break;
            default:
                throw `Unknown method: ${this.method}`;
        }
    }
}

const renderer = new GoLRenderer({fpsEl: document.getElementById("fps")});

drawGrid();
drawCells();
// renderer.loop();
