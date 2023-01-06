import { Universe, Cell, EdgeBehavior } from "wasm-game-of-life";
import { memory } from "wasm-game-of-life/wasm_game_of_life_bg";

const CELL_SIZE = 5;  // width/height in pixels
const GRID_COLOR = "#EEEEEE";
// const GRID_COLOR = "#FFFFFF";
const DEAD_COLOR = "#FFFFFF";
const ALIVE_COLOR = "#000000";

let animationId = null;

const fps = new class {
  constructor() {
    this.fps = document.getElementById("fps");
    this.frames = [];
    this.lastFrameTimeStamp = performance.now();
  }

  reset() {
      this.frames = [];
  }

  render() {
    // Convert the delta time since the last frame render into a measure
    // of frames per second.
    const now = performance.now();
    const delta = now - this.lastFrameTimeStamp;
    this.lastFrameTimeStamp = now;
    const fps = 1 / delta * 1000;

    // Save only the latest 100 timings.
    this.frames.push(fps);
    if (this.frames.length > 100) {
      this.frames.shift();
    }

    // Find the max, min, and mean of our 100 latest timings.
    let min = Infinity;
    let max = -Infinity;
    let sum = 0;
    for (let i = 0; i < this.frames.length; i++) {
      sum += this.frames[i];
      min = Math.min(this.frames[i], min);
      max = Math.max(this.frames[i], max);
    }
    let mean = sum / this.frames.length;

    // Render the statistics.
    this.fps.textContent = `FPS: ~${Math.round(mean)} -${Math.round(min)} +${Math.round(max)}`;
  }
};

const isPaused = () => {
    return !Renderer.isRunning();
};

const playPauseBtn = document.getElementById("play-pause");
const stepBtn = document.getElementById("step");
const patternSlt = document.getElementById("pattern-select");
const resetBtn = document.getElementById("reset");
const customTxt = document.getElementById("custom-txt");
const widthEl = document.getElementById("width");
const heightEl = document.getElementById("height");
const speedNum = document.getElementById("speed-num");
const speedRange = document.getElementById("speed-range");
const customUrlTxt = document.getElementById("custom-url");
const customUrlBtn = document.getElementById("custom-url-submit");
const edgeBehaviorSlt = document.getElementById("edge-behavior");

function play() {
    playPauseBtn.textContent = "⏸";
    // playPauseBtn.textContent = "⏯";
    stepBtn.enabled = false;
    Renderer.loop();
    fps.reset();
};

function pause() {
    playPauseBtn.textContent = "▶";
    // playPauseBtn.textContent = "⏯";
    Renderer.cancel();
    stepBtn.enabled = true;
}

function reset(pattern) {
    if (pattern !== "custom" && (widthEl.value !== width || heightEl.value !== height)) {
        // resize based on dimensions input
        universe.width = widthEl.value;
        universe.height = heightEl.value;
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
                const file = new TextEncoder().encode(customTxt.value);
                universe.reset_from_file(file);
            } catch(e) {
                console.error(e);
                customTxt.setCustomValidity('Parse error: ' + e);
                customTxt.reportValidity();
            }
            customTxt.setCustomValidity('');
            break;
        default:
            throw "unknown pattern: " + pattern;
    }

    // TODO: update or persist edge behavior
    edgeBehaviorSlt.value = "wrap";

    resize_canvas();
    drawGrid();
    drawCells();
}

playPauseBtn.addEventListener("click", () => {
    if (isPaused()) {
        play();
    } else {
        pause();
    }
});

stepBtn.addEventListener("click", () => {
    Renderer.step();
});

patternSlt.addEventListener("change", event => {
    const pattern = event.target.value;
    // customTxt.disabled = (pattern !== "custom");
    reset(pattern);
});

edgeBehaviorSlt.addEventListener("change", event => {
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

resetBtn.addEventListener("click", () => {
    const pattern = patternSlt.value;
    reset(pattern);
});

speedNum.addEventListener("change", event => {
    const log = event.target.value;
    speedRange.value = speedFromLog(log);
    Renderer.goalMsPerTick = speedToMsPerTick(log);
    if (Renderer.isRunning()) {
        Renderer.cancel();
        Renderer.loop();
    }
});

speedRange.addEventListener("input", event => {
    const lin = event.target.value;
    const log = Math.round(speedToLog(lin) * 10) / 10;
    speedNum.value = log;
    Renderer.goalMsPerTick = speedToMsPerTick(log);
    if (Renderer.isRunning()) {
        Renderer.cancel();
        Renderer.loop();
    }
});

customUrlBtn.addEventListener("click", async function (event) {
    const url = customUrlTxt.value;
    try {
        const resp = await fetch(url);
        const text = resp.text();
            // .then(r => r.arrayBuffer())
            // .then(b => new Uint8Array(b));
         customTxt.value = text;
         debugger
    } catch(e) {
        console.error(e);
        customUrlTxt.setCustomValidity('Error fetching url: ' + e);
        customUrlTxt.reportValidity();
    }
    customUrlTxt.setCustomValidity('');
});

const linMin = 0.1;
const linMax = 100;
const logMin = Math.log(0.5);
const logMax = Math.log(100);
const speedLinLogScale = (logMax - logMin) / (linMax - linMin);

function speedToLog(lin) {
    return Math.exp(logMin + speedLinLogScale*(lin-linMin));
}

function speedFromLog(log) {
    return (Math.log(log)-logMin)/speedLinLogScale + linMin;
}

function speedToMsPerTick(speed) {
    // speed is ticks per second
    return 1000 / speed;
}

const universe = Universe.new(64, 64);
universe.reset_fancy();

let width;
let height;

const canvas = document.getElementById("game-of-life-canvas");

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

const ctx = canvas.getContext("2d");

canvas.addEventListener("click", event => {
    const boundingRect = canvas.getBoundingClientRect();

    const scaleX = canvas.width / boundingRect.width;
    const scaleY = canvas.height / boundingRect.height;

    const canvasLeft = (event.clientX - boundingRect.left) * scaleX;
    const canvasTop = (event.clientY - boundingRect.top) * scaleY;

    const row = Math.min(Math.floor(canvasTop / (CELL_SIZE + 1)), height - 1);
    const col = Math.min(Math.floor(canvasLeft / (CELL_SIZE + 1)), width - 1);

    universe.toggle_cell(row, col);
    Renderer.redraw();
});

const Renderer = new class {
    constructor() {
        this.previousTimestamp;
        this.animationId;
        this.timeoutId;
        this.method = "delta";
        // this.method = "full";
        this.goalMsPerTick = -Infinity;
    }

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
            // drawGrid();
    }

    loop(timestamp=null) {
        // debugger;

        // skip if no tick needed
        if (timestamp) {
            const difference = timestamp - this.previousTimestamp;
            if (difference < this.goalMsPerTick) {
                this.timeoutId = setTimeout(() => this.loop(performance.now()), this.goalMsPerTick - difference);
                return;
            }
        }

        this.calculate();

        // only draw on new frame
        if (!timestamp || timestamp !== this.previousTimestamp) {
            this.redraw();
        }

        this.previousTimestamp = timestamp;
        fps.render();
        this.animationId = requestAnimationFrame(timestamp => this.loop(timestamp));
    }

    step() {
        this.calculate();
        this.redraw();
    }

    cancel() {
        cancelAnimationFrame(this.animationId);
        this.animationId = null;
        clearTimeout(this.timeoutId);
        this.timeoutId = null;
    }

    isRunning() {
        return this.animationId || this.timeoutId;
    }
};

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

drawGrid();
drawCells();
// Renderer.loop();
