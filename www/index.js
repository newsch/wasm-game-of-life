import { Universe, Cell } from "wasm-game-of-life";
import { memory } from "wasm-game-of-life/wasm_game_of_life_bg";

const CELL_SIZE = 5;  // width/height in pixels
// const GRID_COLOR = "#CCCCCC";
const GRID_COLOR = "#FFFFFF";
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
    return animationId === null;
};

const playPauseBtn = document.getElementById("play-pause");
const stepBtn = document.getElementById("step");
const patternSlt = document.getElementById("pattern-select");

function play() {
    // playPauseBtn.textContent = "⏸";
    playPauseBtn.textContent = "⏯";
    stepBtn.enabled = false;
    renderLoop();
    fps.reset();
};

function pause() {
    // playPauseBtn.textContent = "▶";
    playPauseBtn.textContent = "⏯";
    cancelAnimationFrame(animationId);
    stepBtn.enabled = true;
    animationId = null;
}

function reset(pattern) {
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
        default:
            throw "unknown pattern: " + pattern;
    }
}

playPauseBtn.addEventListener("click", () => {
    if (isPaused()) {
        play();
    } else {
        pause();
    }
});

stepBtn.addEventListener("click", () => {
    renderLoop(false);
});

patternSlt.addEventListener("change", event => {
    const pattern = event.target.value;
    reset(pattern);
    drawGrid();
    drawCells();
});

const pre = document.getElementById("game-of-life-canvas");
const universe = Universe.new(64, 64);
universe.reset_fancy();
const width = universe.width();
const height = universe.height();

const canvas = document.getElementById("game-of-life-canvas");
canvas.height = (CELL_SIZE + 1) * height + 1;
canvas.width = (CELL_SIZE + 1) * width + 1;

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
    drawGrid();
    drawCells();
});

function renderLoop(loop = true) {
    // debugger;
    universe.tick();
    drawGrid();
    drawCells();

    if (loop) {
        fps.render();
        animationId = requestAnimationFrame(renderLoop);
    }
}

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
  };

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
};

drawGrid();
drawCells();
// requestAnimationFrame(renderLoop);
