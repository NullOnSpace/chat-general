let wsSeq = 0;

function nextSeq() {
    return ++wsSeq;
}

class WebSocketClient {
    constructor() {
        this.ws = null;
        this.reconnectAttempts = 0;
        this.maxReconnectAttempts = 5;
        this.reconnectDelay = 1000;
        this.messageHandlers = new Map();
        this.isConnected = false;
        this._token = null;
        this._deviceId = null;
    }

    connect(token, deviceId) {
        this._token = token;
        this._deviceId = deviceId;

        const wsUrl = `${window.location.protocol === 'https:' ? 'wss:' : 'ws:'}//${window.location.host}/ws?token=${encodeURIComponent(token)}&device_id=${encodeURIComponent(deviceId)}`;

        this.ws = new WebSocket(wsUrl);

        this.ws.onopen = () => {
            console.log('WebSocket connected');
            this.isConnected = true;
            this.reconnectAttempts = 0;
            this.emit('connected');
        };

        this.ws.onclose = (event) => {
            console.log('WebSocket disconnected:', event.code, event.reason);
            this.isConnected = false;
            this.emit('disconnected');

            if (event.code !== 1000 && this.reconnectAttempts < this.maxReconnectAttempts) {
                setTimeout(() => {
                    this.reconnectAttempts++;
                    console.log(`Reconnecting... Attempt ${this.reconnectAttempts}`);
                    this.connect(this._token, this._deviceId);
                }, this.reconnectDelay * this.reconnectAttempts);
            }
        };

        this.ws.onerror = (error) => {
            console.error('WebSocket error:', error);
            this.emit('error', error);
        };

        this.ws.onmessage = (event) => {
            try {
                const data = JSON.parse(event.data);
                this.handleMessage(data);
            } catch (e) {
                console.error('Failed to parse message:', e);
            }
        };
    }

    handleMessage(data) {
        const type = data.type;

        switch (type) {
            case 'connected':
                console.log('WebSocket connected:', data);
                this.emit('connected', data);
                break;
            case 'message':
                this.emit('message', data);
                break;
            case 'message_sent':
                this.emit('message_sent', data);
                break;
            case 'typing':
                this.emit('typing', {
                    user_id: data.user_id,
                    conversation_id: data.conversation_id,
                    is_typing: data.is_typing,
                    user_name: data.user_name,
                });
                break;
            case 'presence':
                this.emit(data.is_online ? 'user_online' : 'user_offline', {
                    user_id: data.user_id,
                    device_id: data.device_id,
                });
                break;
            case 'ack':
                this.emit('ack', data);
                break;
            case 'message_read':
                this.emit('message_read', data);
                break;
            case 'error':
                console.error('Server error:', data);
                this.emit('error', data);
                break;
            default:
                console.log('Unknown message type:', type, data);
        }
    }

    send(data) {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify(data));
        } else {
            console.warn('WebSocket is not connected');
        }
    }

    sendMessage(conversationId, content, tempId) {
        const seq = nextSeq();
        this.send({
            type: 'message',
            conversation_id: conversationId,
            content: content,
            message_type: 'text',
            reply_to: null,
            seq: seq,
        });
        return seq;
    }

    sendTyping(conversationId) {
        this.send({
            type: 'typing',
            conversation_id: conversationId,
            is_typing: true,
        });
    }

    markAsRead(conversationId, messageId) {
        this.send({
            type: 'ack',
            message_id: messageId,
            seq: nextSeq(),
        });
    }

    on(event, callback) {
        if (!this.messageHandlers.has(event)) {
            this.messageHandlers.set(event, []);
        }
        this.messageHandlers.get(event).push(callback);
    }

    off(event, callback) {
        if (this.messageHandlers.has(event)) {
            const handlers = this.messageHandlers.get(event);
            const index = handlers.indexOf(callback);
            if (index > -1) {
                handlers.splice(index, 1);
            }
        }
    }

    emit(event, data) {
        if (this.messageHandlers.has(event)) {
            this.messageHandlers.get(event).forEach(callback => callback(data));
        }
    }

    disconnect() {
        if (this.ws) {
            this.ws.close(1000, 'User logout');
            this.ws = null;
        }
        this.isConnected = false;
    }
}

const wsClient = new WebSocketClient();
window.wsClient = wsClient;
