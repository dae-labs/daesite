import {compressData, decompressData} from "~/utils";

type OnMaxRetriesReachedCallback = () => void;

class Gateway {
  private socket: WebSocket | null = null;
  private reconnectInterval: number = 1000;
  private maxReconnectInterval: number = 30000;
  private reconnectAttempts: number = 0;
  private maxReconnectAttempts: number = 3;
  private hasMaxRetriesCallbackBeenCalled: boolean = false;
  private heartbeatIntervalId: NodeJS.Timeout | null = null;
  private onOpenResolve!: () => void;

  public ready: Promise<void>;

  constructor(
    private url: string,
    private token: string,
    private onMaxRetriesReached?: OnMaxRetriesReachedCallback
  ) {
    this.ready = new Promise((resolve) => {
      this.onOpenResolve = resolve;
    });
    this.connect();
  }

  private connect(): void {
    this.socket = new WebSocket(this.url);
    this.socket.binaryType = "arraybuffer";

    this.socket.onopen = () => {
      console.log("WebSocket connection established.");
      this.resetReconnectState();

      const authMessage = ["Auth", this.token];
      this.sendMessage(authMessage);

      this.onOpenResolve();
      this.startHeartbeat();
    };

    this.socket.onclose = () => {
      console.log("WebSocket connection closed. Attempting to reconnect...");
      this.stopHeartbeat();
      this.reconnectAttempts++;

      if (this.reconnectAttempts === this.maxReconnectAttempts && !this.hasMaxRetriesCallbackBeenCalled) {
        console.log("Reached third failed reconnect attempt. Executing callback.");
        this.hasMaxRetriesCallbackBeenCalled = true;
        this.onMaxRetriesReached?.();
      }

      setTimeout(() => this.connect(), this.reconnectInterval);
      this.reconnectInterval = Math.min(this.reconnectInterval * 2, this.maxReconnectInterval);
    };

    this.socket.onerror = (error) => {
      console.error("WebSocket error:", error);
      this.socket?.close();
    };

    this.socket.onmessage = (event) => {
      const messageData = new Uint8Array(event.data as ArrayBuffer);
      const message = decompressData(messageData);
      this.handleMessage(message);
    };
  }

  public async sendMessage(message: any): Promise<void> {
    await this.ready;
    const compressedData = compressData(message);
    this.socket?.send(compressedData);
  }

  protected handleMessage(message: any): void {
    console.log("Received message:", message);
  }

  private startHeartbeat(): void {
    if (!this.heartbeatIntervalId) {
      this.heartbeatIntervalId = setInterval(() => {
        const heartbeatMessage = { type: "Heartbeat" };
        this.sendMessage(heartbeatMessage);
      }, 10000);
    }
  }

  private stopHeartbeat(): void {
    if (this.heartbeatIntervalId) {
      clearInterval(this.heartbeatIntervalId);
      this.heartbeatIntervalId = null;
    }
  }

  private resetReconnectState(): void {
    this.reconnectInterval = 1000;
    this.reconnectAttempts = 0;
    this.hasMaxRetriesCallbackBeenCalled = false;
  }
}

export {Gateway};
