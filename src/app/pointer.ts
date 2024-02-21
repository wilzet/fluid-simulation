export default class Pointer {
    private position: Float32Array;
    private lastPosition: number[];
    private velocity: Float32Array;
    private pointerMoved: boolean;
    private pointerDown: boolean;

    constructor(position: number[]) {
        this.position = new Float32Array(position);
        this.lastPosition = position;
        this.velocity = new Float32Array([0.0, 0.0]);
        this.pointerMoved = false;
        this.pointerDown = false;
    }

    public update(x: number, y: number, strength: number) {
        if (!this.pointerDown) return;

        this.lastPosition[0] = this.position[0] ?? 0.0;
        this.lastPosition[1] = this.position[1] ?? 0.0;
        this.position[0] = x;
        this.position[1] = y;
        this.velocity[0] = (this.position[0] - this.lastPosition[0]) * strength;
        this.velocity[1] = (this.position[1] - this.lastPosition[1]) * strength;
        this.pointerMoved = Math.abs(this.velocity[0]) > 0.0 || Math.abs(this.velocity[1]) > 0.0;
    }

    public down(x: number, y: number) {
        this.pointerMoved = false;
        this.pointerDown = true;
        this.position[0] = this.lastPosition[0] = x;
        this.position[1] = this.lastPosition[0] = y;
        this.velocity[0] = this.velocity[1] = 0.0;
    }

    public up() {
        this.pointerDown = this.pointerMoved = false;
        this.velocity[0] = this.velocity[1] = 0.0;
    }

    public resetMove() {
        if (!this.pointerMoved) return;

        this.pointerMoved = false;
        this.velocity[0] = this.velocity[1] = 0.0;
    }

    get getPosition() {
        return this.position;
    }

    get getVelocity() {
        return this.velocity;
    }

    get isMoved() {
        return this.pointerMoved;
    }
}