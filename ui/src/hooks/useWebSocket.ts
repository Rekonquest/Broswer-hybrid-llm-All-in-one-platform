import { useEffect, useRef, useState, useCallback } from 'react';
import {
  WebSocketMessage,
  LLMStatusMessage,
  DocumentUploadedMessage,
  LockdownChangedMessage,
  SandboxOutputMessage,
} from '../types/api';

const WS_URL = 'ws://127.0.0.1:3030';
const RECONNECT_DELAY = 3000;
const MAX_RECONNECT_ATTEMPTS = 10;

export interface WebSocketCallbacks {
  onLLMStatus?: (message: LLMStatusMessage) => void;
  onDocumentUploaded?: (message: DocumentUploadedMessage) => void;
  onLockdownChanged?: (message: LockdownChangedMessage) => void;
  onSandboxOutput?: (message: SandboxOutputMessage) => void;
  onAuditLog?: (message: any) => void;
}

export function useWebSocket(callbacks: WebSocketCallbacks) {
  const [isConnected, setIsConnected] = useState(false);
  const [lastMessage, setLastMessage] = useState<WebSocketMessage | null>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectAttemptsRef = useRef(0);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  const connect = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      return;
    }

    try {
      const ws = new WebSocket(WS_URL);

      ws.onopen = () => {
        console.log('WebSocket connected');
        setIsConnected(true);
        reconnectAttemptsRef.current = 0;
      };

      ws.onmessage = (event) => {
        try {
          const message: WebSocketMessage = JSON.parse(event.data);
          setLastMessage(message);

          // Route message to appropriate callback
          switch (message.type) {
            case 'llm_status':
              callbacks.onLLMStatus?.(message.payload as LLMStatusMessage);
              break;
            case 'document_uploaded':
              callbacks.onDocumentUploaded?.(message.payload as DocumentUploadedMessage);
              break;
            case 'lockdown_changed':
              callbacks.onLockdownChanged?.(message.payload as LockdownChangedMessage);
              break;
            case 'sandbox_output':
              callbacks.onSandboxOutput?.(message.payload as SandboxOutputMessage);
              break;
            case 'audit_log':
              callbacks.onAuditLog?.(message.payload);
              break;
            default:
              console.warn('Unknown WebSocket message type:', message.type);
          }
        } catch (error) {
          console.error('Failed to parse WebSocket message:', error);
        }
      };

      ws.onerror = (error) => {
        console.error('WebSocket error:', error);
      };

      ws.onclose = () => {
        console.log('WebSocket disconnected');
        setIsConnected(false);
        wsRef.current = null;

        // Attempt to reconnect
        if (reconnectAttemptsRef.current < MAX_RECONNECT_ATTEMPTS) {
          reconnectAttemptsRef.current++;
          console.log(
            `Reconnecting... (attempt ${reconnectAttemptsRef.current}/${MAX_RECONNECT_ATTEMPTS})`
          );
          reconnectTimeoutRef.current = setTimeout(connect, RECONNECT_DELAY);
        } else {
          console.error('Max reconnection attempts reached');
        }
      };

      wsRef.current = ws;
    } catch (error) {
      console.error('Failed to create WebSocket connection:', error);
    }
  }, [callbacks]);

  const disconnect = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }

    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }

    setIsConnected(false);
  }, []);

  const send = useCallback((message: any) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify(message));
    } else {
      console.warn('WebSocket is not connected. Message not sent:', message);
    }
  }, []);

  useEffect(() => {
    connect();

    return () => {
      disconnect();
    };
  }, [connect, disconnect]);

  return {
    isConnected,
    lastMessage,
    send,
    connect,
    disconnect,
  };
}

// Convenience hook for specific message types
export function useLLMStatus(callback: (message: LLMStatusMessage) => void) {
  return useWebSocket({
    onLLMStatus: callback,
  });
}

export function useDocumentUpdates(callback: (message: DocumentUploadedMessage) => void) {
  return useWebSocket({
    onDocumentUploaded: callback,
  });
}

export function useLockdownUpdates(callback: (message: LockdownChangedMessage) => void) {
  return useWebSocket({
    onLockdownChanged: callback,
  });
}

export function useSandboxOutput(callback: (message: SandboxOutputMessage) => void) {
  return useWebSocket({
    onSandboxOutput: callback,
  });
}
