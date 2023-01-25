const ctrl = {
    playPauseBtn: document.getElementById("play-pause"),
    stepBtn: document.getElementById("step"),
    patternSlt: document.getElementById("pattern-select"),
    resetBtn: document.getElementById("reset"),
    customTxt: document.getElementById("custom-txt"),
    widthEl: document.getElementById("width"),
    heightEl: document.getElementById("height"),
    customUrlTxt: document.getElementById("custom-url"),
    customUrlBtn: document.getElementById("custom-url-submit"),
    edgeBehaviorSlt: document.getElementById("edge-behavior"),
    speedNum: document.getElementById("speed-num"),
};

export default ctrl;

const speedRange = document.getElementById("speed-range");

/** update range on text input */
ctrl.speedNum.addEventListener("change", event => {
    const log = event.target.value;
    speedRange.value = speedFromLog(log);
});

/** update text input on range change */
speedRange.addEventListener("input", event => {
    const lin = event.target.value;
    const log = Math.round(speedToLog(lin) * 10) / 10;
    ctrl.speedNum.value = log;
    // trigger listeners on text input
    ctrl.speedNum.dispatchEvent(new Event('change'));
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

export function speedToMsPerTick(speed) {
    // speed is ticks per second
    return 1000 / speed;
}
