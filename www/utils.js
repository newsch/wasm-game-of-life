export class Renderer {
    #previousTimestamp = 0;
    #animationId;
    #timeoutId;
    #fps

    goalMsPerTick = 0;

    constructor({ fpsEl = null } = {}) {
        // this.method = "full";
        this.#fps = fpsEl ? new Fps(fpsEl) : null;
    }

    calculate() {}

    redraw() {}

    loop(timestamp=null) {
        // debugger;

        timestamp ??= performance.now();

        // skip if no tick needed
        const difference = timestamp - this.#previousTimestamp;
        if (difference < this.goalMsPerTick) {
            // cancel any animation frame and switch to setTimeout
            this.cancel();
            this.#timeoutId = setTimeout(() => this.loop(), this.goalMsPerTick - difference);
            return;
        }

        this.calculate();

        // only draw on new frame (when called by requestAnimationFrame)
        if (timestamp !== this.#previousTimestamp) {
            this.redraw();
        }

        this.#previousTimestamp = timestamp;
        this.#animationId = requestAnimationFrame(timestamp => this.loop(timestamp));

        this.#fps?.render();
    }

    step() {
        this.calculate();
        this.redraw();
    }

    cancel() {
        cancelAnimationFrame(this.#animationId);
        this.#animationId = null;
        clearTimeout(this.#timeoutId);
        this.#timeoutId = null;
        this.#fps?.reset();
    }

    isRunning() {
        return this.#animationId || this.#timeoutId;
    }
};

class Fps {
  constructor(el) {
    this.fps = el;
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

