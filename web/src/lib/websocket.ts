// WebSocket client for real-time updates

export interface PositionUpdate {
  type: 'position_update'
  position_address: string
  timestamp: string
  data: {
    value_usd: string
    pnl_percent: string
    il_percent: string
    in_range: boolean
  }
}

export interface AlertUpdate {
  type: 'alert'
  level: 'info' | 'warning' | 'critical'
  title: string
  message: string
  timestamp: string
}

export type WebSocketMessage = PositionUpdate | AlertUpdate

type MessageHandler = (message: WebSocketMessage) => void

class WebSocketClient {
  private ws: WebSocket | null = null
  private handlers: Set<MessageHandler> = new Set()
  private reconnectAttempts = 0
  private maxReconnectAttempts = 5
  private reconnectDelay = 1000

  connect(endpoint: string) {
    if (this.ws?.readyState === WebSocket.OPEN) {
      return
    }

    const wsUrl = `${window.location.protocol === 'https:' ? 'wss:' : 'ws:'}//${window.location.host}${endpoint}`
    
    this.ws = new WebSocket(wsUrl)

    this.ws.onopen = () => {
      console.log('WebSocket connected')
      this.reconnectAttempts = 0
    }

    this.ws.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data) as WebSocketMessage
        this.handlers.forEach(handler => handler(message))
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error)
      }
    }

    this.ws.onclose = () => {
      console.log('WebSocket disconnected')
      this.attemptReconnect(endpoint)
    }

    this.ws.onerror = (error) => {
      console.error('WebSocket error:', error)
    }
  }

  private attemptReconnect(endpoint: string) {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      this.reconnectAttempts++
      const delay = this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1)
      console.log(`Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts})`)
      setTimeout(() => this.connect(endpoint), delay)
    }
  }

  disconnect() {
    if (this.ws) {
      this.ws.close()
      this.ws = null
    }
  }

  subscribe(handler: MessageHandler) {
    this.handlers.add(handler)
    return () => this.handlers.delete(handler)
  }

  send(message: unknown) {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message))
    }
  }
}

export const positionsWs = new WebSocketClient()
export const alertsWs = new WebSocketClient()

export function connectWebSockets() {
  positionsWs.connect('/ws/positions')
  alertsWs.connect('/ws/alerts')
}

export function disconnectWebSockets() {
  positionsWs.disconnect()
  alertsWs.disconnect()
}
