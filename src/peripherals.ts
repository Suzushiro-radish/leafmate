export interface PeripheralCallbacks {
  moveTo: (direction: "left" | "right") => void;
  goForward: () => void;
}

export default class Peripherals {
  private targets: EventTarget[] = [];
  private readonly callbacks: PeripheralCallbacks;
  private readonly handleKeyUp = (e: KeyboardEvent) => {
    if (e.code === "Space") this.callbacks.goForward();
  };
  private readonly handleKeyDown = (e: KeyboardEvent) => {
    if (e.code === "ArrowRight") this.callbacks.moveTo("right");
    else if (e.code === "ArrowLeft") this.callbacks.moveTo("left");
  };

  constructor(callbacks: PeripheralCallbacks) {
    this.callbacks = callbacks;
  }

  destroy() {
    this.targets.forEach((t) => this.unobserve(t));
  }

  unobserve(item: EventTarget) {
    if (!item) return;
    item.removeEventListener("keyup", this.handleKeyUp);
    item.removeEventListener("keydown", this.handleKeyDown);
    this.targets = this.targets.filter((t) => t !== item);
  }

  observe(item: EventTarget) {
    if (!item) return;
    if (this.targets.includes(item)) return;
    item.addEventListener("keyup", this.handleKeyUp);
    item.addEventListener("keydown", this.handleKeyDown);
    this.targets.push(item);
  }
}
